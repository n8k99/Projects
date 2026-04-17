// Packet Sniffer — capture and analyze network packets with filtering and statistics.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetProtocol {
    Tcp, Udp, Icmp, Arp, Dns, Http, Unknown,
}

impl NetProtocol {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP", Self::Udp => "UDP", Self::Icmp => "ICMP",
            Self::Arp => "ARP", Self::Dns => "DNS", Self::Http => "HTTP",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub timestamp_ms: u64,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: NetProtocol,
    pub size: usize,
    pub flags: Vec<String>,
    pub payload: Vec<u8>,
    pub ttl: u8,
}

impl std::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flags = if self.flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", self.flags.join(","))
        };
        write!(f, "{} {:>5} {}:{} -> {}:{} {}B{}",
               self.timestamp_ms, self.protocol.name(),
               self.src_ip, self.src_port, self.dst_ip, self.dst_port,
               self.size, flags)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CaptureFilter {
    pub src_ip: String,
    pub dst_ip: String,
    pub protocol: Option<NetProtocol>,
    pub port: Option<u16>,
    pub min_size: usize,
    pub max_size: usize,
}

impl CaptureFilter {
    pub fn matches(&self, pkt: &Packet) -> bool {
        if !self.src_ip.is_empty() && self.src_ip != pkt.src_ip { return false; }
        if !self.dst_ip.is_empty() && self.dst_ip != pkt.dst_ip { return false; }
        if let Some(proto) = self.protocol { if proto != pkt.protocol { return false; } }
        if let Some(port) = self.port {
            if pkt.src_port != port && pkt.dst_port != port { return false; }
        }
        if self.min_size > 0 && pkt.size < self.min_size { return false; }
        if self.max_size > 0 && pkt.size > self.max_size { return false; }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub ip_a: String,
    pub port_a: u16,
    pub ip_b: String,
    pub port_b: u16,
    pub protocol: NetProtocol,
}

impl ConnectionKey {
    pub fn from_packet(pkt: &Packet) -> Self {
        if (&pkt.src_ip, pkt.src_port) <= (&pkt.dst_ip, pkt.dst_port) {
            ConnectionKey {
                ip_a: pkt.src_ip.clone(), port_a: pkt.src_port,
                ip_b: pkt.dst_ip.clone(), port_b: pkt.dst_port,
                protocol: pkt.protocol,
            }
        } else {
            ConnectionKey {
                ip_a: pkt.dst_ip.clone(), port_a: pkt.dst_port,
                ip_b: pkt.src_ip.clone(), port_b: pkt.src_port,
                protocol: pkt.protocol,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub key: ConnectionKey,
    pub packet_count: usize,
    pub total_bytes: usize,
    pub first_seen: u64,
    pub last_seen: u64,
}

impl ConnectionStats {
    pub fn new(key: ConnectionKey, ts: u64) -> Self {
        ConnectionStats { key, packet_count: 0, total_bytes: 0, first_seen: ts, last_seen: ts }
    }

    pub fn update(&mut self, pkt: &Packet) {
        self.packet_count += 1;
        self.total_bytes += pkt.size;
        if pkt.timestamp_ms < self.first_seen { self.first_seen = pkt.timestamp_ms; }
        if pkt.timestamp_ms > self.last_seen { self.last_seen = pkt.timestamp_ms; }
    }
}

pub struct PacketCapture {
    pub filter: Option<CaptureFilter>,
    pub packets: Vec<Packet>,
    pub connections: HashMap<ConnectionKey, ConnectionStats>,
}

impl PacketCapture {
    pub fn new(filter: Option<CaptureFilter>) -> Self {
        PacketCapture { filter, packets: Vec::new(), connections: HashMap::new() }
    }

    pub fn ingest(&mut self, pkt: Packet) -> bool {
        if let Some(ref f) = self.filter {
            if !f.matches(&pkt) { return false; }
        }
        let key = ConnectionKey::from_packet(&pkt);
        let ts = pkt.timestamp_ms;
        let conn = self.connections.entry(key.clone())
            .or_insert_with(|| ConnectionStats::new(key, ts));
        conn.update(&pkt);
        self.packets.push(pkt);
        true
    }

    pub fn query(&self, f: &CaptureFilter) -> Vec<&Packet> {
        self.packets.iter().filter(|p| f.matches(p)).collect()
    }

    pub fn total_packets(&self) -> usize { self.packets.len() }
    pub fn total_bytes(&self) -> usize { self.packets.iter().map(|p| p.size).sum() }

    pub fn protocol_breakdown(&self) -> HashMap<NetProtocol, usize> {
        let mut m = HashMap::new();
        for p in &self.packets {
            *m.entry(p.protocol).or_insert(0) += 1;
        }
        m
    }

    pub fn top_talkers(&self, n: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for p in &self.packets {
            *counts.entry(&p.src_ip).or_insert(0) += 1;
        }
        let mut v: Vec<(String, usize)> = counts.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.truncate(n);
        v
    }

    pub fn top_connections(&self, n: usize) -> Vec<&ConnectionStats> {
        let mut v: Vec<&ConnectionStats> = self.connections.values().collect();
        v.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        v.truncate(n);
        v
    }

    pub fn unique_ips(&self) -> Vec<String> {
        let mut ips: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for p in &self.packets {
            ips.insert(&p.src_ip);
            ips.insert(&p.dst_ip);
        }
        let mut v: Vec<String> = ips.into_iter().map(|s| s.to_string()).collect();
        v.sort();
        v
    }

    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("Capture: {} packets, {} bytes", self.total_packets(), self.total_bytes()),
            format!("  Unique IPs: {}", self.unique_ips().len()),
            format!("  Connections: {}", self.connections.len()),
            "  Protocol breakdown:".to_string(),
        ];
        let mut breakdown: Vec<(NetProtocol, usize)> = self.protocol_breakdown().into_iter().collect();
        breakdown.sort_by(|a, b| b.1.cmp(&a.1));
        for (proto, count) in breakdown {
            lines.push(format!("    {:>8}: {}", proto.name(), count));
        }
        let talkers = self.top_talkers(5);
        if !talkers.is_empty() {
            lines.push("  Top talkers:".to_string());
            for (ip, count) in talkers {
                lines.push(format!("    {:>15}: {} packets", ip, count));
            }
        }
        lines.join("\n")
    }
}

pub fn detect_syn_flood(capture: &PacketCapture, threshold: usize) -> Vec<String> {
    let mut syn_counts: HashMap<&str, usize> = HashMap::new();
    let mut ack_counts: HashMap<&str, usize> = HashMap::new();
    for p in &capture.packets {
        if p.protocol == NetProtocol::Tcp {
            if p.flags.contains(&"SYN".to_string()) && !p.flags.contains(&"ACK".to_string()) {
                *syn_counts.entry(&p.src_ip).or_insert(0) += 1;
            }
            if p.flags.contains(&"ACK".to_string()) {
                *ack_counts.entry(&p.src_ip).or_insert(0) += 1;
            }
        }
    }
    let mut suspects = Vec::new();
    for (ip, syns) in &syn_counts {
        let acks = ack_counts.get(ip).copied().unwrap_or(0);
        if *syns > threshold && (acks == 0 || *syns / acks > 10) {
            suspects.push(ip.to_string());
        }
    }
    suspects.sort();
    suspects
}

pub fn detect_port_scan(capture: &PacketCapture, threshold: usize) -> Vec<String> {
    let mut src_dst_ports: HashMap<(&str, &str), std::collections::HashSet<u16>> = HashMap::new();
    for p in &capture.packets {
        if p.protocol == NetProtocol::Tcp && p.flags.contains(&"SYN".to_string()) {
            src_dst_ports.entry((&p.src_ip, &p.dst_ip))
                .or_default()
                .insert(p.dst_port);
        }
    }
    let mut suspects: std::collections::HashSet<String> = std::collections::HashSet::new();
    for ((src, _), ports) in &src_dst_ports {
        if ports.len() >= threshold {
            suspects.insert(src.to_string());
        }
    }
    let mut v: Vec<String> = suspects.into_iter().collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pkt(src: &str, dst: &str, sp: u16, dp: u16, proto: NetProtocol,
                size: usize, flags: &[&str], ts: u64) -> Packet {
        Packet {
            timestamp_ms: ts, src_ip: src.into(), dst_ip: dst.into(),
            src_port: sp, dst_port: dp, protocol: proto, size,
            flags: flags.iter().map(|s| s.to_string()).collect(),
            payload: vec![], ttl: 64,
        }
    }

    #[test]
    fn test_capture_filter() {
        let pkt = make_pkt("10.0.0.1", "10.0.0.2", 1234, 80, NetProtocol::Tcp, 100, &[], 0);
        let f = CaptureFilter { src_ip: "10.0.0.1".into(), ..Default::default() };
        assert!(f.matches(&pkt));
        let f = CaptureFilter { src_ip: "10.0.0.9".into(), ..Default::default() };
        assert!(!f.matches(&pkt));
        let f = CaptureFilter { protocol: Some(NetProtocol::Udp), ..Default::default() };
        assert!(!f.matches(&pkt));
        let f = CaptureFilter { port: Some(80), ..Default::default() };
        assert!(f.matches(&pkt));
    }

    #[test]
    fn test_ingest_and_stats() {
        let mut cap = PacketCapture::new(None);
        cap.ingest(make_pkt("10.0.0.1", "10.0.0.2", 1234, 80, NetProtocol::Tcp, 100, &[], 0));
        cap.ingest(make_pkt("10.0.0.1", "10.0.0.2", 1234, 80, NetProtocol::Tcp, 200, &[], 1));
        cap.ingest(make_pkt("10.0.0.3", "10.0.0.2", 5555, 53, NetProtocol::Udp, 50, &[], 2));
        assert_eq!(cap.total_packets(), 3);
        assert_eq!(cap.total_bytes(), 350);
        assert_eq!(cap.unique_ips().len(), 3);
        assert_eq!(cap.connections.len(), 2);
        let breakdown = cap.protocol_breakdown();
        assert_eq!(breakdown[&NetProtocol::Tcp], 2);
        assert_eq!(breakdown[&NetProtocol::Udp], 1);
    }

    #[test]
    fn test_connection_key_bidirectional() {
        let p1 = make_pkt("A", "B", 1, 2, NetProtocol::Tcp, 10, &[], 0);
        let p2 = make_pkt("B", "A", 2, 1, NetProtocol::Tcp, 10, &[], 1);
        assert_eq!(ConnectionKey::from_packet(&p1), ConnectionKey::from_packet(&p2));
    }

    #[test]
    fn test_detect_syn_flood() {
        let mut cap = PacketCapture::new(None);
        for i in 0..200 {
            cap.ingest(make_pkt("evil", "victim", 1000 + i, 80, NetProtocol::Tcp, 40, &["SYN"], i as u64));
        }
        cap.ingest(make_pkt("good", "victim", 2000, 80, NetProtocol::Tcp, 40, &["SYN"], 200));
        cap.ingest(make_pkt("good", "victim", 2000, 80, NetProtocol::Tcp, 40, &["ACK"], 201));
        let suspects = detect_syn_flood(&cap, 100);
        assert!(suspects.contains(&"evil".to_string()));
        assert!(!suspects.contains(&"good".to_string()));
    }

    #[test]
    fn test_detect_port_scan() {
        let mut cap = PacketCapture::new(None);
        for port in 1..=30 {
            cap.ingest(make_pkt("scanner", "target", 9999, port, NetProtocol::Tcp, 40, &["SYN"], port as u64));
        }
        let suspects = detect_port_scan(&cap, 20);
        assert!(suspects.contains(&"scanner".to_string()));
    }

    #[test]
    fn test_filtered_capture() {
        let f = CaptureFilter { protocol: Some(NetProtocol::Tcp), ..Default::default() };
        let mut cap = PacketCapture::new(Some(f));
        cap.ingest(make_pkt("A", "B", 1, 80, NetProtocol::Tcp, 100, &[], 0));
        cap.ingest(make_pkt("A", "B", 1, 53, NetProtocol::Udp, 50, &[], 1));
        assert_eq!(cap.total_packets(), 1); // UDP was filtered out
    }
}
