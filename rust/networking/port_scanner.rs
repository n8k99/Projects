// Port Scanner — probe a host's ports to discover listening services.

use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct PortResult {
    pub port: u16,
    pub status: PortStatus,
    pub service: String,
    pub banner: String,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortStatus {
    Open,
    Closed,
    Filtered,
}

impl std::fmt::Display for PortResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self.status {
            PortStatus::Open => "open",
            PortStatus::Closed => "closed",
            PortStatus::Filtered => "filtered",
        };
        let svc = if self.service.is_empty() { String::new() } else { format!(" ({})", self.service) };
        write!(f, "{:>5}/{}{} ({:.1}ms)", self.port, status, svc, self.latency_ms)
    }
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub host: String,
    pub ports_scanned: usize,
    pub open_ports: Vec<PortResult>,
    pub closed_ports: usize,
    pub filtered_ports: usize,
    pub duration_ms: f64,
}

impl ScanReport {
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("Scan report for {}", self.host),
            format!("  Scanned {} ports in {:.0}ms", self.ports_scanned, self.duration_ms),
            format!("  Open: {}, Closed: {}, Filtered: {}",
                    self.open_ports.len(), self.closed_ports, self.filtered_ports),
        ];
        if !self.open_ports.is_empty() {
            lines.push("  Open ports:".to_string());
            for p in &self.open_ports {
                lines.push(format!("    {}", p));
            }
        }
        lines.join("\n")
    }
}

fn well_known_ports() -> HashMap<u16, &'static str> {
    let pairs: &[(u16, &str)] = &[
        (20, "ftp-data"), (21, "ftp"), (22, "ssh"), (23, "telnet"), (25, "smtp"),
        (53, "dns"), (80, "http"), (110, "pop3"), (123, "ntp"), (135, "msrpc"),
        (139, "netbios-ssn"), (143, "imap"), (443, "https"), (445, "microsoft-ds"),
        (465, "smtps"), (587, "submission"), (993, "imaps"), (995, "pop3s"),
        (1433, "mssql"), (1521, "oracle"), (3306, "mysql"), (3389, "ms-wbt-server"),
        (5432, "postgresql"), (5433, "postgresql-alt"), (5672, "amqp"),
        (5900, "vnc"), (6379, "redis"), (8080, "http-proxy"), (8443, "https-alt"),
        (9200, "elasticsearch"), (27017, "mongodb"),
    ];
    pairs.iter().cloned().collect()
}

pub fn lookup_service(port: u16) -> String {
    well_known_ports().get(&port).map(|s| s.to_string()).unwrap_or_default()
}

pub struct PortScanner {
    pub timeout: Duration,
    pub max_threads: usize,
}

impl PortScanner {
    pub fn new(timeout_ms: u64, max_threads: usize) -> Self {
        PortScanner {
            timeout: Duration::from_millis(timeout_ms),
            max_threads,
        }
    }

    pub fn scan_port(&self, host: &str, port: u16) -> PortResult {
        let addr_str = format!("{}:{}", host, port);
        let start = Instant::now();

        match addr_str.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    match TcpStream::connect_timeout(&addr, self.timeout) {
                        Ok(_stream) => {
                            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                            PortResult {
                                port,
                                status: PortStatus::Open,
                                service: lookup_service(port),
                                banner: String::new(),
                                latency_ms: elapsed,
                            }
                        }
                        Err(e) => {
                            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                            let status = if e.kind() == std::io::ErrorKind::ConnectionRefused {
                                PortStatus::Closed
                            } else {
                                PortStatus::Filtered
                            };
                            PortResult { port, status, service: String::new(), banner: String::new(), latency_ms: elapsed }
                        }
                    }
                } else {
                    PortResult { port, status: PortStatus::Filtered, service: String::new(), banner: String::new(), latency_ms: 0.0 }
                }
            }
            Err(_) => {
                PortResult { port, status: PortStatus::Filtered, service: String::new(), banner: String::new(), latency_ms: 0.0 }
            }
        }
    }

    pub fn scan_range(&self, host: &str, start_port: u16, end_port: u16) -> ScanReport {
        let ports: Vec<u16> = (start_port..=end_port).collect();
        self.scan_ports_list(host, &ports)
    }

    pub fn scan_common(&self, host: &str) -> ScanReport {
        let mut ports: Vec<u16> = well_known_ports().keys().cloned().collect();
        ports.sort();
        self.scan_ports_list(host, &ports)
    }

    pub fn scan_ports_list(&self, host: &str, ports: &[u16]) -> ScanReport {
        let start = Instant::now();
        let results: Arc<Mutex<Vec<PortResult>>> = Arc::new(Mutex::new(Vec::new()));

        // Process in batches to limit thread count
        for chunk in ports.chunks(self.max_threads) {
            let mut handles = Vec::new();
            for &port in chunk {
                let host = host.to_string();
                let timeout = self.timeout;
                let results = Arc::clone(&results);
                handles.push(thread::spawn(move || {
                    let scanner = PortScanner { timeout, max_threads: 1 };
                    let result = scanner.scan_port(&host, port);
                    results.lock().unwrap().push(result);
                }));
            }
            for h in handles {
                let _ = h.join();
            }
        }

        let mut all_results = results.lock().unwrap().clone();
        all_results.sort_by_key(|r| r.port);

        let open_ports: Vec<PortResult> = all_results.iter()
            .filter(|r| r.status == PortStatus::Open)
            .cloned()
            .collect();
        let closed = all_results.iter().filter(|r| r.status == PortStatus::Closed).count();
        let filtered = all_results.iter().filter(|r| r.status == PortStatus::Filtered).count();

        ScanReport {
            host: host.to_string(),
            ports_scanned: ports.len(),
            open_ports,
            closed_ports: closed,
            filtered_ports: filtered,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }
}

