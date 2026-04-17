"""P2P File Sharing App — decentralized file exchange between peers."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional
import hashlib
import math


@dataclass
class FileChunk:
    """A chunk of a file, identified by index and hash."""
    index: int
    data: bytes
    sha256: str

    @staticmethod
    def from_data(index: int, data: bytes) -> "FileChunk":
        return FileChunk(
            index=index,
            data=data,
            sha256=hashlib.sha256(data).hexdigest(),
        )

    def verify(self) -> bool:
        return hashlib.sha256(self.data).hexdigest() == self.sha256


@dataclass
class SharedFile:
    """Metadata for a file shared by a peer."""
    name: str
    size: int
    sha256: str
    chunk_size: int = 4096
    chunk_count: int = 0

    def __post_init__(self) -> None:
        if self.chunk_count == 0:
            self.chunk_count = math.ceil(self.size / self.chunk_size) if self.size > 0 else 0


@dataclass
class PeerInfo:
    """Information about a known peer."""
    peer_id: str
    address: str
    port: int
    last_seen: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    shared_files: list[SharedFile] = field(default_factory=list)

    def is_stale(self, timeout_seconds: int = 300) -> bool:
        age = (datetime.now(timezone.utc) - self.last_seen).total_seconds()
        return age > timeout_seconds


@dataclass
class TransferRequest:
    """A request to transfer a file between peers."""
    request_id: str
    filename: str
    requester_id: str
    provider_id: str
    status: str = "pending"  # pending, active, completed, failed
    chunks_received: int = 0
    total_chunks: int = 0
    started: Optional[datetime] = None
    completed: Optional[datetime] = None

    @property
    def progress(self) -> float:
        if self.total_chunks == 0:
            return 0.0
        return self.chunks_received / self.total_chunks

    @property
    def progress_pct(self) -> str:
        return f"{self.progress * 100:.1f}%"


class PeerNode:
    """A peer in the P2P network. Manages local files, known peers,
    discovery, catalog exchange, and chunked file transfers."""

    def __init__(self, peer_id: str, address: str = "127.0.0.1", port: int = 0) -> None:
        self.peer_id = peer_id
        self.address = address
        self.port = port
        self.peers: dict[str, PeerInfo] = {}
        self.local_files: dict[str, bytes] = {}
        self.shared_catalog: dict[str, SharedFile] = {}
        self.transfers: dict[str, TransferRequest] = {}
        self._chunks_store: dict[str, list[Optional[FileChunk]]] = {}
        self._next_request_id = 0

    # --- Local file management ---

    def add_file(self, name: str, data: bytes, chunk_size: int = 4096) -> SharedFile:
        """Add a file to the local store and share it on the network."""
        self.local_files[name] = data
        sf = SharedFile(
            name=name,
            size=len(data),
            sha256=hashlib.sha256(data).hexdigest(),
            chunk_size=chunk_size,
        )
        self.shared_catalog[name] = sf
        return sf

    def remove_file(self, name: str) -> None:
        """Stop sharing a file."""
        self.local_files.pop(name, None)
        self.shared_catalog.pop(name, None)

    def get_chunk(self, filename: str, chunk_index: int) -> Optional[FileChunk]:
        """Retrieve a specific chunk of a local file (called by requesting peers)."""
        data = self.local_files.get(filename)
        if data is None:
            return None
        sf = self.shared_catalog.get(filename)
        if sf is None:
            return None
        start = chunk_index * sf.chunk_size
        end = min(start + sf.chunk_size, len(data))
        if start >= len(data):
            return None
        return FileChunk.from_data(chunk_index, data[start:end])

    # --- Peer discovery ---

    def discover_peer(self, peer_id: str, address: str, port: int) -> PeerInfo:
        """Register a known peer (from bootstrap, broadcast, or introduction)."""
        info = PeerInfo(peer_id=peer_id, address=address, port=port)
        self.peers[peer_id] = info
        return info

    def remove_peer(self, peer_id: str) -> None:
        """Remove a peer from the known list."""
        self.peers.pop(peer_id, None)

    def prune_stale_peers(self, timeout_seconds: int = 300) -> list[str]:
        """Remove peers not seen within the timeout. Returns removed IDs."""
        stale = [pid for pid, p in self.peers.items() if p.is_stale(timeout_seconds)]
        for pid in stale:
            del self.peers[pid]
        return stale

    def heartbeat(self, peer_id: str) -> None:
        """Update last-seen timestamp for a peer."""
        if peer_id in self.peers:
            self.peers[peer_id].last_seen = datetime.now(timezone.utc)

    # --- Catalog exchange ---

    def get_catalog(self) -> list[SharedFile]:
        """Return this peer's shared file catalog (sent to other peers)."""
        return list(self.shared_catalog.values())

    def receive_catalog(self, peer_id: str, catalog: list[SharedFile]) -> None:
        """Receive and store another peer's catalog."""
        if peer_id in self.peers:
            self.peers[peer_id].shared_files = catalog
            self.peers[peer_id].last_seen = datetime.now(timezone.utc)

    def search_network(self, query: str) -> list[tuple[str, SharedFile]]:
        """Search all known peers' catalogs for files matching the query."""
        results = []
        q = query.lower()
        for peer_id, peer in self.peers.items():
            for sf in peer.shared_files:
                if q in sf.name.lower():
                    results.append((peer_id, sf))
        return results

    # --- File transfer ---

    def request_file(self, provider_id: str, filename: str) -> Optional[TransferRequest]:
        """Initiate a file transfer request from a peer."""
        peer = self.peers.get(provider_id)
        if peer is None:
            return None
        sf = next((f for f in peer.shared_files if f.name == filename), None)
        if sf is None:
            return None

        self._next_request_id += 1
        req_id = f"xfer-{self._next_request_id}"
        req = TransferRequest(
            request_id=req_id,
            filename=filename,
            requester_id=self.peer_id,
            provider_id=provider_id,
            total_chunks=sf.chunk_count,
            started=datetime.now(timezone.utc),
        )
        req.status = "active"
        self.transfers[req_id] = req
        self._chunks_store[req_id] = [None] * sf.chunk_count
        return req

    def receive_chunk(self, request_id: str, chunk: FileChunk) -> bool:
        """Receive a chunk for an active transfer. Returns True if chunk is valid."""
        req = self.transfers.get(request_id)
        if req is None or req.status != "active":
            return False
        if not chunk.verify():
            return False
        chunks = self._chunks_store.get(request_id)
        if chunks is None or chunk.index >= len(chunks):
            return False
        chunks[chunk.index] = chunk
        req.chunks_received = sum(1 for c in chunks if c is not None)
        if req.chunks_received == req.total_chunks:
            req.status = "completed"
            req.completed = datetime.now(timezone.utc)
            # Reassemble file
            assembled = b"".join(c.data for c in chunks if c is not None)
            self.local_files[req.filename] = assembled
        return True

    def get_missing_chunks(self, request_id: str) -> list[int]:
        """Return indices of chunks not yet received for a transfer."""
        chunks = self._chunks_store.get(request_id)
        if chunks is None:
            return []
        return [i for i, c in enumerate(chunks) if c is None]

    # --- Network-wide stats ---

    def network_stats(self) -> dict:
        """Summary of this peer's view of the network."""
        total_files = sum(len(p.shared_files) for p in self.peers.values())
        total_bytes = sum(
            sf.size for p in self.peers.values() for sf in p.shared_files
        )
        return {
            "peer_id": self.peer_id,
            "known_peers": len(self.peers),
            "local_files": len(self.local_files),
            "network_files": total_files,
            "network_bytes": total_bytes,
            "active_transfers": sum(
                1 for t in self.transfers.values() if t.status == "active"
            ),
        }


