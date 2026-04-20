//! YouTube Downloader — concurrency-limited queue with resumable downloads.
//!
//! Eighth Graphics project. Distinctive moves:
//!
//!   * **Concurrency limit as tick-driven slot count** — `max_concurrent`
//!     is the only parallelism primitive. No threads: bandwidth is split
//!     equally across active downloads each tick.
//!   * **Resumable state** — `bytes_done` persists across failures. A
//!     failed download retains its progress; retry resumes from where
//!     it failed (HTTP Range semantics modelled in-memory).
//!   * **Exponential backoff as data** — `RetryPolicy` is a value:
//!     `{max_attempts, initial_ms, multiplier}`. `backoff_for_attempt(n)`
//!     is a pure function; no timers, just tick comparison.
//!   * **Format selection policy** — `FormatPolicy::{MaxQuality,
//!     BestUnder(cap), Lowest}`. Pure function over the format list.
//!     Users pick *the policy*, the system picks *the format*.

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoFormat {
    pub id: String,
    pub resolution: u32, // e.g. 1080 for 1080p
    pub codec: String,
    pub container: String,
    pub size_bytes: u64,
    pub bitrate_bps: u64,
}

#[derive(Debug, Clone)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub formats: Vec<VideoFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatPolicy {
    MaxQuality,
    Lowest,
    BestUnder(u64), // size cap in bytes
}

