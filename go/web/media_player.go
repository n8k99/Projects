// Media Player Widget — playback FSM + playlist with repeat/shuffle modes.
package web

// MPState is the playback FSM state.
type MPState int

const (
	MPStopped MPState = iota
	MPPlaying
	MPPaused
)

// MPRepeat is the repeat mode.
type MPRepeat int

const (
	MPRepeatNone MPRepeat = iota
	MPRepeatOne
	MPRepeatAll
)

// MPTrack is one track in a playlist.
type MPTrack struct {
	ID         int64
	Title      string
	Artist     string
	DurationMs int64
}

// MPEventKind identifies an event from the player.
type MPEventKind int

const (
	MPTrackStarted MPEventKind = iota
	MPTrackEnded
	MPPlaylistEnded
	MPStateChanged
)

// MPEvent is one playback event.
type MPEvent struct {
	Kind     MPEventKind
	TrackID  int64
	NewState MPState
}

// MPPlayer is the playback state machine.
type MPPlayer struct {
	playlist      []MPTrack
	state         MPState
	currentIdx    int
	hasCurrent    bool
	positionMs    int64
	volume        float64
	repeat        MPRepeat
	shuffle       bool
	shuffleOrder  []int
	rngState      uint64
}

// NewMPPlayer returns a new Stopped player.
func NewMPPlayer() *MPPlayer {
	return &MPPlayer{volume: 1.0, rngState: 0xDEADBEEF}
}

// LoadPlaylist replaces the playlist and resets to Stopped.
func (p *MPPlayer) LoadPlaylist(tracks []MPTrack) {
	p.playlist = append([]MPTrack{}, tracks...)
	p.state = MPStopped
	p.hasCurrent = false
	p.currentIdx = 0
	p.positionMs = 0
	p.rebuildShuffleOrder()
}

// State returns the current FSM state.
func (p *MPPlayer) State() MPState     { return p.state }
func (p *MPPlayer) PositionMs() int64  { return p.positionMs }
func (p *MPPlayer) Volume() float64    { return p.volume }
func (p *MPPlayer) Repeat() MPRepeat   { return p.repeat }
func (p *MPPlayer) Shuffle() bool      { return p.shuffle }
func (p *MPPlayer) Playlist() []MPTrack { return p.playlist }

// CurrentTrack returns the currently-selected track, or nil.
func (p *MPPlayer) CurrentTrack() *MPTrack {
	if !p.hasCurrent {
		return nil
	}
	plIdx := p.currentIdx
	if p.shuffle {
		plIdx = p.shuffleOrder[p.currentIdx]
	}
	if plIdx < 0 || plIdx >= len(p.playlist) {
		return nil
	}
	t := p.playlist[plIdx]
	return &t
}

// SetVolume clamps to [0, 1].
func (p *MPPlayer) SetVolume(v float64) {
	if v < 0 {
		v = 0
	} else if v > 1 {
		v = 1
	}
	p.volume = v
}

func (p *MPPlayer) SetRepeat(m MPRepeat) { p.repeat = m }

// SetShuffle enables/disables shuffle, preserving current track identity.
func (p *MPPlayer) SetShuffle(on bool) {
	if on == p.shuffle {
		return
	}
	prevPlIdx := p.currentIdx
	if p.shuffle && p.hasCurrent {
		prevPlIdx = p.shuffleOrder[p.currentIdx]
	}
	p.shuffle = on
	p.rebuildShuffleOrder()
	if !p.hasCurrent {
		return
	}
	if on {
		for i, x := range p.shuffleOrder {
			if x == prevPlIdx {
				p.currentIdx = i
				return
			}
		}
	} else {
		p.currentIdx = prevPlIdx
	}
}

// Play is context-dependent.
func (p *MPPlayer) Play() []MPEvent {
	switch p.state {
	case MPPlaying:
		return nil
	case MPPaused:
		p.state = MPPlaying
		return []MPEvent{{Kind: MPStateChanged, NewState: p.state}}
	case MPStopped:
		if len(p.playlist) == 0 {
			return nil
		}
		p.currentIdx = 0
		p.hasCurrent = true
		p.positionMs = 0
		p.state = MPPlaying
		out := []MPEvent{}
		if t := p.CurrentTrack(); t != nil {
			out = append(out, MPEvent{Kind: MPTrackStarted, TrackID: t.ID})
		}
		out = append(out, MPEvent{Kind: MPStateChanged, NewState: p.state})
		return out
	}
	return nil
}