def simulate_transfer(provider: PeerNode, requester: PeerNode, filename: str) -> Optional[TransferRequest]:
    """Simulate a complete file transfer between two peers in-process."""
    req = requester.request_file(provider.peer_id, filename)
    if req is None:
        return None
    for idx in range(req.total_chunks):
        chunk = provider.get_chunk(filename, idx)
        if chunk is not None:
            requester.receive_chunk(req.request_id, chunk)
    return req


# --- demo ---
if __name__ == "__main__":
    # Create two peers
    alice = PeerNode("alice", "192.168.1.10", 8001)
    bob = PeerNode("bob", "192.168.1.11", 8002)

    # Mutual discovery
    alice.discover_peer("bob", "192.168.1.11", 8002)
    bob.discover_peer("alice", "192.168.1.10", 8001)

    # Alice shares a file
    content = b"The Rosetta Stone is not a learning exercise." * 100
    alice.add_file("rosetta.txt", content)

    # Exchange catalogs
    bob.receive_catalog("alice", alice.get_catalog())

    # Bob searches and finds it
    results = bob.search_network("rosetta")
    print(f"Search results: {[(pid, sf.name, sf.size) for pid, sf in results]}")

    # Bob requests the file
    req = simulate_transfer(alice, bob, "rosetta.txt")
    if req:
        print(f"Transfer {req.request_id}: {req.status} ({req.progress_pct})")
        print(f"Bob now has {len(bob.local_files)} local file(s)")
        print(f"File integrity: {bob.local_files['rosetta.txt'] == content}")
