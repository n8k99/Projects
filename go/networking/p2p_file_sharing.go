// P2P File Sharing App — decentralized file exchange between peers.
package networking

import (
	"crypto/sha256"
	"fmt"
	"strings"
	"sync"
	"time"
)

// FileChunk is a chunk of a file with integrity verification.
type FileChunk struct {
	Index  int
	Data   []byte
	Sha256 string
}

// NewFileChunk creates a chunk with a computed hash.
func NewFileChunk(index int, data []byte) FileChunk {
	return FileChunk{
		Index:  index,
		Data:   append([]byte(nil), data...),
		Sha256: fmt.Sprintf("%x", sha256.Sum256(data)),
	}
}

// Verify checks that the chunk data matches its hash.
func (c FileChunk) Verify() bool {
	return fmt.Sprintf("%x", sha256.Sum256(c.Data)) == c.Sha256
}

// SharedFile describes a file available from a peer.
type SharedFile struct {
	Name       string
	Size       int
	Sha256     string
	ChunkSize  int
	ChunkCount int
}

// NewSharedFile creates file metadata with computed chunk count.
func NewSharedFile(name string, size int, sha string, chunkSize int) SharedFile {
	count := 0
	if size > 0 {
		count = (size + chunkSize - 1) / chunkSize
	}
	return SharedFile{Name: name, Size: size, Sha256: sha, ChunkSize: chunkSize, ChunkCount: count}
}

// P2PPeerInfo describes a known peer.
type P2PPeerInfo struct {
	PeerID      string
	Address     string
	Port        int
	LastSeen    time.Time
	SharedFiles []SharedFile
}

// IsStale returns true if the peer hasn't been seen within timeout.
func (p *P2PPeerInfo) IsStale(timeout time.Duration) bool {
	return time.Since(p.LastSeen) > timeout
}

// TransferStatus tracks the state of a file transfer.
type P2PTransferStatus int

const (
	P2PTransferPending P2PTransferStatus = iota
	P2PTransferActive
	P2PTransferCompleted
	P2PTransferFailed
)

// P2PTransferRequest tracks a file transfer between peers.
type P2PTransferRequest struct {
	RequestID      string
	Filename       string
	RequesterID    string
	ProviderID     string
	Status         P2PTransferStatus
	ChunksReceived int
	TotalChunks    int
}

// Progress returns transfer completion as a float [0, 1].
func (t *P2PTransferRequest) Progress() float64 {
	if t.TotalChunks == 0 {
		return 0
	}
	return float64(t.ChunksReceived) / float64(t.TotalChunks)
}

// PeerNode is a peer in the P2P network.
type PeerNode struct {
	mu            sync.RWMutex
	PeerID        string
	Address       string
	Port          int
	peers         map[string]*P2PPeerInfo
	localFiles    map[string][]byte
	sharedCatalog map[string]SharedFile
	transfers     map[string]*P2PTransferRequest
	chunksStore   map[string][]*FileChunk
	nextReqID     int
}

// NewPeerNode creates a new peer.
func NewPeerNode(peerID, address string, port int) *PeerNode {
	return &PeerNode{
		PeerID:        peerID,
		Address:       address,
		Port:          port,
		peers:         make(map[string]*P2PPeerInfo),
		localFiles:    make(map[string][]byte),
		sharedCatalog: make(map[string]SharedFile),
		transfers:     make(map[string]*P2PTransferRequest),
		chunksStore:   make(map[string][]*FileChunk),
	}
}

// AddFile adds a file to the local store and shares it.
func (n *PeerNode) AddFile(name string, data []byte, chunkSize int) SharedFile {
	n.mu.Lock()
	defer n.mu.Unlock()
	hash := fmt.Sprintf("%x", sha256.Sum256(data))
	cp := make([]byte, len(data))
	copy(cp, data)
	n.localFiles[name] = cp
	sf := NewSharedFile(name, len(data), hash, chunkSize)
	n.sharedCatalog[name] = sf
	return sf
}

// RemoveFile stops sharing a file.
func (n *PeerNode) RemoveFile(name string) {
	n.mu.Lock()
	defer n.mu.Unlock()
	delete(n.localFiles, name)
	delete(n.sharedCatalog, name)
}

// GetChunk returns a specific chunk of a local file.
func (n *PeerNode) GetChunk(filename string, index int) (*FileChunk, bool) {
	n.mu.RLock()
	defer n.mu.RUnlock()
	data, ok := n.localFiles[filename]
	if !ok {
		return nil, false
	}
	sf, ok := n.sharedCatalog[filename]
	if !ok {
		return nil, false
	}
	start := index * sf.ChunkSize
	if start >= len(data) {
		return nil, false
	}
	end := start + sf.ChunkSize
	if end > len(data) {
		end = len(data)
	}
	chunk := NewFileChunk(index, data[start:end])
	return &chunk, true
}

// DiscoverPeer registers a known peer.
func (n *PeerNode) DiscoverPeer(peerID, address string, port int) {
	n.mu.Lock()
	defer n.mu.Unlock()
	n.peers[peerID] = &P2PPeerInfo{
		PeerID:   peerID,
		Address:  address,
		Port:     port,
		LastSeen: time.Now().UTC(),
	}
}

