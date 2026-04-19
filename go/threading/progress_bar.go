// Progress Bar for Downloads — the first concurrent producer-consumer.
//
// A producer goroutine advances a byte counter; a consumer goroutine reads it
// to render a display. The counter uses sync/atomic so the two never block
// each other; the FSM status uses a CompareAndSwap so terminal states
// (Completed, Cancelled, Failed) are sticky.
//
// This is the first Rosetta Stone project where "who computes" and "who
// displays" are deliberately distinct actors. Cancellation is cooperative:
// the controller calls Cancel; the worker checks IsCancelled between chunks
// and exits.
package threading

import (
	"fmt"
	"strings"
	"sync/atomic"
	"time"
)

// Status is the tracker's FSM state.
type Status int32

const (
	StatusRunning Status = iota
	StatusCompleted
	StatusCancelled
	StatusFailed
)

// ProgressTracker is a concurrent counter + status machine.
type ProgressTracker struct {
	bytesDone  atomic.Int64
	totalBytes int64
	status     atomic.Int32 // Status
	startTime  time.Time
}

// NewProgressTracker starts a running tracker for a download of totalBytes.
func NewProgressTracker(totalBytes int64) *ProgressTracker {
	t := &ProgressTracker{
		totalBytes: totalBytes,
		startTime:  time.Now(),
	}
	t.status.Store(int32(StatusRunning))
	return t
}

// Advance increases the byte counter by delta.
func (t *ProgressTracker) Advance(delta int64) { t.bytesDone.Add(delta) }

// Cancel requests cancellation. Workers see this on their next IsCancelled
// check. Only transitions Running → Cancelled; later calls are no-ops.
func (t *ProgressTracker) Cancel() {
	t.status.CompareAndSwap(int32(StatusRunning), int32(StatusCancelled))
}

// Complete marks the download finished successfully.
func (t *ProgressTracker) Complete() {
	t.status.CompareAndSwap(int32(StatusRunning), int32(StatusCompleted))
}

// Fail marks the download failed.
func (t *ProgressTracker) Fail() {
	t.status.CompareAndSwap(int32(StatusRunning), int32(StatusFailed))
}

// IsCancelled reports whether cancellation has been requested.
func (t *ProgressTracker) IsCancelled() bool {
	return Status(t.status.Load()) == StatusCancelled
}

// ProgressSnapshot is a point-in-time view of a tracker.
type ProgressSnapshot struct {
	BytesDone      int64
	TotalBytes     int64
	Status         Status
	ElapsedSeconds float64
}

// Snapshot reads the current state.
func (t *ProgressTracker) Snapshot() ProgressSnapshot {
	return ProgressSnapshot{
		BytesDone:      t.bytesDone.Load(),
		TotalBytes:     t.totalBytes,
		Status:         Status(t.status.Load()),
		ElapsedSeconds: time.Since(t.startTime).Seconds(),
	}
}

// Percent returns progress as 0–100.
func (s ProgressSnapshot) Percent() float64 {
	if s.TotalBytes == 0 {
		return 100.0
	}
	return float64(s.BytesDone) / float64(s.TotalBytes) * 100.0
}

// BytesPerSecond returns the current throughput; 0 if no time has elapsed.
func (s ProgressSnapshot) BytesPerSecond() float64 {
	if s.ElapsedSeconds <= 0 {
		return 0
	}
	return float64(s.BytesDone) / s.ElapsedSeconds
}

// ETASeconds estimates time-to-completion; -1 if unknown (no rate or already done).
func (s ProgressSnapshot) ETASeconds() float64 {
	rate := s.BytesPerSecond()
	if rate <= 0 || s.BytesDone >= s.TotalBytes {
		return -1
	}
	return float64(s.TotalBytes-s.BytesDone) / rate
}

// RenderBar renders a text-mode progress bar of the given width.
func (s ProgressSnapshot) RenderBar(width int) string {
	pct := s.Percent()
	filled := int(pct/100.0*float64(width) + 0.5)
	if filled > width {
		filled = width
	}
	empty := width - filled
	return fmt.Sprintf("[%s%s] %6.2f%%",
		strings.Repeat("#", filled), strings.Repeat("-", empty), pct)
}
