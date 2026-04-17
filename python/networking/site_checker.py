"""Site Checker with Time Scheduling — periodic health monitoring of web endpoints."""

from dataclasses import dataclass, field
from datetime import datetime, timezone, timedelta
from typing import Optional, Callable
from enum import Enum
import urllib.request
import urllib.error
import time


class SiteStatus(Enum):
    UP = "up"
    DOWN = "down"
    DEGRADED = "degraded"
    UNKNOWN = "unknown"


@dataclass
class CheckConfig:
    """Configuration for a monitored site."""
    url: str
    name: str = ""
    interval_seconds: int = 60
    timeout_seconds: float = 10.0
    expected_status: int = 200
    expected_content: str = ""
    max_response_ms: float = 5000.0

    def __post_init__(self) -> None:
        if not self.name:
            self.name = self.url


@dataclass
class CheckResultEntry:
    """Result of a single site check."""
    url: str
    status: SiteStatus
    http_status: int = 0
    response_time_ms: float = 0.0
    error: str = ""
    content_matched: bool = True
    checked_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def __repr__(self) -> str:
        return (f"{self.checked_at.strftime('%H:%M:%S')} {self.status.value:>8} "
                f"{self.url} ({self.response_time_ms:.0f}ms) HTTP {self.http_status}")


@dataclass
class SiteHistory:
    """Health history for a single monitored site."""
    config: CheckConfig
    checks: list[CheckResultEntry] = field(default_factory=list)
    consecutive_failures: int = 0
    last_status_change: Optional[datetime] = None

    @property
    def current_status(self) -> SiteStatus:
        return self.checks[-1].status if self.checks else SiteStatus.UNKNOWN

    @property
    def uptime_pct(self) -> float:
        if not self.checks:
            return 0.0
        up = sum(1 for c in self.checks if c.status == SiteStatus.UP)
        return (up / len(self.checks)) * 100

    @property
    def avg_response_ms(self) -> float:
        successful = [c.response_time_ms for c in self.checks if c.status == SiteStatus.UP]
        return sum(successful) / len(successful) if successful else 0.0

    @property
    def max_response_ms(self) -> float:
        successful = [c.response_time_ms for c in self.checks if c.status == SiteStatus.UP]
        return max(successful) if successful else 0.0

    def add_check(self, result: CheckResultEntry) -> Optional[str]:
        """Add a check result. Returns a status change message if the status changed."""
        prev = self.current_status
        self.checks.append(result)
        if result.status in (SiteStatus.DOWN, SiteStatus.DEGRADED):
            self.consecutive_failures += 1
        else:
            self.consecutive_failures = 0
        if prev != result.status and prev != SiteStatus.UNKNOWN:
            self.last_status_change = result.checked_at
            return f"{self.config.name}: {prev.value} -> {result.status.value}"
        return None

    def summary(self) -> str:
        return (f"{self.config.name}: {self.current_status.value} | "
                f"uptime {self.uptime_pct:.1f}% | avg {self.avg_response_ms:.0f}ms | "
                f"{len(self.checks)} checks | {self.consecutive_failures} consecutive failures")


@dataclass
class ScheduleEntry:
    """A scheduled check with next-run tracking."""
    config: CheckConfig
    next_run: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    enabled: bool = True

    def is_due(self, now: Optional[datetime] = None) -> bool:
        now = now or datetime.now(timezone.utc)
        return self.enabled and now >= self.next_run

    def advance(self) -> None:
        self.next_run = datetime.now(timezone.utc) + timedelta(seconds=self.config.interval_seconds)


