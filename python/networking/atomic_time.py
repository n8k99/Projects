"""Get Atomic Time from Internet Clock -- time synchronization protocol."""

import time
from dataclasses import dataclass, field


@dataclass
class TimeServer:
    """Authoritative time source. Wraps system clock with optional fixed offset for testing."""

    _offset_seconds: float = 0.0

    def get_time(self) -> float:
        """Return authoritative timestamp (epoch seconds)."""
        return time.time() + self._offset_seconds

    @classmethod
    def with_offset(cls, offset: float) -> "TimeServer":
        """Create a server whose clock is shifted by offset seconds from system time."""
        return cls(_offset_seconds=offset)


@dataclass
class TimeClient:
    """NTP-style client that queries a TimeServer and tracks synchronization state."""

    server: TimeServer
    _local_offset: float = 0.0  # deliberate drift from system clock
    _sync_offset: float = 0.0   # correction learned from server
    _synced: bool = False
    _samples: list = field(default_factory=list)

    @classmethod
    def with_drift(cls, server: TimeServer, drift: float) -> "TimeClient":
        """Create a client whose local clock is off by drift seconds."""
        return cls(server=server, _local_offset=drift)

    def local_time(self) -> float:
        """Return the client's uncorrected local time."""
        return time.time() + self._local_offset

    def query_time(self) -> float:
        """Query the server and return its authoritative timestamp."""
        t_send = self.local_time()
        server_time = self.server.get_time()
        t_recv = self.local_time()
        round_trip = t_recv - t_send
        # Estimate server time at midpoint of round trip
        estimated_server_now = server_time + round_trip / 2.0
        self._samples.append(estimated_server_now - t_recv)
        return server_time

    def get_offset(self) -> float:
        """Return measured offset between local clock and server clock.

        Positive means local clock is behind server (needs to advance).
        Averages all collected samples for accuracy.
        """
        if not self._samples:
            # Take a sample first
            self.query_time()
        return sum(self._samples) / len(self._samples)

    def sync(self) -> None:
        """Adjust local time estimate by applying the measured offset."""
        self._sync_offset = self.get_offset()
        self._synced = True

    def get_synced_time(self) -> float:
        """Return local time corrected by the synchronization offset."""
        if not self._synced:
            return self.local_time()
        return self.local_time() + self._sync_offset

    @property
    def is_synced(self) -> bool:
        return self._synced


def format_time(epoch: float) -> str:
    """Format epoch timestamp as human-readable local time."""
    return time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(epoch)) + f".{int(epoch % 1 * 1000):03d}"


if __name__ == "__main__":
    print("=== G034: Atomic Time Synchronization ===\n")

    # Create an authoritative time server (using system clock)
    server = TimeServer()

    # Create a client whose local clock is 3.5 seconds behind
    drift = -3.5
    client = TimeClient.with_drift(server, drift=drift)

    print(f"Server time:       {format_time(server.get_time())}")
    print(f"Client local time: {format_time(client.local_time())}")
    print(f"Deliberate drift:  {drift:+.1f}s\n")

    # Query server multiple times to build sample set
    print("Querying server (3 samples)...")
    for i in range(3):
        ts = client.query_time()
        print(f"  Sample {i+1}: server reports {format_time(ts)}")

    offset = client.get_offset()
    print(f"\nMeasured offset: {offset:+.4f}s")
    print(f"(Expected ~{-drift:+.1f}s to correct the {drift:+.1f}s drift)\n")

    # Sync
    client.sync()
    print(f"After sync:")
    print(f"  Server time:  {format_time(server.get_time())}")
    print(f"  Synced time:  {format_time(client.get_synced_time())}")
    print(f"  Local time:   {format_time(client.local_time())}")

    error = abs(client.get_synced_time() - server.get_time())
    print(f"\n  Sync error: {error*1000:.1f}ms")
    print(f"  Synced: {client.is_synced}")