pub fn pick_format<'a>(formats: &'a [VideoFormat], policy: FormatPolicy) -> Option<&'a VideoFormat> {
    if formats.is_empty() { return None; }
    match policy {
        FormatPolicy::MaxQuality => formats.iter().max_by_key(|f| f.resolution),
        FormatPolicy::Lowest => formats.iter().min_by_key(|f| f.resolution),
        FormatPolicy::BestUnder(cap) => {
            formats.iter()
                .filter(|f| f.size_bytes <= cap)
                .max_by_key(|f| f.resolution)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_ms: u64,
    pub multiplier: u64, // integer multiplier (e.g. 2 for doubling)
}

impl RetryPolicy {
    pub fn default_policy() -> Self {
        Self { max_attempts: 3, initial_ms: 1000, multiplier: 2 }
    }

    /// Backoff ms for retry attempt `n` (1-indexed): initial * multiplier^(n-1).
    pub fn backoff_for_attempt(&self, n: u32) -> u64 {
        if n == 0 { return 0; }
        let mut ms = self.initial_ms;
        for _ in 1..n {
            ms = ms.saturating_mul(self.multiplier);
        }
        ms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    Pending,
    InFlight,
    Retrying { ready_at_ms: u64 },
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Download {
    pub id: u64,
    pub video_id: String,
    pub format: VideoFormat,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub attempts: u32,
    pub state: DownloadState,
}

#[derive(Debug, Clone)]
pub enum ManagerEvent {
    Enqueued { id: u64 },
    Started { id: u64, attempt: u32 },
    Progress { id: u64, bytes_done: u64, bytes_total: u64 },
    Completed { id: u64 },
    Failed { id: u64, attempt: u32 },
    RetryScheduled { id: u64, ready_at_ms: u64 },
    Abandoned { id: u64 }, // out of retry attempts
}

#[derive(Debug, Clone)]
pub struct Manager {
    pub downloads: Vec<Download>,
    pub pending_queue: VecDeque<u64>,
    pub max_concurrent: usize,
    pub bandwidth_bps: u64,
    pub elapsed_ms: u64,
    pub retry_policy: RetryPolicy,
    pub events: Vec<ManagerEvent>,
    next_id: u64,
}

impl Manager {
    pub fn new(max_concurrent: usize, bandwidth_bps: u64, retry_policy: RetryPolicy) -> Self {
        Self {
            downloads: vec![],
            pending_queue: VecDeque::new(),
            max_concurrent,
            bandwidth_bps,
            elapsed_ms: 0,
            retry_policy,
            events: vec![],
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, video_id: impl Into<String>, format: VideoFormat) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let d = Download {
            id,
            video_id: video_id.into(),
            format: format.clone(),
            bytes_done: 0,
            bytes_total: format.size_bytes,
            attempts: 0,
            state: DownloadState::Pending,
        };
        self.downloads.push(d);
        self.pending_queue.push_back(id);
        self.events.push(ManagerEvent::Enqueued { id });
        id
    }

    pub fn active_count(&self) -> usize {
        self.downloads.iter()
            .filter(|d| d.state == DownloadState::InFlight)
            .count()
    }

    fn download_mut(&mut self, id: u64) -> Option<&mut Download> {
        self.downloads.iter_mut().find(|d| d.id == id)
    }

    pub fn download(&self, id: u64) -> Option<&Download> {
        self.downloads.iter().find(|d| d.id == id)
    }

    /// Move any Retrying downloads whose deadline has passed back to Pending.
    fn promote_retries(&mut self) {
        let now = self.elapsed_ms;
        let ready: Vec<u64> = self.downloads.iter()
            .filter_map(|d| match d.state {
                DownloadState::Retrying { ready_at_ms } if ready_at_ms <= now => Some(d.id),
                _ => None,
            })
            .collect();
        for id in ready {
            if let Some(d) = self.download_mut(id) {
                d.state = DownloadState::Pending;
            }
            self.pending_queue.push_back(id);
        }
    }

    /// Start pending downloads up to the concurrency limit.
    fn fill_slots(&mut self) {
        while self.active_count() < self.max_concurrent {
            let Some(id) = self.pending_queue.pop_front() else { break; };
            let attempt = {
                let Some(d) = self.download_mut(id) else { continue; };
                if d.state != DownloadState::Pending { continue; }
                d.attempts += 1;
                d.state = DownloadState::InFlight;
                d.attempts
            };
            self.events.push(ManagerEvent::Started { id, attempt });
        }
    }

    /// Report a failure on a currently in-flight download.
    pub fn report_failure(&mut self, id: u64) {
        let policy = self.retry_policy;
        let now = self.elapsed_ms;
        let (should_retry, ready_at) = {
            let Some(d) = self.download_mut(id) else { return; };
            if d.state != DownloadState::InFlight { return; }
            let can_retry = d.attempts < policy.max_attempts;
            let ready_at = if can_retry {
                now + policy.backoff_for_attempt(d.attempts)
            } else { 0 };
            (can_retry, ready_at)
        };
        if should_retry {
            let attempt = {
                let d = self.download_mut(id).unwrap();
                d.state = DownloadState::Retrying { ready_at_ms: ready_at };
                d.attempts
            };
            self.events.push(ManagerEvent::Failed { id, attempt });
            self.events.push(ManagerEvent::RetryScheduled { id, ready_at_ms: ready_at });
        } else {
            let d = self.download_mut(id).unwrap();
            d.state = DownloadState::Failed;
            let attempt = d.attempts;
            self.events.push(ManagerEvent::Failed { id, attempt });
            self.events.push(ManagerEvent::Abandoned { id });
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.elapsed_ms += ms;
        self.promote_retries();
        self.fill_slots();

        let active_ids: Vec<u64> = self.downloads.iter()
            .filter(|d| d.state == DownloadState::InFlight)
            .map(|d| d.id)
            .collect();

        if active_ids.is_empty() { return; }

        let per_stream_bps = self.bandwidth_bps / active_ids.len() as u64;
        let bytes_this_tick = (per_stream_bps * ms) / 8000; // bps * ms / 8000 = bytes

        for id in active_ids {
            let (progressed, completed, bytes_done, bytes_total) = {
                let d = self.download_mut(id).unwrap();
                let before = d.bytes_done;
                d.bytes_done = (d.bytes_done + bytes_this_tick).min(d.bytes_total);
                let completed = d.bytes_done >= d.bytes_total;
                if completed { d.state = DownloadState::Succeeded; }
                (d.bytes_done > before, completed, d.bytes_done, d.bytes_total)
            };
            if progressed {
                self.events.push(ManagerEvent::Progress {
                    id, bytes_done, bytes_total
                });
            }
            if completed {
                self.events.push(ManagerEvent::Completed { id });
            }
        }
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(id: &str, res: u32, size: u64) -> VideoFormat {
        VideoFormat {
            id: id.to_string(),
            resolution: res,
            codec: "avc1".to_string(),
            container: "mp4".to_string(),
            size_bytes: size,
            bitrate_bps: 1_000_000,
        }
    }

    #[test]
    fn pick_format_max_quality() {
        let fs = vec![fmt("a", 480, 50), fmt("b", 1080, 200), fmt("c", 720, 120)];
        let p = pick_format(&fs, FormatPolicy::MaxQuality).unwrap();
        assert_eq!(p.resolution, 1080);
    }

    #[test]
    fn pick_format_lowest() {
        let fs = vec![fmt("a", 480, 50), fmt("b", 1080, 200), fmt("c", 720, 120)];
        let p = pick_format(&fs, FormatPolicy::Lowest).unwrap();
        assert_eq!(p.resolution, 480);
    }

    #[test]
    fn pick_format_best_under_cap() {
        let fs = vec![fmt("a", 480, 50), fmt("b", 1080, 200), fmt("c", 720, 120)];
        let p = pick_format(&fs, FormatPolicy::BestUnder(150)).unwrap();
        assert_eq!(p.resolution, 720);
    }

    #[test]
    fn pick_format_none_under_cap() {
        let fs = vec![fmt("a", 480, 500), fmt("b", 1080, 2000)];
        let p = pick_format(&fs, FormatPolicy::BestUnder(100));
        assert!(p.is_none());
    }

    #[test]
    fn pick_format_empty() {
        let fs: Vec<VideoFormat> = vec![];
        assert!(pick_format(&fs, FormatPolicy::MaxQuality).is_none());
    }

    #[test]
    fn retry_policy_backoff() {
        let p = RetryPolicy { max_attempts: 5, initial_ms: 1000, multiplier: 2 };
        assert_eq!(p.backoff_for_attempt(1), 1000);
        assert_eq!(p.backoff_for_attempt(2), 2000);
        assert_eq!(p.backoff_for_attempt(3), 4000);
        assert_eq!(p.backoff_for_attempt(4), 8000);
    }

    #[test]
    fn enqueue_adds_to_pending() {
        let mut mgr = Manager::new(2, 8_000_000, RetryPolicy::default_policy());
        let id = mgr.enqueue("v1", fmt("1080p", 1080, 1_000_000));
        assert_eq!(id, 1);
        assert_eq!(mgr.pending_queue.len(), 1);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn tick_starts_pending_up_to_limit() {
        let mut mgr = Manager::new(2, 8_000_000, RetryPolicy::default_policy());
        mgr.enqueue("v1", fmt("a", 720, 10_000_000));
        mgr.enqueue("v2", fmt("b", 720, 10_000_000));
        mgr.enqueue("v3", fmt("c", 720, 10_000_000));
        mgr.tick(1);
        assert_eq!(mgr.active_count(), 2);
        assert_eq!(mgr.pending_queue.len(), 1);
    }

    #[test]
    fn tick_drains_bytes_proportionally() {
        // 8 Mbps shared between 2 streams = 4 Mbps each = 500 KB/s each
        let mut mgr = Manager::new(2, 8_000_000, RetryPolicy::default_policy());
        mgr.enqueue("v1", fmt("a", 720, 1_000_000));
        mgr.enqueue("v2", fmt("b", 720, 1_000_000));
        mgr.tick(1000); // 1 second
        // each should have ~500_000 bytes
        let d1 = mgr.download(1).unwrap();
        let d2 = mgr.download(2).unwrap();
        assert_eq!(d1.bytes_done, 500_000);
        assert_eq!(d2.bytes_done, 500_000);
    }

    #[test]
    fn tick_completes_when_bytes_equal_total() {
        let mut mgr = Manager::new(1, 8_000_000, RetryPolicy::default_policy());
        mgr.enqueue("v1", fmt("a", 720, 1_000_000));
        mgr.tick(1000); // 1 MB/s enough
        let d = mgr.download(1).unwrap();
        assert_eq!(d.bytes_done, 1_000_000);
        assert_eq!(d.state, DownloadState::Succeeded);
    }

    #[test]
    fn failure_retries_with_backoff() {
        let policy = RetryPolicy { max_attempts: 3, initial_ms: 1000, multiplier: 2 };
        let mut mgr = Manager::new(1, 8_000_000, policy);
        mgr.enqueue("v1", fmt("a", 720, 10_000_000));
        mgr.tick(100); // start it
        let d = mgr.download(1).unwrap();
        assert_eq!(d.state, DownloadState::InFlight);
        assert_eq!(d.attempts, 1);

        mgr.report_failure(1);
        let d = mgr.download(1).unwrap();
        match d.state {
            DownloadState::Retrying { ready_at_ms } => {
                assert_eq!(ready_at_ms, 100 + 1000);
            }
            _ => panic!("expected Retrying"),
        }
    }

    #[test]
    fn retry_resumes_bytes_done() {
        let policy = RetryPolicy { max_attempts: 3, initial_ms: 100, multiplier: 2 };
        let mut mgr = Manager::new(1, 8_000_000, policy);
        mgr.enqueue("v1", fmt("a", 720, 10_000_000));
        mgr.tick(500); // 500 KB drained @ 1MB/s
        let before = mgr.download(1).unwrap().bytes_done;
        assert!(before > 0);

        mgr.report_failure(1);
        mgr.tick(200); // past backoff of 100ms, retry promoted
        let after = mgr.download(1).unwrap();
        // bytes_done preserved across failure
        assert!(after.bytes_done >= before);
        assert_eq!(after.state, DownloadState::InFlight);
        assert_eq!(after.attempts, 2);
    }

    #[test]
    fn abandoned_after_max_attempts() {
        let policy = RetryPolicy { max_attempts: 2, initial_ms: 10, multiplier: 2 };
        let mut mgr = Manager::new(1, 8_000_000, policy);
        mgr.enqueue("v1", fmt("a", 720, 10_000_000));

        mgr.tick(1);
        mgr.report_failure(1);
        mgr.tick(50); // past backoff
        mgr.report_failure(1);

        let d = mgr.download(1).unwrap();
        assert_eq!(d.state, DownloadState::Failed);
        assert_eq!(d.attempts, 2);
    }

    #[test]
    fn events_log_lifecycle() {
        let mut mgr = Manager::new(1, 8_000_000, RetryPolicy::default_policy());
        mgr.enqueue("v1", fmt("a", 720, 1_000_000));
        mgr.tick(1000);
        let has_enqueued = mgr.events.iter().any(|e| matches!(e, ManagerEvent::Enqueued { .. }));
        let has_started = mgr.events.iter().any(|e| matches!(e, ManagerEvent::Started { .. }));
        let has_completed = mgr.events.iter().any(|e| matches!(e, ManagerEvent::Completed { .. }));
        assert!(has_enqueued);
        assert!(has_started);
        assert!(has_completed);
    }

    #[test]
    fn slot_opens_on_completion() {
        let mut mgr = Manager::new(1, 8_000_000, RetryPolicy::default_policy());
        mgr.enqueue("v1", fmt("a", 720, 1_000_000));
        mgr.enqueue("v2", fmt("b", 720, 1_000_000));
        mgr.tick(1000); // v1 completes
        assert_eq!(mgr.download(1).unwrap().state, DownloadState::Succeeded);
        // v2 should have been picked up
        mgr.tick(0); // force slot fill
        assert_eq!(mgr.download(2).unwrap().state, DownloadState::InFlight);
    }
}
