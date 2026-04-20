// YouTube Downloader — concurrency-limited queue with resumable downloads.
package graphics

import "fmt"

type VideoFormat struct {
	ID          string
	Resolution  uint32
	Codec       string
	Container   string
	SizeBytes   uint64
	BitrateBps  uint64
}

type FormatPolicyKind int

const (
	PolicyMaxQuality FormatPolicyKind = iota
	PolicyLowest
	PolicyBestUnder
)

type FormatPolicy struct {
	Kind     FormatPolicyKind
	CapBytes uint64
}

func PickFormat(formats []VideoFormat, policy FormatPolicy) *VideoFormat {
	if len(formats) == 0 {
		return nil
	}
	switch policy.Kind {
	case PolicyMaxQuality:
		best := &formats[0]
		for i := 1; i < len(formats); i++ {
			if formats[i].Resolution > best.Resolution {
				best = &formats[i]
			}
		}
		return best
	case PolicyLowest:
		best := &formats[0]
		for i := 1; i < len(formats); i++ {
			if formats[i].Resolution < best.Resolution {
				best = &formats[i]
			}
		}
		return best
	case PolicyBestUnder:
		var best *VideoFormat
		for i := range formats {
			f := &formats[i]
			if f.SizeBytes <= policy.CapBytes {
				if best == nil || f.Resolution > best.Resolution {
					best = f
				}
			}
		}
		return best
	}
	return nil
}

type RetryPolicy struct {
	MaxAttempts uint32
	InitialMs   uint64
	Multiplier  uint64
}

func DefaultRetryPolicy() RetryPolicy {
	return RetryPolicy{MaxAttempts: 3, InitialMs: 1000, Multiplier: 2}
}

func (r RetryPolicy) BackoffForAttempt(n uint32) uint64 {
	if n == 0 {
		return 0
	}
	ms := r.InitialMs
	for i := uint32(1); i < n; i++ {
		ms *= r.Multiplier
	}
	return ms
}

type DownloadStateKind int

const (
	DownloadPending DownloadStateKind = iota
	DownloadInFlight
	DownloadRetrying
	DownloadSucceeded
	DownloadFailed
)

type DownloadState struct {
	Kind       DownloadStateKind
	ReadyAtMs  uint64
}

type Download struct {
	ID         uint64
	VideoID    string
	Format     VideoFormat
	BytesDone  uint64
	BytesTotal uint64
	Attempts   uint32
	State      DownloadState
}

type ManagerEventKind int

const (
	EventEnqueued ManagerEventKind = iota
	EventStarted
	EventProgress
	EventCompleted
	EventFailed
	EventRetryScheduled
	EventAbandoned
)

type ManagerEvent struct {
	Kind   ManagerEventKind
	ID     uint64
	Detail string
}

type Manager struct {
	MaxConcurrent  int
	BandwidthBps   uint64
	RetryPolicy    RetryPolicy
	Downloads      []*Download
	PendingQueue   []uint64
	ElapsedMs      uint64
	Events         []ManagerEvent
	nextID         uint64
}

func NewManager(maxConcurrent int, bandwidthBps uint64, retryPolicy RetryPolicy) *Manager {
	return &Manager{
		MaxConcurrent: maxConcurrent,
		BandwidthBps:  bandwidthBps,
		RetryPolicy:   retryPolicy,
		nextID:        1,
	}
}

func (m *Manager) Enqueue(videoID string, format VideoFormat) uint64 {
	id := m.nextID
	m.nextID++
	d := &Download{
		ID: id, VideoID: videoID, Format: format,
		BytesTotal: format.SizeBytes,
		State:      DownloadState{Kind: DownloadPending},
	}
	m.Downloads = append(m.Downloads, d)
	m.PendingQueue = append(m.PendingQueue, id)
	m.Events = append(m.Events, ManagerEvent{Kind: EventEnqueued, ID: id})
	return id
}

func (m *Manager) Download(id uint64) *Download {
	for _, d := range m.Downloads {
		if d.ID == id {
			return d
		}
	}
	return nil
}

func (m *Manager) ActiveCount() int {
	n := 0
	for _, d := range m.Downloads {
		if d.State.Kind == DownloadInFlight {
			n++
		}
	}
	return n
}

func (m *Manager) promoteRetries() {
	now := m.ElapsedMs
	for _, d := range m.Downloads {
		if d.State.Kind == DownloadRetrying && d.State.ReadyAtMs <= now {
			d.State = DownloadState{Kind: DownloadPending}
			m.PendingQueue = append(m.PendingQueue, d.ID)
		}
	}
}

func (m *Manager) fillSlots() {
	for m.ActiveCount() < m.MaxConcurrent && len(m.PendingQueue) > 0 {
		id := m.PendingQueue[0]
		m.PendingQueue = m.PendingQueue[1:]
		d := m.Download(id)
		if d == nil || d.State.Kind != DownloadPending {
			continue
		}
		d.Attempts++
		d.State = DownloadState{Kind: DownloadInFlight}
		m.Events = append(m.Events, ManagerEvent{
			Kind: EventStarted, ID: id,
			Detail: fmt.Sprintf("attempt %d", d.Attempts),
		})
	}
}

