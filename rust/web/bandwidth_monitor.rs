//! Bandwidth Monitor — time-series rate calculation over samples.
//!
//! First Rosetta Stone project with **time series as primary data structure**.
//! The raw observations are monotonic byte counts; the derived signal is the
//! rate — bytes per second over a rolling window. Average rate smooths noise;
//! peak rate preserves spikes that averaging hides.
//!
//! The monitor is a fixed-capacity ring buffer of samples; recording a new
//! sample past capacity drops the oldest. Rate queries walk the buffer
//! backward from the latest sample until they span the requested window.
//!
//! Counter wraparound is handled: when a reading is smaller than the
//! previous one, we assume the underlying counter wrapped and skip that
//! pair's contribution rather than reporting an implausible negative rate.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub timestamp_ms: u64,
    pub bytes_total: u64,
}

pub struct Monitor {
    samples: VecDeque<Sample>,
    capacity: usize,
}

impl Monitor {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 2, "need at least 2 samples to compute a rate");
        Self { samples: VecDeque::with_capacity(capacity), capacity }
    }

    /// Record a new sample. Drops the oldest if the ring is full.
    pub fn record(&mut self, timestamp_ms: u64, bytes_total: u64) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(Sample { timestamp_ms, bytes_total });
    }

    pub fn samples(&self) -> &VecDeque<Sample> { &self.samples }

    pub fn sample_count(&self) -> usize { self.samples.len() }

    pub fn latest(&self) -> Option<&Sample> { self.samples.back() }

    /// Bytes observed between the first and the latest sample, net of any
    /// wraparound intervals that are skipped as unmeasurable.
    pub fn total_bytes(&self) -> u64 {
        let mut total: u64 = 0;
        let mut prev: Option<&Sample> = None;
        for s in &self.samples {
            if let Some(p) = prev {
                if s.bytes_total >= p.bytes_total {
                    total = total.saturating_add(s.bytes_total - p.bytes_total);
                }
                // else: wraparound — ignore this interval
            }
            prev = Some(s);
        }
        total
    }

    /// Instantaneous rate between the last two samples, in bytes/second.
    /// Returns 0.0 if fewer than 2 samples or zero time delta.
    pub fn instantaneous_rate(&self) -> f64 {
        let n = self.samples.len();
        if n < 2 { return 0.0; }
        let a = self.samples[n - 2];
        let b = self.samples[n - 1];
        if b.timestamp_ms <= a.timestamp_ms { return 0.0; }
        if b.bytes_total < a.bytes_total { return 0.0; }  // wraparound
        let dt_s = (b.timestamp_ms - a.timestamp_ms) as f64 / 1000.0;
        (b.bytes_total - a.bytes_total) as f64 / dt_s
    }

    /// Average rate over the last `window_ms` milliseconds from the latest
    /// sample, in bytes/second.
    pub fn average_rate(&self, window_ms: u64) -> f64 {
        let Some(latest) = self.samples.back() else { return 0.0; };
        let min_ts = latest.timestamp_ms.saturating_sub(window_ms);
        // Find earliest sample within the window (walk backward from newest).
        let mut earliest: Option<&Sample> = None;
        for s in self.samples.iter().rev() {
            if s.timestamp_ms >= min_ts { earliest = Some(s); }
            else { break; }
        }
        let earliest = match earliest { Some(e) => e, None => return 0.0 };
        if latest.timestamp_ms <= earliest.timestamp_ms { return 0.0; }
        if latest.bytes_total < earliest.bytes_total { return 0.0; }  // wraparound
        let dt_s = (latest.timestamp_ms - earliest.timestamp_ms) as f64 / 1000.0;
        (latest.bytes_total - earliest.bytes_total) as f64 / dt_s
    }

    /// Peak instantaneous rate across all consecutive sample pairs whose
    /// interval starts within the window.
    pub fn peak_rate(&self, window_ms: u64) -> f64 {
        let Some(latest) = self.samples.back() else { return 0.0; };
        let min_ts = latest.timestamp_ms.saturating_sub(window_ms);
        let mut prev: Option<&Sample> = None;
        let mut peak: f64 = 0.0;
        for s in &self.samples {
            if let Some(p) = prev {
                if s.timestamp_ms >= min_ts
                    && s.timestamp_ms > p.timestamp_ms
                    && s.bytes_total >= p.bytes_total
                {
                    let dt_s = (s.timestamp_ms - p.timestamp_ms) as f64 / 1000.0;
                    let rate = (s.bytes_total - p.bytes_total) as f64 / dt_s;
                    if rate > peak { peak = rate; }
                }
            }
            prev = Some(s);
        }
        peak
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn need_at_least_two_samples_for_rate() {
        let mut m = Monitor::new(10);
        assert_eq!(m.instantaneous_rate(), 0.0);
        m.record(0, 0);
        assert_eq!(m.instantaneous_rate(), 0.0);
        m.record(1000, 1000);
        assert_eq!(m.instantaneous_rate(), 1000.0);  // 1000 B in 1s = 1000 B/s
    }

    #[test]
    fn instantaneous_rate_uses_last_two_samples() {
        let mut m = Monitor::new(10);
        m.record(0, 0);
        m.record(1000, 500);     // 500 B/s
        m.record(2000, 1500);    // 1000 B in next 1s = 1000 B/s
        assert!((m.instantaneous_rate() - 1000.0).abs() < 0.01);
    }

    #[test]
    fn average_rate_over_window() {
        let mut m = Monitor::new(10);
        // 10 samples spaced 100ms, each advancing bytes by 50.
        for i in 0..=10u64 {
            m.record(i * 100, i * 50);
        }
        // Over the full 1 second window: 500 B in 1s = 500 B/s.
        assert!((m.average_rate(1000) - 500.0).abs() < 0.01);
        // Over the last 300ms (3 intervals = 150 B): 150 / 0.3 = 500 B/s.
        assert!((m.average_rate(300) - 500.0).abs() < 0.01);
    }

    #[test]
    fn peak_rate_catches_spike_averages_miss() {
        let mut m = Monitor::new(10);
        // Steady 100 B/s with one huge spike.
        m.record(0,    0);
        m.record(1000, 100);
        m.record(2000, 200);
        m.record(2100, 10_200);   // 10,000 B in 100ms = 100,000 B/s spike
        m.record(3100, 10_300);
        // Average over the 3.1 s window: 10_300 B / 3.1 s ≈ 3,322 B/s
        let avg = m.average_rate(3100);
        assert!(avg < 10_000.0, "avg was {avg}, expected far less than spike");
        // Peak should be around 100,000 B/s.
        let peak = m.peak_rate(3100);
        assert!(peak > 50_000.0, "peak was {peak}, expected ~100,000");
    }

    #[test]
    fn counter_wraparound_is_tolerated() {
        let mut m = Monitor::new(10);
        m.record(0,    1_000_000);
        m.record(1000, 500);   // wraparound: counter reset
        // instantaneous_rate should return 0 rather than a negative rate.
        assert_eq!(m.instantaneous_rate(), 0.0);
        // total_bytes should skip the unmeasurable interval.
        assert_eq!(m.total_bytes(), 0);
        // After more normal samples, rates resume.
        m.record(2000, 1500);    // 1000 B in 1s
        assert!((m.instantaneous_rate() - 1000.0).abs() < 0.01);
    }

    #[test]
    fn ring_buffer_drops_oldest_when_full() {
        let mut m = Monitor::new(3);
        m.record(0, 0);
        m.record(1000, 100);
        m.record(2000, 200);
        m.record(3000, 300);     // evicts timestamp 0
        assert_eq!(m.sample_count(), 3);
        let first = m.samples().front().unwrap();
        assert_eq!(first.timestamp_ms, 1000);
    }

    #[test]
    fn total_bytes_skips_wraparound_intervals() {
        let mut m = Monitor::new(10);
        m.record(0,    0);
        m.record(1000, 500);         // +500
        m.record(2000, 10);          // wraparound — skipped
        m.record(3000, 200);         // +190
        assert_eq!(m.total_bytes(), 690);
    }

    #[test]
    fn zero_time_delta_returns_zero_rate() {
        let mut m = Monitor::new(10);
        m.record(1000, 100);
        m.record(1000, 500);         // same timestamp, different bytes
        assert_eq!(m.instantaneous_rate(), 0.0);
    }

    #[test]
    fn empty_window_returns_zero() {
        let mut m = Monitor::new(10);
        m.record(0, 0);
        m.record(10_000, 1_000_000);
        // Very small window that only includes the last sample.
        assert_eq!(m.average_rate(1), 0.0);
    }
}
