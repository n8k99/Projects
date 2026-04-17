"""Port Scanner — probe a host's ports to discover listening services."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional
import socket
import concurrent.futures


@dataclass
class PortResult:
    """Result of scanning a single port."""
    port: int
    status: str          # "open", "closed", "filtered"
    service: str = ""    # well-known service name
    banner: str = ""     # banner grabbed from service
    latency_ms: float = 0.0

    def __repr__(self) -> str:
        svc = f" ({self.service})" if self.service else ""
        ban = f" [{self.banner}]" if self.banner else ""
        return f"{self.port:>5}/{self.status}{svc}{ban} ({self.latency_ms:.1f}ms)"


@dataclass
class ScanReport:
    """Full scan report for a host."""
    host: str
    ports_scanned: int
    open_ports: list[PortResult]
    closed_ports: int
    filtered_ports: int
    start_time: datetime
    end_time: Optional[datetime] = None

    @property
    def duration_ms(self) -> float:
        if self.end_time is None:
            return 0.0
        return (self.end_time - self.start_time).total_seconds() * 1000

    def summary(self) -> str:
        lines = [
            f"Scan report for {self.host}",
            f"  Scanned {self.ports_scanned} ports in {self.duration_ms:.0f}ms",
            f"  Open: {len(self.open_ports)}, Closed: {self.closed_ports}, Filtered: {self.filtered_ports}",
        ]
        if self.open_ports:
            lines.append("  Open ports:")
            for p in self.open_ports:
                lines.append(f"    {p}")
        return "\n".join(lines)


# Well-known services (subset)
WELL_KNOWN_PORTS: dict[int, str] = {
    20: "ftp-data", 21: "ftp", 22: "ssh", 23: "telnet", 25: "smtp",
    53: "dns", 80: "http", 110: "pop3", 111: "rpcbind", 119: "nntp",
    123: "ntp", 135: "msrpc", 139: "netbios-ssn", 143: "imap",
    161: "snmp", 194: "irc", 389: "ldap", 443: "https", 445: "microsoft-ds",
    465: "smtps", 514: "syslog", 587: "submission", 631: "ipp",
    993: "imaps", 995: "pop3s", 1080: "socks", 1433: "mssql",
    1434: "mssql-m", 1521: "oracle", 1723: "pptp", 2049: "nfs",
    3306: "mysql", 3389: "ms-wbt-server", 5432: "postgresql",
    5433: "postgresql-alt", 5672: "amqp", 5900: "vnc", 6379: "redis",
    6667: "irc", 8080: "http-proxy", 8443: "https-alt", 8888: "http-alt",
    9090: "zeus-admin", 9200: "elasticsearch", 27017: "mongodb",
}


def lookup_service(port: int) -> str:
    """Return well-known service name for a port, or empty string."""
    return WELL_KNOWN_PORTS.get(port, "")


class PortScanner:
    """TCP port scanner with configurable timeout and concurrency."""

    def __init__(self, timeout: float = 1.0, max_workers: int = 100) -> None:
        self.timeout = timeout
        self.max_workers = max_workers

    def scan_port(self, host: str, port: int, grab_banner: bool = False) -> PortResult:
        """Scan a single TCP port on a host."""
        import time
        start = time.monotonic()
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(self.timeout)
            result = sock.connect_ex((host, port))
            elapsed = (time.monotonic() - start) * 1000

            if result == 0:
                banner = ""
                if grab_banner:
                    try:
                        sock.settimeout(0.5)
                        sock.sendall(b"\r\n")
                        banner = sock.recv(1024).decode("utf-8", errors="replace").strip()
                    except (socket.timeout, OSError):
                        pass
                sock.close()
                return PortResult(
                    port=port, status="open",
                    service=lookup_service(port),
                    banner=banner, latency_ms=elapsed,
                )
            else:
                sock.close()
                return PortResult(port=port, status="closed", latency_ms=elapsed)
        except socket.timeout:
            elapsed = (time.monotonic() - start) * 1000
            return PortResult(port=port, status="filtered", latency_ms=elapsed)
        except OSError:
            elapsed = (time.monotonic() - start) * 1000
            return PortResult(port=port, status="filtered", latency_ms=elapsed)

    def scan_range(self, host: str, start_port: int = 1, end_port: int = 1024,
                   grab_banner: bool = False) -> ScanReport:
        """Scan a range of ports on a host, concurrently."""
        ports = list(range(start_port, end_port + 1))
        return self._scan_ports(host, ports, grab_banner)

    def scan_ports(self, host: str, ports: list[int],
                   grab_banner: bool = False) -> ScanReport:
        """Scan a specific list of ports on a host, concurrently."""
        return self._scan_ports(host, ports, grab_banner)

    def scan_common(self, host: str, grab_banner: bool = False) -> ScanReport:
        """Scan well-known ports only."""
        ports = sorted(WELL_KNOWN_PORTS.keys())
        return self._scan_ports(host, ports, grab_banner)

    def _scan_ports(self, host: str, ports: list[int],
                    grab_banner: bool) -> ScanReport:
        start_time = datetime.now(timezone.utc)
        results: list[PortResult] = []

        with concurrent.futures.ThreadPoolExecutor(max_workers=self.max_workers) as pool:
            futures = {
                pool.submit(self.scan_port, host, port, grab_banner): port
                for port in ports
            }
            for future in concurrent.futures.as_completed(futures):
                results.append(future.result())

        results.sort(key=lambda r: r.port)
        open_ports = [r for r in results if r.status == "open"]
        closed = sum(1 for r in results if r.status == "closed")
        filtered = sum(1 for r in results if r.status == "filtered")

        return ScanReport(
            host=host,
            ports_scanned=len(ports),
            open_ports=open_ports,
            closed_ports=closed,
            filtered_ports=filtered,
            start_time=start_time,
            end_time=datetime.now(timezone.utc),
        )


def compare_scans(before: ScanReport, after: ScanReport) -> dict:
    """Compare two scan reports for the same host. Returns newly opened/closed ports."""
    before_open = {p.port for p in before.open_ports}
    after_open = {p.port for p in after.open_ports}
    return {
        "newly_opened": sorted(after_open - before_open),
        "newly_closed": sorted(before_open - after_open),
        "still_open": sorted(before_open & after_open),
    }


# --- demo ---
if __name__ == "__main__":
    scanner = PortScanner(timeout=0.5, max_workers=50)
    # Scan localhost for common ports
    report = scanner.scan_common("127.0.0.1")
    print(report.summary())
