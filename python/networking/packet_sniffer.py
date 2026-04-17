"""Packet Sniffer — capture and analyze network packets with filtering and statistics."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional, Callable
from enum import Enum


class Protocol(Enum):
    TCP = "TCP"
    UDP = "UDP"
    ICMP = "ICMP"
    ARP = "ARP"
    DNS = "DNS"
    HTTP = "HTTP"
    UNKNOWN = "UNKNOWN"


@dataclass
class Packet:
    """A captured network packet."""
    timestamp: datetime
    src_ip: str
    dst_ip: str
    src_port: int
    dst_port: int
    protocol: Protocol
    size: int
    flags: list[str] = field(default_factory=list)
    payload: bytes = b""
    ttl: int = 64

    def __repr__(self) -> str:
        ts = self.timestamp.strftime("%H:%M:%S.%f")[:-3]
        flags_str = f" [{','.join(self.flags)}]" if self.flags else ""
        return (f"{ts} {self.protocol.value:>5} {self.src_ip}:{self.src_port} -> "
                f"{self.dst_ip}:{self.dst_port} {self.size}B{flags_str}")


@dataclass
class CaptureFilter:
    """Filter criteria for packet capture."""
    src_ip: str = ""
    dst_ip: str = ""
    protocol: Optional[Protocol] = None
    port: Optional[int] = None
    min_size: int = 0
    max_size: int = 0

    def matches(self, pkt: Packet) -> bool:
        if self.src_ip and self.src_ip != pkt.src_ip:
            return False
        if self.dst_ip and self.dst_ip != pkt.dst_ip:
            return False
        if self.protocol and self.protocol != pkt.protocol:
            return False
        if self.port is not None:
            if pkt.src_port != self.port and pkt.dst_port != self.port:
                return False
        if self.min_size > 0 and pkt.size < self.min_size:
            return False
        if self.max_size > 0 and pkt.size > self.max_size:
            return False
        return True


@dataclass
class ConnectionKey:
    """Identifies a unique connection (bidirectional)."""
    ip_a: str
    port_a: int
    ip_b: str
    port_b: int
    protocol: Protocol

    @staticmethod
    def from_packet(pkt: Packet) -> "ConnectionKey":
        if (pkt.src_ip, pkt.src_port) <= (pkt.dst_ip, pkt.dst_port):
            return ConnectionKey(pkt.src_ip, pkt.src_port, pkt.dst_ip, pkt.dst_port, pkt.protocol)
        return ConnectionKey(pkt.dst_ip, pkt.dst_port, pkt.src_ip, pkt.src_port, pkt.protocol)

    def __hash__(self) -> int:
        return hash((self.ip_a, self.port_a, self.ip_b, self.port_b, self.protocol))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ConnectionKey):
            return False
        return (self.ip_a == other.ip_a and self.port_a == other.port_a and
                self.ip_b == other.ip_b and self.port_b == other.port_b and
                self.protocol == other.protocol)


@dataclass
class ConnectionStats:
    """Statistics for a single connection."""
    key: ConnectionKey
    packet_count: int = 0
    total_bytes: int = 0
    first_seen: Optional[datetime] = None
    last_seen: Optional[datetime] = None

    def update(self, pkt: Packet) -> None:
        self.packet_count += 1
        self.total_bytes += pkt.size
        if self.first_seen is None or pkt.timestamp < self.first_seen:
            self.first_seen = pkt.timestamp
        if self.last_seen is None or pkt.timestamp > self.last_seen:
            self.last_seen = pkt.timestamp


class PacketCapture:
    """Captures, filters, and analyzes network packets.

    This is the data analysis layer — it processes packets from any source
    (live capture, pcap file, or simulation). The actual packet capture
    mechanism is transport-level; this module is the callable library underneath.
    """

    def __init__(self, capture_filter: Optional[CaptureFilter] = None) -> None:
        self.capture_filter = capture_filter
        self.packets: list[Packet] = []
        self.connections: dict[ConnectionKey, ConnectionStats] = {}
        self._callbacks: list[Callable[[Packet], None]] = []

    def ingest(self, pkt: Packet) -> bool:
        """Ingest a packet. Returns True if it passed the filter."""
        if self.capture_filter and not self.capture_filter.matches(pkt):
            return False
        self.packets.append(pkt)
        key = ConnectionKey.from_packet(pkt)
        if key not in self.connections:
            self.connections[key] = ConnectionStats(key=key)
        self.connections[key].update(pkt)
        for cb in self._callbacks:
            cb(pkt)
        return True

    def on_packet(self, callback: Callable[[Packet], None]) -> None:
        """Register a callback for each captured packet."""
        self._callbacks.append(callback)

    def query(self, q_filter: CaptureFilter) -> list[Packet]:
        """Query captured packets with a post-capture filter."""
        return [p for p in self.packets if q_filter.matches(p)]

    # --- Statistics ---

    def total_packets(self) -> int:
        return len(self.packets)

    def total_bytes(self) -> int:
        return sum(p.size for p in self.packets)

    def protocol_breakdown(self) -> dict[Protocol, int]:
        breakdown: dict[Protocol, int] = {}
        for p in self.packets:
            breakdown[p.protocol] = breakdown.get(p.protocol, 0) + 1
        return breakdown

    def top_talkers(self, n: int = 10) -> list[tuple[str, int]]:
        """Return the top N source IPs by packet count."""
        counts: dict[str, int] = {}
        for p in self.packets:
            counts[p.src_ip] = counts.get(p.src_ip, 0) + 1
        return sorted(counts.items(), key=lambda x: x[1], reverse=True)[:n]

    def top_connections(self, n: int = 10) -> list[ConnectionStats]:
        """Return the top N connections by byte volume."""
        return sorted(self.connections.values(), key=lambda c: c.total_bytes, reverse=True)[:n]

    def bandwidth_per_second(self) -> list[tuple[int, int]]:
        """Return bytes per second over the capture duration."""
        if not self.packets:
            return []
        start = self.packets[0].timestamp
        buckets: dict[int, int] = {}
        for p in self.packets:
            sec = int((p.timestamp - start).total_seconds())
            buckets[sec] = buckets.get(sec, 0) + p.size
        max_sec = max(buckets.keys()) if buckets else 0
        return [(s, buckets.get(s, 0)) for s in range(max_sec + 1)]

    def unique_ips(self) -> set[str]:
        """Return all unique IP addresses seen."""
        ips: set[str] = set()
        for p in self.packets:
            ips.add(p.src_ip)
            ips.add(p.dst_ip)
        return ips

    def summary(self) -> str:
        lines = [
            f"Capture: {self.total_packets()} packets, {self.total_bytes()} bytes",
            f"  Unique IPs: {len(self.unique_ips())}",
            f"  Connections: {len(self.connections)}",
            f"  Protocol breakdown:",
        ]
        for proto, count in sorted(self.protocol_breakdown().items(), key=lambda x: x[1], reverse=True):
            lines.append(f"    {proto.value:>8}: {count}")
        talkers = self.top_talkers(5)
        if talkers:
            lines.append(f"  Top talkers:")
            for ip, count in talkers:
                lines.append(f"    {ip:>15}: {count} packets")
        return "\n".join(lines)


def detect_syn_flood(capture: PacketCapture, threshold: int = 100) -> list[str]:
    """Detect potential SYN flood: IPs sending many SYN packets without completing handshake."""
    syn_counts: dict[str, int] = {}
    ack_counts: dict[str, int] = {}
    for p in capture.packets:
        if p.protocol == Protocol.TCP:
            if "SYN" in p.flags and "ACK" not in p.flags:
                syn_counts[p.src_ip] = syn_counts.get(p.src_ip, 0) + 1
            if "ACK" in p.flags:
                ack_counts[p.src_ip] = ack_counts.get(p.src_ip, 0) + 1
    suspects = []
    for ip, syns in syn_counts.items():
        acks = ack_counts.get(ip, 0)
        if syns > threshold and (acks == 0 or syns / acks > 10):
            suspects.append(ip)
    return suspects


def detect_port_scan(capture: PacketCapture, threshold: int = 20) -> list[str]:
    """Detect potential port scan: IPs connecting to many different ports on a single host."""
    src_dst_ports: dict[tuple[str, str], set[int]] = {}
    for p in capture.packets:
        if p.protocol == Protocol.TCP and "SYN" in p.flags:
            key = (p.src_ip, p.dst_ip)
            if key not in src_dst_ports:
                src_dst_ports[key] = set()
            src_dst_ports[key].add(p.dst_port)
    suspects = []
    for (src, _dst), ports in src_dst_ports.items():
        if len(ports) >= threshold:
            suspects.append(src)
    return list(set(suspects))


# --- demo ---
if __name__ == "__main__":
    from datetime import timedelta
    import random

    capture = PacketCapture()
    base_time = datetime.now(timezone.utc)
    ips = ["192.168.1.10", "192.168.1.20", "10.0.0.1", "172.16.0.5", "8.8.8.8"]
    protocols = [Protocol.TCP, Protocol.UDP, Protocol.DNS, Protocol.HTTP]

    for i in range(200):
        pkt = Packet(
            timestamp=base_time + timedelta(milliseconds=i * 50),
            src_ip=random.choice(ips),
            dst_ip=random.choice(ips),
            src_port=random.randint(1024, 65535),
            dst_port=random.choice([80, 443, 53, 22, 8080]),
            protocol=random.choice(protocols),
            size=random.randint(40, 1500),
            flags=random.choice([["SYN"], ["ACK"], ["SYN", "ACK"], ["FIN"], []]),
        )
        capture.ingest(pkt)

    print(capture.summary())
