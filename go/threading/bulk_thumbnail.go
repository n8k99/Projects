// Bulk Thumbnail Creator — CPU-bound fan-out with content-addressed dedup.
//
// G066's pattern had isolated per-job results. G068 has a shared INDEX as
// the output: the result IS the aggregation. Workers compute a content hash
// and check the index before running the expensive thumbnailer — identical
// content processed twice produces one thumbnail; the second request
// records the existing result rather than recomputing.
//
// Reservation pattern: under the index lock, check-and-claim with a
// placeholder; release the lock; run the thumbnailer unlocked; reacquire
// and install the real value. Concurrent workers with the same hash see
// the placeholder and skip generation. The critical section stays tiny;
// expensive work runs in parallel.
package threading

import "sync"

// ThumbnailRequest is one image to be thumbnailed.
type ThumbnailRequest struct {
	SourceID string
	Content  []byte
}

// ThumbnailIndexSnapshot is the aggregated output of a bulk run.
type ThumbnailIndexSnapshot struct {
	Thumbnails    map[uint64]string // content hash -> thumbnail token
	Mappings      map[string]uint64 // source_id -> content hash
	Generated     int
	Deduplicated  int
}

// BulkThumbnailCreator owns a queue and the shared index it produces.
type BulkThumbnailCreator struct {
	workerCount int
	queueMu     sync.Mutex
	queue       []ThumbnailRequest
	idxMu       sync.Mutex
	thumbnails  map[uint64]string
	mappings    map[string]uint64
	generated   int
	dedup       int
}

// NewBulkThumbnailCreator returns an empty creator with `workerCount` workers.
func NewBulkThumbnailCreator(workerCount int) *BulkThumbnailCreator {
	if workerCount < 1 {
		panic("workerCount must be >= 1")
	}
	return &BulkThumbnailCreator{
		workerCount: workerCount,
		thumbnails:  make(map[uint64]string),
		mappings:    make(map[string]uint64),
	}
}

// EnqueueThumb adds a request to the queue.
func (b *BulkThumbnailCreator) EnqueueThumb(r ThumbnailRequest) {
	b.queueMu.Lock()
	defer b.queueMu.Unlock()
	b.queue = append(b.queue, r)
}

func (b *BulkThumbnailCreator) pop() (ThumbnailRequest, bool) {
	b.queueMu.Lock()
	defer b.queueMu.Unlock()
	if len(b.queue) == 0 {
		return ThumbnailRequest{}, false
	}
	r := b.queue[0]
	b.queue = b.queue[1:]
	return r, true
}

// claim returns true if this worker must generate the thumbnail for hash.
// Installs a placeholder under the index lock so concurrent workers with
// the same hash see it as present.
func (b *BulkThumbnailCreator) claim(hash uint64) bool {
	b.idxMu.Lock()
	defer b.idxMu.Unlock()
	if _, ok := b.thumbnails[hash]; ok {
		return false
	}
	b.thumbnails[hash] = "" // reservation
	return true
}

func (b *BulkThumbnailCreator) installThumb(hash uint64, thumb, sourceID string) {
	b.idxMu.Lock()
	defer b.idxMu.Unlock()
	b.thumbnails[hash] = thumb
	b.mappings[sourceID] = hash
	b.generated++
}

func (b *BulkThumbnailCreator) recordDedup(hash uint64, sourceID string) {
	b.idxMu.Lock()
	defer b.idxMu.Unlock()
	b.mappings[sourceID] = hash
	b.dedup++
}

// RunWithThumb spawns workers; each pops requests, hashes content, and
// either generates a thumbnail (claim path) or records the existing one
// (dedup path). Returns when the queue is empty and all workers have exited.
func (b *BulkThumbnailCreator) RunWithThumb(
	hasher func([]byte) uint64,
	thumbnailer func(uint64, []byte) string,
) ThumbnailIndexSnapshot {
	var wg sync.WaitGroup
	wg.Add(b.workerCount)
	for i := 0; i < b.workerCount; i++ {
		go func() {
			defer wg.Done()
			for {
				req, ok := b.pop()
				if !ok {
					return
				}
				hash := hasher(req.Content)
				if b.claim(hash) {
					thumb := thumbnailer(hash, req.Content)
					b.installThumb(hash, thumb, req.SourceID)
				} else {
					b.recordDedup(hash, req.SourceID)
				}
			}
		}()
	}
	wg.Wait()

	b.idxMu.Lock()
	defer b.idxMu.Unlock()
	// Copy maps so the snapshot is safe after the creator is dropped/reused.
	thumbs := make(map[uint64]string, len(b.thumbnails))
	for k, v := range b.thumbnails {
		thumbs[k] = v
	}
	maps := make(map[string]uint64, len(b.mappings))
	for k, v := range b.mappings {
		maps[k] = v
	}
	return ThumbnailIndexSnapshot{
		Thumbnails: thumbs, Mappings: maps,
		Generated: b.generated, Deduplicated: b.dedup,
	}
}