func (m *Manager) ReportFailure(id uint64) {
	d := m.Download(id)
	if d == nil || d.State.Kind != DownloadInFlight {
		return
	}
	if d.Attempts < m.RetryPolicy.MaxAttempts {
		backoff := m.RetryPolicy.BackoffForAttempt(d.Attempts)
		ready := m.ElapsedMs + backoff
		d.State = DownloadState{Kind: DownloadRetrying, ReadyAtMs: ready}
		m.Events = append(m.Events, ManagerEvent{
			Kind: EventFailed, ID: id,
			Detail: fmt.Sprintf("attempt %d", d.Attempts),
		})
		m.Events = append(m.Events, ManagerEvent{
			Kind: EventRetryScheduled, ID: id,
			Detail: fmt.Sprintf("ready %d", ready),
		})
	} else {
		d.State = DownloadState{Kind: DownloadFailed}
		m.Events = append(m.Events, ManagerEvent{
			Kind: EventFailed, ID: id,
			Detail: fmt.Sprintf("attempt %d", d.Attempts),
		})
		m.Events = append(m.Events, ManagerEvent{Kind: EventAbandoned, ID: id})
	}
}

func (m *Manager) Tick(ms uint64) {
	m.ElapsedMs += ms
	m.promoteRetries()
	m.fillSlots()

	var active []*Download
	for _, d := range m.Downloads {
		if d.State.Kind == DownloadInFlight {
			active = append(active, d)
		}
	}
	if len(active) == 0 {
		return
	}

	perStreamBps := m.BandwidthBps / uint64(len(active))
	bytesThisTick := (perStreamBps * ms) / 8000

	for _, d := range active {
		before := d.BytesDone
		d.BytesDone += bytesThisTick
		if d.BytesDone > d.BytesTotal {
			d.BytesDone = d.BytesTotal
		}
		if d.BytesDone > before {
			m.Events = append(m.Events, ManagerEvent{
				Kind: EventProgress, ID: d.ID,
				Detail: fmt.Sprintf("%d/%d", d.BytesDone, d.BytesTotal),
			})
		}
		if d.BytesDone >= d.BytesTotal {
			d.State = DownloadState{Kind: DownloadSucceeded}
			m.Events = append(m.Events, ManagerEvent{Kind: EventCompleted, ID: d.ID})
		}
	}
}

func YoutubeDownloaderDemo() {
	formats := []VideoFormat{
		{ID: "a", Resolution: 480, Codec: "avc1", Container: "mp4", SizeBytes: 50_000_000, BitrateBps: 500_000},
		{ID: "b", Resolution: 720, Codec: "avc1", Container: "mp4", SizeBytes: 100_000_000, BitrateBps: 1_000_000},
		{ID: "c", Resolution: 1080, Codec: "avc1", Container: "mp4", SizeBytes: 200_000_000, BitrateBps: 2_000_000},
	}

	fmt.Println("format picks:")
	fmt.Printf("  max_quality -> %s\n", PickFormat(formats, FormatPolicy{Kind: PolicyMaxQuality}).ID)
	fmt.Printf("  lowest -> %s\n", PickFormat(formats, FormatPolicy{Kind: PolicyLowest}).ID)
	fmt.Printf("  under 120MB -> %s\n", PickFormat(formats, FormatPolicy{Kind: PolicyBestUnder, CapBytes: 120_000_000}).ID)

	mgr := NewManager(2, 8_000_000, RetryPolicy{MaxAttempts: 3, InitialMs: 1000, Multiplier: 2})
	mgr.Enqueue("v1", formats[1])
	mgr.Enqueue("v2", formats[0])
	mgr.Enqueue("v3", formats[2])

	mgr.Tick(1000)
	fmt.Printf("after 1s: active=%d pending=%d\n", mgr.ActiveCount(), len(mgr.PendingQueue))
	for _, i := range []uint64{1, 2, 3} {
		d := mgr.Download(i)
		stateNames := []string{"pending", "in_flight", "retrying", "succeeded", "failed"}
		fmt.Printf("  [%d] %s %d/%d\n", i, stateNames[d.State.Kind], d.BytesDone, d.BytesTotal)
	}

	mgr.ReportFailure(1)
	d1 := mgr.Download(1)
	fmt.Printf("after failure: [1] retrying ready_at=%d attempts=%d\n", d1.State.ReadyAtMs, d1.Attempts)

	fmt.Printf("backoff for retry 1: %dms\n", mgr.RetryPolicy.BackoffForAttempt(1))
	fmt.Printf("backoff for retry 2: %dms\n", mgr.RetryPolicy.BackoffForAttempt(2))
	fmt.Printf("backoff for retry 3: %dms\n", mgr.RetryPolicy.BackoffForAttempt(3))
}
