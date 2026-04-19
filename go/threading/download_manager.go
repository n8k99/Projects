// Download Manager — N concurrent workers sharing one job queue.
//
// Fan-out: the manager pushes jobs onto a buffered channel; N worker
// goroutines range over the channel in parallel. Fan-in: each worker sends
// its result on a results channel; the manager collects after all workers
// finish.
//
// Go's channels are the idiomatic shared queue — this file uses one
// channel for jobs and one channel for results. Closing the jobs channel
// tells every worker "no more work" all at once; a WaitGroup then tells
// the collector "everyone's done."
package threading

import (
	"context"
	"fmt"
	"sync"
)

// Download is a job to run.
type Download struct {
	URL           string
	ExpectedBytes int64
}

// OutcomeStatus is one of the three terminal states a job can reach.
type OutcomeStatus int

const (
	OutcomeCompleted OutcomeStatus = iota
	OutcomeCancelled
	OutcomeFailed
)

// DownloadResult is the manager's record of a finished (or skipped) job.
type DownloadResult struct {
	URL    string
	Status OutcomeStatus
	Bytes  int64  // valid when Status == OutcomeCompleted
	Error  string // valid when Status == OutcomeFailed
}

// DownloadManager runs a pool of workers against a channel of jobs.
type DownloadManager struct {
	workerCount int
	jobs        chan Download
	results     chan DownloadResult
}

// NewDownloadManager creates a manager with the given worker count and
// an optional internal queue capacity (0 = unbuffered / synchronous handoff).
func NewDownloadManager(workerCount, queueCap int) *DownloadManager {
	if workerCount < 1 {
		panic(fmt.Sprintf("worker count must be >= 1, got %d", workerCount))
	}
	return &DownloadManager{
		workerCount: workerCount,
		jobs:        make(chan Download, queueCap),
		results:     make(chan DownloadResult, queueCap),
	}
}

// Enqueue adds a job. Blocks if the queue is full.
func (m *DownloadManager) Enqueue(d Download) { m.jobs <- d }

// Close signals "no more jobs will be enqueued"; workers drain then exit.
func (m *DownloadManager) Close() { close(m.jobs) }

// RunWith starts the worker pool and returns results in completion order.
// Call Enqueue zero or more times before (or concurrently with) RunWith,
// then call Close when done enqueueing. RunWith returns when every job
// has been processed or ctx is cancelled.
func (m *DownloadManager) RunWith(
	ctx context.Context,
	simulate func(Download) (int64, error),
) []DownloadResult {
	var wg sync.WaitGroup
	wg.Add(m.workerCount)
	for i := 0; i < m.workerCount; i++ {
		go func() {
			defer wg.Done()
			for d := range m.jobs {
				select {
				case <-ctx.Done():
					m.results <- DownloadResult{URL: d.URL, Status: OutcomeCancelled}
					continue
				default:
				}
				bytes, err := simulate(d)
				if err != nil {
					m.results <- DownloadResult{URL: d.URL, Status: OutcomeFailed,
						Error: err.Error()}
				} else {
					m.results <- DownloadResult{URL: d.URL, Status: OutcomeCompleted,
						Bytes: bytes}
				}
			}
		}()
	}

	// Collector goroutine closes results once all workers finish.
	go func() {
		wg.Wait()
		close(m.results)
	}()

	out := make([]DownloadResult, 0)
	for r := range m.results {
		out = append(out, r)
	}
	return out
}