func (p *MPPlayer) Pause() []MPEvent {
	if p.state == MPPlaying {
		p.state = MPPaused
		return []MPEvent{{Kind: MPStateChanged, NewState: p.state}}
	}
	return nil
}

func (p *MPPlayer) Stop() []MPEvent {
	if p.state == MPStopped {
		return nil
	}
	p.state = MPStopped
	p.hasCurrent = false
	p.positionMs = 0
	return []MPEvent{{Kind: MPStateChanged, NewState: p.state}}
}

func (p *MPPlayer) NextTrack() []MPEvent {
	if !p.hasCurrent {
		return p.Play()
	}
	if p.repeat == MPRepeatOne {
		p.positionMs = 0
		if t := p.CurrentTrack(); t != nil {
			return []MPEvent{{Kind: MPTrackStarted, TrackID: t.ID}}
		}
		return nil
	}
	nxt := p.currentIdx + 1
	if nxt < len(p.playlist) {
		p.currentIdx = nxt
		p.positionMs = 0
		if t := p.CurrentTrack(); t != nil {
			return []MPEvent{{Kind: MPTrackStarted, TrackID: t.ID}}
		}
		return nil
	}
	if p.repeat == MPRepeatAll {
		if p.shuffle {
			p.rebuildShuffleOrder()
		}
		p.currentIdx = 0
		p.positionMs = 0
		if t := p.CurrentTrack(); t != nil {
			return []MPEvent{{Kind: MPTrackStarted, TrackID: t.ID}}
		}
		return nil
	}
	p.state = MPStopped
	p.hasCurrent = false
	p.positionMs = 0
	return []MPEvent{
		{Kind: MPPlaylistEnded},
		{Kind: MPStateChanged, NewState: p.state},
	}
}

func (p *MPPlayer) PreviousTrack() []MPEvent {
	if !p.hasCurrent {
		return nil
	}
	if p.currentIdx == 0 {
		p.positionMs = 0
		if t := p.CurrentTrack(); t != nil {
			return []MPEvent{{Kind: MPTrackStarted, TrackID: t.ID}}
		}
		return nil
	}
	p.currentIdx--
	p.positionMs = 0
	if t := p.CurrentTrack(); t != nil {
		return []MPEvent{{Kind: MPTrackStarted, TrackID: t.ID}}
	}
	return nil
}

func (p *MPPlayer) SeekMs(pos int64) bool {
	t := p.CurrentTrack()
	if t == nil || pos > t.DurationMs || pos < 0 {
		return false
	}
	p.positionMs = pos
	return true
}

func (p *MPPlayer) Tick(deltaMs int64) []MPEvent {
	if p.state != MPPlaying {
		return nil
	}
	var events []MPEvent
	remaining := deltaMs
	for remaining > 0 {
		cur := p.CurrentTrack()
		if cur == nil {
			break
		}
		timeLeft := cur.DurationMs - p.positionMs
		if timeLeft < 0 {
			timeLeft = 0
		}
		if remaining < timeLeft {
			p.positionMs += remaining
			remaining = 0
		} else {
			remaining -= timeLeft
			p.positionMs = cur.DurationMs
			events = append(events, MPEvent{Kind: MPTrackEnded, TrackID: cur.ID})
			events = append(events, p.NextTrack()...)
			if p.state == MPStopped {
				break
			}
		}
	}
	return events
}

func (p *MPPlayer) rebuildShuffleOrder() {
	n := len(p.playlist)
	p.shuffleOrder = make([]int, n)
	for i := 0; i < n; i++ {
		p.shuffleOrder[i] = i
	}
	if !p.shuffle || n <= 1 {
		return
	}
	for i := n - 1; i >= 1; i-- {
		p.rngState = p.rngState*6364136223846793005 + 1442695040888963407
		j := int((p.rngState >> 32) % uint64(i+1))
		p.shuffleOrder[i], p.shuffleOrder[j] = p.shuffleOrder[j], p.shuffleOrder[i]
	}
}