class SiteChecker:
    """Monitors web endpoints on a schedule with alerting and history."""

    def __init__(self) -> None:
        self.schedules: dict[str, ScheduleEntry] = {}
        self.histories: dict[str, SiteHistory] = {}
        self._alert_callbacks: list[Callable[[str], None]] = []

    def add_site(self, config: CheckConfig) -> None:
        """Add a site to monitor."""
        self.schedules[config.url] = ScheduleEntry(config=config)
        self.histories[config.url] = SiteHistory(config=config)

    def remove_site(self, url: str) -> None:
        """Remove a site from monitoring."""
        self.schedules.pop(url, None)
        self.histories.pop(url, None)

    def on_alert(self, callback: Callable[[str], None]) -> None:
        """Register a callback for status change alerts."""
        self._alert_callbacks.append(callback)

    def check_site(self, config: CheckConfig) -> CheckResultEntry:
        """Perform a single health check on a site."""
        now = datetime.now(timezone.utc)
        start = time.monotonic()
        try:
            req = urllib.request.Request(config.url, headers={"User-Agent": "rosetta-sitechecker/1.0"})
            with urllib.request.urlopen(req, timeout=config.timeout_seconds) as resp:
                elapsed = (time.monotonic() - start) * 1000
                body = resp.read().decode("utf-8", errors="replace")
                http_status = resp.status

                content_ok = True
                if config.expected_content and config.expected_content not in body:
                    content_ok = False

                if http_status != config.expected_status:
                    return CheckResultEntry(url=config.url, status=SiteStatus.DOWN,
                                            http_status=http_status, response_time_ms=elapsed,
                                            error=f"Expected {config.expected_status}, got {http_status}",
                                            checked_at=now)

                if not content_ok:
                    return CheckResultEntry(url=config.url, status=SiteStatus.DEGRADED,
                                            http_status=http_status, response_time_ms=elapsed,
                                            content_matched=False, error="Content mismatch",
                                            checked_at=now)

                if elapsed > config.max_response_ms:
                    return CheckResultEntry(url=config.url, status=SiteStatus.DEGRADED,
                                            http_status=http_status, response_time_ms=elapsed,
                                            error=f"Slow: {elapsed:.0f}ms > {config.max_response_ms:.0f}ms",
                                            checked_at=now)

                return CheckResultEntry(url=config.url, status=SiteStatus.UP,
                                        http_status=http_status, response_time_ms=elapsed,
                                        checked_at=now)

        except urllib.error.HTTPError as e:
            elapsed = (time.monotonic() - start) * 1000
            return CheckResultEntry(url=config.url, status=SiteStatus.DOWN,
                                    http_status=e.code, response_time_ms=elapsed,
                                    error=str(e), checked_at=now)
        except Exception as e:
            elapsed = (time.monotonic() - start) * 1000
            return CheckResultEntry(url=config.url, status=SiteStatus.DOWN,
                                    response_time_ms=elapsed, error=str(e), checked_at=now)

    def check_due(self, now: Optional[datetime] = None) -> list[CheckResultEntry]:
        """Check all sites that are due. Returns list of results."""
        now = now or datetime.now(timezone.utc)
        results = []
        for url, sched in self.schedules.items():
            if sched.is_due(now):
                result = self.check_site(sched.config)
                alert_msg = self.histories[url].add_check(result)
                if alert_msg:
                    for cb in self._alert_callbacks:
                        cb(alert_msg)
                sched.advance()
                results.append(result)
        return results

    def force_check_all(self) -> list[CheckResultEntry]:
        """Check all sites immediately regardless of schedule."""
        results = []
        for url, sched in self.schedules.items():
            result = self.check_site(sched.config)
            alert_msg = self.histories[url].add_check(result)
            if alert_msg:
                for cb in self._alert_callbacks:
                    cb(alert_msg)
            sched.advance()
            results.append(result)
        return results

    def dashboard(self) -> str:
        """Return a text dashboard of all monitored sites."""
        lines = ["Site Monitor Dashboard", "=" * 60]
        for url, history in sorted(self.histories.items()):
            lines.append(history.summary())
        return "\n".join(lines)

    def sites_by_status(self, status: SiteStatus) -> list[str]:
        """Return URLs of sites with the given status."""
        return [url for url, h in self.histories.items() if h.current_status == status]


def simulate_checks(checker: SiteChecker, results_map: dict[str, list[CheckResultEntry]]) -> list[str]:
    """Simulate a series of checks with predetermined results. Returns alert messages."""
    alerts: list[str] = []
    for url, results in results_map.items():
        if url in checker.histories:
            for result in results:
                msg = checker.histories[url].add_check(result)
                if msg:
                    alerts.append(msg)
    return alerts


# --- demo ---
if __name__ == "__main__":
    checker = SiteChecker()
    checker.add_site(CheckConfig("https://eckenrodemuziekopname.com", "EM Website", interval_seconds=300))
    checker.add_site(CheckConfig("https://wiki.eckenrodemuziekopname.com", "EM Wiki", interval_seconds=300))

    alerts: list[str] = []
    checker.on_alert(lambda msg: alerts.append(msg))

    results = checker.force_check_all()
    for r in results:
        print(r)
    print()
    print(checker.dashboard())
    if alerts:
        print(f"\nAlerts: {alerts}")