// RemovePeer removes a peer.
func (n *PeerNode) RemovePeer(peerID string) {
	n.mu.Lock()
	defer n.mu.Unlock()
	delete(n.peers, peerID)
}

// PruneStalePeers removes peers not seen within timeout.
func (n *PeerNode) PruneStalePeers(timeout time.Duration) []string {
	n.mu.Lock()
	defer n.mu.Unlock()
	var stale []string
	for id, p := range n.peers {
		if p.IsStale(timeout) {
			stale = append(stale, id)
		}
	}
	for _, id := range stale {
		delete(n.peers, id)
	}
	return stale
}

// Heartbeat updates last-seen for a peer.
func (n *PeerNode) Heartbeat(peerID string) {
	n.mu.Lock()
	defer n.mu.Unlock()
	if p, ok := n.peers[peerID]; ok {
		p.LastSeen = time.Now().UTC()
	}
}

// GetCatalog returns this peer's shared files.
func (n *PeerNode) GetCatalog() []SharedFile {
	n.mu.RLock()
	defer n.mu.RUnlock()
	catalog := make([]SharedFile, 0, len(n.sharedCatalog))
	for _, sf := range n.sharedCatalog {
		catalog = append(catalog, sf)
	}
	return catalog
}

// ReceiveCatalog stores another peer's catalog.
func (n *PeerNode) ReceiveCatalog(peerID string, catalog []SharedFile) {
	n.mu.Lock()
	defer n.mu.Unlock()
	if p, ok := n.peers[peerID]; ok {
		p.SharedFiles = catalog
		p.LastSeen = time.Now().UTC()
	}
}

// SearchNetwork finds files matching query across all known peers.
func (n *PeerNode) SearchNetwork(query string) []struct {
	PeerID string
	File   SharedFile
} {
	n.mu.RLock()
	defer n.mu.RUnlock()
	q := strings.ToLower(query)
	var results []struct {
		PeerID string
		File   SharedFile
	}
	for pid, peer := range n.peers {
		for _, sf := range peer.SharedFiles {
			if strings.Contains(strings.ToLower(sf.Name), q) {
				results = append(results, struct {
					PeerID string
					File   SharedFile
				}{pid, sf})
			}
		}
	}
	return results
}

// RequestFile initiates a transfer request. Returns the request ID.
func (n *PeerNode) RequestFile(providerID, filename string) (string, error) {
	n.mu.Lock()
	defer n.mu.Unlock()
	peer, ok := n.peers[providerID]
	if !ok {
		return "", fmt.Errorf("unknown peer: %s", providerID)
	}
	var sf *SharedFile
	for i := range peer.SharedFiles {
		if peer.SharedFiles[i].Name == filename {
			sf = &peer.SharedFiles[i]
			break
		}
	}
	if sf == nil {
		return "", fmt.Errorf("file %q not in peer %s catalog", filename, providerID)
	}
	n.nextReqID++
	reqID := fmt.Sprintf("xfer-%d", n.nextReqID)
	n.transfers[reqID] = &P2PTransferRequest{
		RequestID:   reqID,
		Filename:    filename,
		RequesterID: n.PeerID,
		ProviderID:  providerID,
		Status:      P2PTransferActive,
		TotalChunks: sf.ChunkCount,
	}
	chunks := make([]*FileChunk, sf.ChunkCount)
	n.chunksStore[reqID] = chunks
	return reqID, nil
}

// ReceiveChunk accepts a chunk for an active transfer.
func (n *PeerNode) ReceiveChunk(requestID string, chunk FileChunk) bool {
	n.mu.Lock()
	defer n.mu.Unlock()
	req, ok := n.transfers[requestID]
	if !ok || req.Status != P2PTransferActive {
		return false
	}
	if !chunk.Verify() {
		return false
	}
	chunks := n.chunksStore[requestID]
	if chunk.Index >= len(chunks) {
		return false
	}
	chunks[chunk.Index] = &chunk
	received := 0
	for _, c := range chunks {
		if c != nil {
			received++
		}
	}
	req.ChunksReceived = received
	if received == req.TotalChunks {
		req.Status = P2PTransferCompleted
		var assembled []byte
		for _, c := range chunks {
			if c != nil {
				assembled = append(assembled, c.Data...)
			}
		}
		n.localFiles[req.Filename] = assembled
	}
	return true
}

// GetTransfer returns the status of a transfer.
func (n *PeerNode) GetTransfer(requestID string) (*P2PTransferRequest, bool) {
	n.mu.RLock()
	defer n.mu.RUnlock()
	r, ok := n.transfers[requestID]
	return r, ok
}

// HasFile checks if a file exists locally.
func (n *PeerNode) HasFile(name string) bool {
	n.mu.RLock()
	defer n.mu.RUnlock()
	_, ok := n.localFiles[name]
	return ok
}

// SimulateP2PTransfer runs a complete transfer between two peers in-process.
func SimulateP2PTransfer(provider, requester *PeerNode, filename string) (string, error) {
	reqID, err := requester.RequestFile(provider.PeerID, filename)
	if err != nil {
		return "", err
	}
	req, _ := requester.GetTransfer(reqID)
	for idx := 0; idx < req.TotalChunks; idx++ {
		chunk, ok := provider.GetChunk(filename, idx)
		if ok {
			requester.ReceiveChunk(reqID, *chunk)
		}
	}
	return reqID, nil
}
