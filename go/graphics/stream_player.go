// Stream Player — segmented video with buffer-based adaptive bitrate.
package graphics

import "fmt"

// SPPlayerState is the playback FSM state.
type SPPlayerState int

const (
	SPIdle      SPPlayerState = 0
	SPBuffering SPPlayerState = 1
	SPPlaying   SPPlayerState = 2
	SPPaused    SPPlayerState = 3
	SPEnded     SPPlayerState = 4
)

func (s SPPlayerState) String() string {
	return [...]string{"idle", "buffering", "playing", "paused", "ended"}[s]
}

// SPSegment is one streaming chunk.
type SPSegment struct {
	Index           uint32
	DurationMs      uint32
	SizesByBitrate  map[uint32]uint64
}

// SizeFor returns bytes at a given bitrate, or (0, false).
func (s SPSegment) SizeFor(bitrate uint32) (uint64, bool) {
	b, ok := s.SizesByBitrate[bitrate]
	return b, ok
}

// SPStream is a stream definition (multiple bitrates, segmented).
type SPStream struct {
	BitratesBps    []uint32
	Segments       []SPSegment
	SafetyFactor   float64
	TargetBufferMs uint32
}

// TotalDurationMs sums segment durations.
func (s *SPStream) TotalDurationMs() uint32 {
	var total uint32
	for _, seg := range s.Segments { total += seg.DurationMs }
	return total
}

// PickBitrate returns the highest bitrate that fits the budget.
func (s *SPStream) PickBitrate(bufferMs, bandwidthBps uint32, segment SPSegment) uint32 {
	effectiveBw := float64(bandwidthBps) * s.SafetyFactor
	if effectiveBw < 1.0 { effectiveBw = 1.0 }
	rawBudget := bufferMs
	if segment.DurationMs > rawBudget { rawBudget = segment.DurationMs }
	budgetMs := rawBudget
	cap := s.TargetBufferMs * 2
	if budgetMs > cap { budgetMs = cap }
	chosen := s.BitratesBps[0]
	for _, bitrate := range s.BitratesBps {
		bytes, ok := segment.SizeFor(bitrate)
		if !ok { continue }
		downloadMs := float64(bytes) * 8.0 / effectiveBw * 1000.0
		if downloadMs <= float64(budgetMs) {
			chosen = bitrate
		}
	}
	return chosen
}

// SPEventKind tags a player event.
type SPEventKind int

const (
	SPEStateChanged    SPEventKind = 0
	SPESegmentLoaded   SPEventKind = 1
	SPEBitrateSwitched SPEventKind = 2
	SPERebuffer        SPEventKind = 3
	SPESeeked          SPEventKind = 4
)

// SPPlayerEvent is one entry in the event log.
type SPPlayerEvent struct {
	Kind   SPEventKind
	Detail string
}

// SPPlayer holds playback state + event log.
type SPPlayer struct {
	State          SPPlayerState
	CurrentSegment uint32
	BufferMs       uint32
	PlayedMs       uint32
	BandwidthBps   uint32
	CurrentBitrate uint32
	Events         []SPPlayerEvent
	Stream         *SPStream
}

// NewSPPlayer returns a default player.
func NewSPPlayer() *SPPlayer {
	return &SPPlayer{State: SPIdle, BandwidthBps: 1_000_000}
}

func (p *SPPlayer) setState(state SPPlayerState) {
	if p.State == state { return }
	old := p.State
	p.State = state
	p.Events = append(p.Events, SPPlayerEvent{
		Kind: SPEStateChanged,
		Detail: fmt.Sprintf("%s -> %s", old, state),
	})
}

// Attach binds a stream and transitions to Buffering.
func (p *SPPlayer) Attach(stream *SPStream) {
	p.Stream = stream
	p.CurrentBitrate = stream.BitratesBps[0]
	p.setState(SPBuffering)
}

// SetBandwidth updates the bandwidth estimate.
func (p *SPPlayer) SetBandwidth(bps uint32) { p.BandwidthBps = bps }

// LoadNextSegment picks bitrate + adds to buffer.
func (p *SPPlayer) LoadNextSegment() bool {
	if p.Stream == nil { return false }
	if int(p.CurrentSegment) >= len(p.Stream.Segments) { return false }
	seg := p.Stream.Segments[p.CurrentSegment]
	newBitrate := p.Stream.PickBitrate(p.BufferMs, p.BandwidthBps, seg)
	if newBitrate != p.CurrentBitrate {
		p.Events = append(p.Events, SPPlayerEvent{
			Kind: SPEBitrateSwitched,
			Detail: fmt.Sprintf("%d -> %d", p.CurrentBitrate, newBitrate),
		})
		p.CurrentBitrate = newBitrate
	}
	p.BufferMs += seg.DurationMs
	p.CurrentSegment++
	p.Events = append(p.Events, SPPlayerEvent{
		Kind: SPESegmentLoaded,
		Detail: fmt.Sprintf("segment %d @ %dbps", seg.Index, newBitrate),
	})
	if p.State == SPBuffering && p.BufferMs >= p.Stream.TargetBufferMs {
		p.setState(SPPlaying)
	}
	return true
}

// Tick drains buffer while playing.
func (p *SPPlayer) Tick(ms uint32) {
	if p.State != SPPlaying || p.Stream == nil { return }
	drained := ms
	if drained > p.BufferMs { drained = p.BufferMs }
	p.BufferMs -= drained
	p.PlayedMs += drained
	atEnd := int(p.CurrentSegment) >= len(p.Stream.Segments)
	if p.BufferMs == 0 && atEnd {
		p.setState(SPEnded)
	} else if p.BufferMs == 0 && !atEnd {
		p.Events = append(p.Events, SPPlayerEvent{
			Kind: SPERebuffer,
			Detail: fmt.Sprintf("buffer empty at %dms", p.PlayedMs),
		})
		p.setState(SPBuffering)
	}
}

// Pause transitions Playing → Paused.
func (p *SPPlayer) Pause() {
	if p.State == SPPlaying { p.setState(SPPaused) }
}

// Play transitions Paused → Playing.
func (p *SPPlayer) Play() {
	if p.State == SPPaused { p.setState(SPPlaying) }
}

// SeekToSegment jumps to a segment; resets buffer.
func (p *SPPlayer) SeekToSegment(segmentIdx uint32) bool {
	if p.Stream == nil { return false }
	if int(segmentIdx) >= len(p.Stream.Segments) { return false }
	p.CurrentSegment = segmentIdx
	p.BufferMs = 0
	var played uint32
	for i := uint32(0); i < segmentIdx; i++ {
		played += p.Stream.Segments[i].DurationMs
	}
	p.PlayedMs = played
	p.Events = append(p.Events, SPPlayerEvent{
		Kind: SPESeeked,
		Detail: fmt.Sprintf("segment %d", segmentIdx),
	})
	p.setState(SPBuffering)
	return true
}
