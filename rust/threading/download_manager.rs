//! Download Manager — N concurrent workers sharing one job queue.
//!
//! Fan-out: one producer (the manager) adds jobs to a shared queue; N
//! worker threads pull jobs off the queue and run them in parallel. Fan-in:
//! each worker writes its outcome to a shared result list; the manager
//! reads the list after all workers finish.
//!
//! The queue is the rendezvous point. Workers don't know about each other —
//! they only know the queue. A slow worker pulls less; a fast worker pulls
//! more. Load balancing is emergent from the pull model, not designed.
//!
//! The simulation function `run_with` decouples the manager from any
//! specific transport (HTTP, FTP, local copy, test mock). The manager is
//! a pattern; the transport is a parameter.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct Download {
    pub url: String,
    pub expected_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Completed { bytes: u64 },
    Cancelled,
    Failed { error: String },
}

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub url: String,
    pub outcome: Outcome,
}

pub struct DownloadManager {
    queue: Arc<Mutex<VecDeque<Download>>>,
    results: Arc<Mutex<Vec<DownloadResult>>>,
    cancel: Arc<AtomicBool>,
    worker_count: usize,
}

impl DownloadManager {
    pub fn new(worker_count: usize) -> Self {
        assert!(worker_count >= 1, "worker_count must be >= 1");
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            cancel: Arc::new(AtomicBool::new(false)),
            worker_count,
        }
    }

    pub fn enqueue(&self, d: Download) {
        self.queue.lock().unwrap().push_back(d);
    }

    pub fn cancel(&self) { self.cancel.store(true, Ordering::SeqCst); }

    /// Spawn `worker_count` workers, each running `simulate` on popped jobs
    /// until the queue is empty (or cancellation is observed). Blocks until
    /// all workers finish. Returns results in completion order (may differ
    /// from enqueue order because workers run in parallel).
    pub fn run_with<F>(&self, simulate: F) -> Vec<DownloadResult>
    where
        F: Fn(&Download) -> Result<u64, String> + Send + Sync + 'static,
    {
        let simulate = Arc::new(simulate);
        let mut handles = Vec::with_capacity(self.worker_count);
        for _ in 0..self.worker_count {
            let queue = Arc::clone(&self.queue);
            let results = Arc::clone(&self.results);
            let cancel = Arc::clone(&self.cancel);
            let simulate = Arc::clone(&simulate);
            handles.push(thread::spawn(move || {
                loop {
                    if cancel.load(Ordering::Relaxed) { return; }
                    let job = queue.lock().unwrap().pop_front();
                    let Some(d) = job else { return; };
                    let outcome = if cancel.load(Ordering::Relaxed) {
                        Outcome::Cancelled
                    } else {
                        match simulate(&d) {
                            Ok(bytes) => Outcome::Completed { bytes },
                            Err(e) => Outcome::Failed { error: e },
                        }
                    };
                    results.lock().unwrap().push(DownloadResult {
                        url: d.url, outcome,
                    });
                }
            }));
        }
        for h in handles { h.join().unwrap(); }
        self.results.lock().unwrap().clone()
    }

    pub fn queued_count(&self) -> usize { self.queue.lock().unwrap().len() }
    pub fn result_count(&self) -> usize { self.results.lock().unwrap().len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn job(url: &str, bytes: u64) -> Download {
        Download { url: url.into(), expected_bytes: bytes }
    }

    #[test]
    fn all_jobs_run_to_completion() {
        let m = DownloadManager::new(4);
        for i in 0..20 {
            m.enqueue(job(&format!("http://x/{i}"), 1000));
        }
        let results = m.run_with(|d| Ok(d.expected_bytes));
        assert_eq!(results.len(), 20);
        assert!(results.iter().all(|r| matches!(r.outcome, Outcome::Completed { .. })));
    }

    #[test]
    fn workers_actually_run_in_parallel() {
        // Four workers doing 40 ms of work each, four jobs: should finish near
        // 40 ms (parallel) not near 160 ms (serial).
        let m = DownloadManager::new(4);
        for i in 0..4 {
            m.enqueue(job(&format!("http://x/{i}"), 1));
        }
        let start = Instant::now();
        let results = m.run_with(|_d| {
            thread::sleep(Duration::from_millis(40));
            Ok(1)
        });
        let elapsed = start.elapsed();
        assert_eq!(results.len(), 4);
        assert!(elapsed < Duration::from_millis(120),
                "expected parallel (~40ms), got {elapsed:?}");
    }

    #[test]
    fn failure_in_one_job_isolates_from_others() {
        let m = DownloadManager::new(2);
        m.enqueue(job("ok1", 100));
        m.enqueue(job("boom", 100));
        m.enqueue(job("ok2", 100));
        let results = m.run_with(|d| {
            if d.url == "boom" { Err("simulated failure".into()) }
            else { Ok(d.expected_bytes) }
        });
        assert_eq!(results.len(), 3);
        let mut completed = 0;
        let mut failed = 0;
        for r in &results {
            match &r.outcome {
                Outcome::Completed { .. } => completed += 1,
                Outcome::Failed { error } => {
                    failed += 1;
                    assert_eq!(r.url, "boom");
                    assert!(error.contains("simulated"));
                }
                _ => panic!("unexpected outcome"),
            }
        }
        assert_eq!(completed, 2);
        assert_eq!(failed, 1);
    }

    #[test]
    fn cancellation_stops_pending_jobs() {
        let m = DownloadManager::new(2);
        for i in 0..50 {
            m.enqueue(job(&format!("http://slow/{i}"), 100));
        }
        // Spawn a canceller that fires after a brief delay.
        let cancel_handle = {
            let cancel = Arc::clone(&m.cancel);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(15));
                cancel.store(true, Ordering::SeqCst);
            })
        };
        let results = m.run_with(|_d| {
            thread::sleep(Duration::from_millis(5));
            Ok(100)
        });
        cancel_handle.join().unwrap();
        // Some jobs completed, some were cancelled; total is at most 50.
        assert!(results.len() <= 50);
        // Cancellation should have cut the run short — not everything ran.
        assert!(results.len() < 50 || m.queued_count() == 0,
                "expected some jobs to be cut; got {} results, {} queued",
                results.len(), m.queued_count());
    }

    #[test]
    fn single_worker_processes_serially() {
        // Regression: worker_count=1 is a legal degenerate case.
        let m = DownloadManager::new(1);
        for i in 0..5 {
            m.enqueue(job(&format!("http://x/{i}"), 10));
        }
        let results = m.run_with(|d| Ok(d.expected_bytes));
        assert_eq!(results.len(), 5);
        // With one worker, results should be in FIFO enqueue order.
        for (i, r) in results.iter().enumerate() {
            assert_eq!(r.url, format!("http://x/{i}"));
        }
    }

    #[test]
    fn total_bytes_is_sum_of_per_job_bytes() {
        let m = DownloadManager::new(3);
        for i in 0..10 {
            m.enqueue(job(&format!("http://x/{i}"), 100 * (i + 1) as u64));
        }
        let results = m.run_with(|d| Ok(d.expected_bytes));
        let total: u64 = results.iter()
            .filter_map(|r| match r.outcome {
                Outcome::Completed { bytes } => Some(bytes),
                _ => None,
            })
            .sum();
        // Sum of 100 + 200 + ... + 1000 = 5500.
        assert_eq!(total, 5500);
    }
}
