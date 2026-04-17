// P2P File Sharing App — decentralized file exchange between peers.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn sha256_simple(data: &[u8]) -> String {
    // Minimal SHA-256 for chunk verification (no external crate).
    // Uses a basic implementation of the NIST FIPS 180-4 algorithm.
    let k: [u32; 64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,
        0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19,
    ];

    // Padding
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    // Process blocks
    for block in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([block[i*4], block[i*4+1], block[i*4+2], block[i*4+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut hh) = (h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1); d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|v| format!("{:08x}", v)).collect()
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    pub index: usize,
    pub data: Vec<u8>,
    pub sha256: String,
}

impl FileChunk {
    pub fn from_data(index: usize, data: Vec<u8>) -> Self {
        let sha256 = sha256_simple(&data);
        FileChunk { index, data, sha256 }
    }

    pub fn verify(&self) -> bool {
        sha256_simple(&self.data) == self.sha256
    }
}

#[derive(Debug, Clone)]
pub struct SharedFile {
    pub name: String,
    pub size: usize,
    pub sha256: String,
    pub chunk_size: usize,
    pub chunk_count: usize,
}

impl SharedFile {
    pub fn new(name: &str, size: usize, sha256: &str, chunk_size: usize) -> Self {
        let chunk_count = if size == 0 { 0 } else { (size + chunk_size - 1) / chunk_size };
        SharedFile {
            name: name.to_string(),
            size,
            sha256: sha256.to_string(),
            chunk_size,
            chunk_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub port: u16,
    pub last_seen: u64,
    pub shared_files: Vec<SharedFile>,
}

impl PeerInfo {
    pub fn new(peer_id: &str, address: &str, port: u16) -> Self {
        PeerInfo {
            peer_id: peer_id.to_string(),
            address: address.to_string(),
            port,
            last_seen: now_secs(),
            shared_files: Vec::new(),
        }
    }

    pub fn is_stale(&self, timeout_secs: u64) -> bool {
        now_secs().saturating_sub(self.last_seen) > timeout_secs
    }
}

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub request_id: String,
    pub filename: String,
    pub requester_id: String,
    pub provider_id: String,
    pub status: TransferStatus,
    pub chunks_received: usize,
    pub total_chunks: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

impl TransferRequest {
    pub fn progress(&self) -> f64 {
        if self.total_chunks == 0 { 0.0 }
        else { self.chunks_received as f64 / self.total_chunks as f64 }
    }
}

#[derive(Debug)]
pub struct PeerNode {
    pub peer_id: String,
    pub address: String,
    pub port: u16,
    peers: HashMap<String, PeerInfo>,
    local_files: HashMap<String, Vec<u8>>,
    shared_catalog: HashMap<String, SharedFile>,
    transfers: HashMap<String, TransferRequest>,
    chunks_store: HashMap<String, Vec<Option<FileChunk>>>,
    next_request_id: usize,
}

impl PeerNode {
    pub fn new(peer_id: &str, address: &str, port: u16) -> Self {
        PeerNode {
            peer_id: peer_id.to_string(),
            address: address.to_string(),
            port,
            peers: HashMap::new(),
            local_files: HashMap::new(),
            shared_catalog: HashMap::new(),
            transfers: HashMap::new(),
            chunks_store: HashMap::new(),
            next_request_id: 0,
        }
    }

    // --- Local file management ---

    pub fn add_file(&mut self, name: &str, data: Vec<u8>, chunk_size: usize) -> SharedFile {
        let sha256 = sha256_simple(&data);
        let size = data.len();
        self.local_files.insert(name.to_string(), data);
        let sf = SharedFile::new(name, size, &sha256, chunk_size);
        self.shared_catalog.insert(name.to_string(), sf.clone());
        sf
    }

    pub fn remove_file(&mut self, name: &str) {
        self.local_files.remove(name);
        self.shared_catalog.remove(name);
    }

    pub fn get_chunk(&self, filename: &str, chunk_index: usize) -> Option<FileChunk> {
        let data = self.local_files.get(filename)?;
        let sf = self.shared_catalog.get(filename)?;
        let start = chunk_index * sf.chunk_size;
        if start >= data.len() { return None; }
        let end = (start + sf.chunk_size).min(data.len());
        Some(FileChunk::from_data(chunk_index, data[start..end].to_vec()))
    }

    // --- Peer discovery ---

    pub fn discover_peer(&mut self, peer_id: &str, address: &str, port: u16) -> &PeerInfo {
        self.peers.entry(peer_id.to_string())
            .or_insert_with(|| PeerInfo::new(peer_id, address, port));
        &self.peers[peer_id]
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }

    pub fn prune_stale_peers(&mut self, timeout_secs: u64) -> Vec<String> {
        let stale: Vec<String> = self.peers.iter()
            .filter(|(_, p)| p.is_stale(timeout_secs))
            .map(|(id, _)| id.clone())
            .collect();
        for id in &stale { self.peers.remove(id); }
        stale
    }

    pub fn heartbeat(&mut self, peer_id: &str) {
        if let Some(p) = self.peers.get_mut(peer_id) {
            p.last_seen = now_secs();
        }
    }

    // --- Catalog exchange ---

    pub fn get_catalog(&self) -> Vec<SharedFile> {
        self.shared_catalog.values().cloned().collect()
    }

    pub fn receive_catalog(&mut self, peer_id: &str, catalog: Vec<SharedFile>) {
        if let Some(p) = self.peers.get_mut(peer_id) {
            p.shared_files = catalog;
            p.last_seen = now_secs();
        }
    }

    pub fn search_network(&self, query: &str) -> Vec<(&str, &SharedFile)> {
        let q = query.to_lowercase();
        let mut results = Vec::new();
        for (pid, peer) in &self.peers {
            for sf in &peer.shared_files {
                if sf.name.to_lowercase().contains(&q) {
                    results.push((pid.as_str(), sf));
                }
            }
        }
        results
    }

    // --- File transfer ---

    pub fn request_file(&mut self, provider_id: &str, filename: &str) -> Option<String> {
        let peer = self.peers.get(provider_id)?;
        let sf = peer.shared_files.iter().find(|f| f.name == filename)?;
        self.next_request_id += 1;
        let req_id = format!("xfer-{}", self.next_request_id);
        let total = sf.chunk_count;
        let req = TransferRequest {
            request_id: req_id.clone(),
            filename: filename.to_string(),
            requester_id: self.peer_id.clone(),
            provider_id: provider_id.to_string(),
            status: TransferStatus::Active,
            chunks_received: 0,
            total_chunks: total,
        };
        self.transfers.insert(req_id.clone(), req);
        self.chunks_store.insert(req_id.clone(), vec![None; total]);
        Some(req_id)
    }

    pub fn receive_chunk(&mut self, request_id: &str, chunk: FileChunk) -> bool {
        let req = match self.transfers.get_mut(request_id) {
            Some(r) if r.status == TransferStatus::Active => r,
            _ => return false,
        };
        if !chunk.verify() { return false; }
        let idx = chunk.index;
        let chunks = match self.chunks_store.get_mut(request_id) {
            Some(c) if idx < c.len() => c,
            _ => return false,
        };
        chunks[idx] = Some(chunk);
        req.chunks_received = chunks.iter().filter(|c| c.is_some()).count();
        if req.chunks_received == req.total_chunks {
            req.status = TransferStatus::Completed;
            let filename = req.filename.clone();
            let assembled: Vec<u8> = chunks.iter()
                .filter_map(|c| c.as_ref().map(|c| c.data.clone()))
                .flatten()
                .collect();
            self.local_files.insert(filename, assembled);
        }
        true
    }

    pub fn get_missing_chunks(&self, request_id: &str) -> Vec<usize> {
        match self.chunks_store.get(request_id) {
            Some(chunks) => chunks.iter().enumerate()
                .filter(|(_, c)| c.is_none())
                .map(|(i, _)| i)
                .collect(),
            None => vec![],
        }
    }

    pub fn get_transfer(&self, request_id: &str) -> Option<&TransferRequest> {
        self.transfers.get(request_id)
    }

    pub fn has_file(&self, name: &str) -> bool {
        self.local_files.contains_key(name)
    }
}

pub fn simulate_transfer(provider: &PeerNode, requester: &mut PeerNode, filename: &str) -> Option<String> {
    let req_id = requester.request_file(&provider.peer_id, filename)?;
    let req = requester.transfers.get(&req_id)?;
    let total = req.total_chunks;
    for idx in 0..total {
        if let Some(chunk) = provider.get_chunk(filename, idx) {
            requester.receive_chunk(&req_id, chunk);
        }
    }
    Some(req_id)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256_simple(b"hello");
        assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_chunk_verify() {
        let chunk = FileChunk::from_data(0, b"test data".to_vec());
        assert!(chunk.verify());
        let bad = FileChunk { index: 0, data: b"tampered".to_vec(), sha256: chunk.sha256.clone() };
        assert!(!bad.verify());
    }

    #[test]
    fn test_peer_discovery() {
        let mut node = PeerNode::new("alice", "127.0.0.1", 8001);
        node.discover_peer("bob", "127.0.0.1", 8002);
        assert_eq!(node.peers.len(), 1);
        node.remove_peer("bob");
        assert_eq!(node.peers.len(), 0);
    }

    #[test]
    fn test_catalog_exchange() {
        let mut alice = PeerNode::new("alice", "127.0.0.1", 8001);
        let mut bob = PeerNode::new("bob", "127.0.0.1", 8002);
        alice.discover_peer("bob", "127.0.0.1", 8002);
        bob.discover_peer("alice", "127.0.0.1", 8001);

        alice.add_file("test.txt", b"hello world".to_vec(), 4);
        bob.receive_catalog("alice", alice.get_catalog());

        let results = bob.search_network("test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.name, "test.txt");
    }

    #[test]
    fn test_full_transfer() {
        let mut alice = PeerNode::new("alice", "127.0.0.1", 8001);
        let mut bob = PeerNode::new("bob", "127.0.0.1", 8002);
        alice.discover_peer("bob", "127.0.0.1", 8002);
        bob.discover_peer("alice", "127.0.0.1", 8001);

        let content = b"The Rosetta Stone is not a learning exercise.".to_vec();
        alice.add_file("rosetta.txt", content.clone(), 16);
        bob.receive_catalog("alice", alice.get_catalog());

        let req_id = simulate_transfer(&alice, &mut bob, "rosetta.txt").unwrap();
        let req = bob.get_transfer(&req_id).unwrap();
        assert_eq!(req.status, TransferStatus::Completed);
        assert!((req.progress() - 1.0).abs() < 0.001);
        assert!(bob.has_file("rosetta.txt"));
        assert_eq!(bob.local_files["rosetta.txt"], content);
    }

    #[test]
    fn test_shared_file_chunks() {
        let sf = SharedFile::new("f.txt", 100, "abc", 32);
        assert_eq!(sf.chunk_count, 4); // ceil(100/32)
    }
}
