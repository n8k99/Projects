// G034 — Get Atomic Time from Internet Clock
// Time synchronization protocol: in-process NTP-style client/server.

use std::time::{SystemTime, UNIX_EPOCH};

/// Authoritative time source. Wraps system clock with optional fixed offset.
pub struct TimeServer {
    offset_seconds: f64,
}

impl TimeServer {
    pub fn new() -> Self {
        Self { offset_seconds: 0.0 }
    }

    pub fn with_offset(offset: f64) -> Self {
        Self { offset_seconds: offset }
    }

    /// Return authoritative timestamp (epoch seconds).
    pub fn get_time(&self) -> f64 {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards");
        since_epoch.as_secs_f64() + self.offset_seconds
    }
}

/// NTP-style client that queries a TimeServer and tracks synchronization state.
pub struct TimeClient<'a> {
    server: &'a TimeServer,
    local_offset: f64,
    sync_offset: f64,
    synced: bool,
    samples: Vec<f64>,
}

impl<'a> TimeClient<'a> {
    pub fn new(server: &'a TimeServer) -> Self {
        Self {
            server,
            local_offset: 0.0,
            sync_offset: 0.0,
            synced: false,
            samples: Vec::new(),
        }
    }

    /// Create a client whose local clock is off by `drift` seconds.
    pub fn with_drift(server: &'a TimeServer, drift: f64) -> Self {
        Self {
            server,
            local_offset: drift,
            sync_offset: 0.0,
            synced: false,
            samples: Vec::new(),
        }
    }

    fn system_time(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs_f64()
    }

    /// Return the client's uncorrected local time.
    pub fn local_time(&self) -> f64 {
        self.system_time() + self.local_offset
    }

    /// Query the server and return its authoritative timestamp.
    pub fn query_time(&mut self) -> f64 {
        let t_send = self.local_time();
        let server_time = self.server.get_time();
        let t_recv = self.local_time();
        let round_trip = t_recv - t_send;
        let estimated_server_now = server_time + round_trip / 2.0;
        self.samples.push(estimated_server_now - t_recv);
        server_time
    }

    /// Measured offset between local clock and server clock.
    /// Positive means local clock is behind server.
    pub fn get_offset(&mut self) -> f64 {
        if self.samples.is_empty() {
            self.query_time();
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }

    /// Adjust local time estimate by applying the measured offset.
    pub fn sync(&mut self) {
        self.sync_offset = self.get_offset();
        self.synced = true;
    }

    /// Return local time corrected by the synchronization offset.
    pub fn get_synced_time(&self) -> f64 {
        if !self.synced {
            return self.local_time();
        }
        self.local_time() + self.sync_offset
    }

    pub fn is_synced(&self) -> bool {
        self.synced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_returns_time() {
        let server = TimeServer::new();
        let t = server.get_time();
        assert!(t > 1_000_000_000.0, "timestamp should be after year 2001");
    }

    #[test]
    fn test_server_with_offset() {
        let server = TimeServer::with_offset(100.0);
        let baseline = TimeServer::new().get_time();
        let offset_time = server.get_time();
        assert!((offset_time - baseline - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_client_local_time_has_drift() {
        let server = TimeServer::new();
        let client = TimeClient::with_drift(&server, -5.0);
        let diff = server.get_time() - client.local_time();
        assert!((diff - 5.0).abs() < 0.1, "drift should be ~5s");
    }

    #[test]
    fn test_query_returns_server_time() {
        let server = TimeServer::new();
        let mut client = TimeClient::new(&server);
        let queried = client.query_time();
        let actual = server.get_time();
        assert!((queried - actual).abs() < 0.1);
    }

    #[test]
    fn test_offset_detects_drift() {
        let server = TimeServer::new();
        let mut client = TimeClient::with_drift(&server, -3.0);
        let offset = client.get_offset();
        // Offset should be ~+3.0 (local is behind, needs to advance)
        assert!((offset - 3.0).abs() < 0.2, "offset was {offset}, expected ~3.0");
    }

    #[test]
    fn test_sync_corrects_time() {
        let server = TimeServer::new();
        let mut client = TimeClient::with_drift(&server, -2.0);

        // Take multiple samples
        for _ in 0..3 {
            client.query_time();
        }
        client.sync();

        assert!(client.is_synced());
        let error = (client.get_synced_time() - server.get_time()).abs();
        assert!(error < 0.2, "sync error was {error}s, expected < 0.2s");
    }

    #[test]
    fn test_unsynced_returns_local_time() {
        let server = TimeServer::new();
        let client = TimeClient::with_drift(&server, -5.0);
        assert!(!client.is_synced());
        let diff = (client.get_synced_time() - client.local_time()).abs();
        assert!(diff < 0.01, "unsynced should return local time");
    }
}
