//! Progress Bar for Downloads — the first concurrent producer-consumer.
//!
//! One thread (the worker) advances a byte counter as it downloads chunks.
//! Another thread (the observer) reads the counter to render a progress
//! display. The two never block each other: the counter is an atomic that
//! can be written and read concurrently without corruption.
//!
//! This is the first project where "who computes" and "who displays" are
//! distinct actors. Cancellation is cooperative: the main thread sets a
//! flag in the tracker; the worker checks it between chunks and exits
//! cleanly. Completion and failure are states, not flags — the status is
//! an FSM observed from a different thread.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::Instant;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running = 0,
    Completed = 1,
    Cancelled = 2,
    Failed = 3,
}

impl Status {
    fn from_u8(v: u8) -> Status {
        match v {
            0 => Status::Running,
            1 => Status::Completed,
            2 => Status::Cancelled,
            _ => Status::Failed,
        }
    }
}

/// A thread-safe counter + status machine. Shareable across threads via `Arc`.
pub struct ProgressTracker {
    bytes_done: AtomicU64,
    total_bytes: u64,
    status: AtomicU8,
    start_time: Instant,
}

impl ProgressTracker {
    pub fn new(total_bytes: u64) -> Self {
        Self {
            bytes_done: AtomicU64::new(0),
            total_bytes,
            status: AtomicU8::new(Status::Running as u8),
            start_time: Instant::now(),
        }
    }

    pub fn advance(&self, delta: u64) {
        self.bytes_done.fetch_add(delta, Ordering::Relaxed);
    }

    /// Request cancellation. Worker sees this on its next `is_cancelled()` check.
    /// Only transitions Running → Cancelled; later calls are no-ops.
    pub fn cancel(&self) {
        let _ = self.status.compare_exchange(
            Status::Running as u8,
            Status::Cancelled as u8,
            Ordering::SeqCst,
            Ordering::Relaxed,
        );
    }

    pub fn complete(&self) {
        let _ = self.status.compare_exchange(
            Status::Running as u8,
            Status::Completed as u8,
            Ordering::SeqCst,
            Ordering::Relaxed,
        );
    }

    pub fn fail(&self) {
        let _ = self.status.compare_exchange(
            Status::Running as u8,
            Status::Failed as u8,
            Ordering::SeqCst,
            Ordering::Relaxed,
        );
    }

    pub fn is_cancelled(&self) -> bool {
        Status::from_u8(self.status.load(Ordering::Relaxed)) == Status::Cancelled
    }

    /// Read a consistent point-in-time view. Not atomic across fields — the
    /// bytes/status read sequentially — but each field is individually atomic.
    pub fn snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            bytes_done: self.bytes_done.load(Ordering::Relaxed),
            total_bytes: self.total_bytes,
            status: Status::from_u8(self.status.load(Ordering::Relaxed)),
            elapsed_seconds: self.start_time.elapsed().as_secs_f64(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub bytes_done: u64,
    pub total_bytes: u64,
    pub status: Status,
    pub elapsed_seconds: f64,
}

impl ProgressSnapshot {
    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 { return 100.0; }
        (self.bytes_done as f64 / self.total_bytes as f64) * 100.0
    }

    pub fn bytes_per_second(&self) -> f64 {
        if self.elapsed_seconds <= 0.0 { return 0.0; }
        self.bytes_done as f64 / self.elapsed_seconds
    }

    /// ETA in seconds based on current rate. None if not yet progressing or done.
    pub fn eta_seconds(&self) -> Option<f64> {
        let rate = self.bytes_per_second();
        if rate <= 0.0 || self.bytes_done >= self.total_bytes { return None; }
        Some((self.total_bytes - self.bytes_done) as f64 / rate)
    }

    pub fn render_bar(&self, width: usize) -> String {
        let filled = ((self.percent() / 100.0) * width as f64).round() as usize;
        let filled = filled.min(width);
        let empty = width - filled;
        format!("[{}{}] {:>6.2}%", "#".repeat(filled), "-".repeat(empty), self.percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn fresh_tracker_reports_zero_percent() {
        let t = ProgressTracker::new(1000);
        let s = t.snapshot();
        assert_eq!(s.bytes_done, 0);
        assert_eq!(s.percent(), 0.0);
        assert_eq!(s.status, Status::Running);
    }

    #[test]
    fn advancing_updates_percent() {
        let t = ProgressTracker::new(1000);
        t.advance(250);
        t.advance(250);
        let s = t.snapshot();
        assert_eq!(s.bytes_done, 500);
        assert!((s.percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn render_bar_shape() {
        let t = ProgressTracker::new(100);
        t.advance(50);
        let bar = t.snapshot().render_bar(20);
        // "[##########----------]  50.00%"
        assert!(bar.starts_with("["));
        assert!(bar.contains("50.00%"));
        let hashes = bar.chars().filter(|&c| c == '#').count();
        let dashes = bar.chars().filter(|&c| c == '-').count();
        assert_eq!(hashes + dashes, 20);
    }

    #[test]
    fn zero_total_reports_100_percent() {
        // Edge case: a zero-byte "download" is trivially complete.
        let t = ProgressTracker::new(0);
        assert_eq!(t.snapshot().percent(), 100.0);
    }

    #[test]
    fn worker_thread_advances_main_thread_reads() {
        let t = Arc::new(ProgressTracker::new(1000));
        let worker = Arc::clone(&t);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                if worker.is_cancelled() { break; }
                worker.advance(100);
            }
            worker.complete();
        });
        handle.join().unwrap();
        let s = t.snapshot();
        assert_eq!(s.bytes_done, 1000);
        assert_eq!(s.status, Status::Completed);
        assert!((s.percent() - 100.0).abs() < 0.01);
    }

    #[test]
    fn cancellation_stops_worker_early() {
        let t = Arc::new(ProgressTracker::new(1_000_000));
        let worker = Arc::clone(&t);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                if worker.is_cancelled() { return; }
                worker.advance(1000);
                thread::sleep(Duration::from_micros(50));
            }
            worker.complete();
        });
        thread::sleep(Duration::from_millis(5));
        t.cancel();
        handle.join().unwrap();
        let s = t.snapshot();
        assert_eq!(s.status, Status::Cancelled);
        assert!(s.bytes_done < 1_000_000, "should have stopped early, got {}", s.bytes_done);
    }

    #[test]
    fn completion_and_cancellation_are_terminal() {
        // Once Completed, further cancel() is a no-op.
        let t = ProgressTracker::new(100);
        t.advance(100);
        t.complete();
        t.cancel();
        assert_eq!(t.snapshot().status, Status::Completed);
    }

    #[test]
    fn eta_computed_from_rate() {
        let t = ProgressTracker::new(1000);
        t.advance(500);
        // Sleep briefly so elapsed > 0.
        thread::sleep(Duration::from_millis(5));
        let s = t.snapshot();
        let eta = s.eta_seconds().expect("should have eta");
        assert!(eta > 0.0, "eta should be positive, got {eta}");
    }
}