pub fn compare_scans(before: &ScanReport, after: &ScanReport) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    let before_open: std::collections::HashSet<u16> = before.open_ports.iter().map(|p| p.port).collect();
    let after_open: std::collections::HashSet<u16> = after.open_ports.iter().map(|p| p.port).collect();

    let mut newly_opened: Vec<u16> = after_open.difference(&before_open).cloned().collect();
    let mut newly_closed: Vec<u16> = before_open.difference(&after_open).cloned().collect();
    let mut still_open: Vec<u16> = before_open.intersection(&after_open).cloned().collect();
    newly_opened.sort();
    newly_closed.sort();
    still_open.sort();
    (newly_opened, newly_closed, still_open)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_service() {
        assert_eq!(lookup_service(22), "ssh");
        assert_eq!(lookup_service(80), "http");
        assert_eq!(lookup_service(443), "https");
        assert_eq!(lookup_service(5432), "postgresql");
        assert_eq!(lookup_service(12345), "");
    }

    #[test]
    fn test_port_result_display() {
        let r = PortResult {
            port: 22, status: PortStatus::Open,
            service: "ssh".into(), banner: String::new(), latency_ms: 1.5,
        };
        let s = format!("{}", r);
        assert!(s.contains("22"));
        assert!(s.contains("open"));
        assert!(s.contains("ssh"));
    }

    #[test]
    fn test_scan_report_summary() {
        let report = ScanReport {
            host: "localhost".into(),
            ports_scanned: 100,
            open_ports: vec![
                PortResult { port: 22, status: PortStatus::Open, service: "ssh".into(), banner: String::new(), latency_ms: 1.0 },
            ],
            closed_ports: 98,
            filtered_ports: 1,
            duration_ms: 500.0,
        };
        let s = report.summary();
        assert!(s.contains("localhost"));
        assert!(s.contains("100 ports"));
        assert!(s.contains("Open: 1"));
    }

    #[test]
    fn test_compare_scans() {
        let before = ScanReport {
            host: "h".into(), ports_scanned: 3,
            open_ports: vec![
                PortResult { port: 22, status: PortStatus::Open, service: String::new(), banner: String::new(), latency_ms: 0.0 },
                PortResult { port: 80, status: PortStatus::Open, service: String::new(), banner: String::new(), latency_ms: 0.0 },
            ],
            closed_ports: 1, filtered_ports: 0, duration_ms: 0.0,
        };
        let after = ScanReport {
            host: "h".into(), ports_scanned: 3,
            open_ports: vec![
                PortResult { port: 80, status: PortStatus::Open, service: String::new(), banner: String::new(), latency_ms: 0.0 },
                PortResult { port: 443, status: PortStatus::Open, service: String::new(), banner: String::new(), latency_ms: 0.0 },
            ],
            closed_ports: 1, filtered_ports: 0, duration_ms: 0.0,
        };
        let (opened, closed, still) = compare_scans(&before, &after);
        assert_eq!(opened, vec![443]);
        assert_eq!(closed, vec![22]);
        assert_eq!(still, vec![80]);
    }
}
