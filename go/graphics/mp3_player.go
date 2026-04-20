// Mp3 Player — playlist navigation with shuffle/repeat + position tracking.
package graphics

import "fmt"

type Track struct {
	ID         int
	Title      string
	Artist     string
	DurationMs int
}

type Library struct {
	Tracks map[int]*Track
}

func NewLibrary() *Library {
	return &Library{Tracks: map[int]*Track{}}
}

func (l *Library) Add(t *Track)    { l.Tracks[t.ID] = t }
func (l *Library) Get(id int) *Track { return l.Tracks[id] }

type Playlist struct {
	ID       int
	Title    string
	TrackIDs []int
}

type RepeatMode int

const (
	RepeatOff RepeatMode = iota
	RepeatOne
	RepeatAll
)

func (r RepeatMode) String() string {
	return []string{"off", "one", "all"}[r]
}

type PlayerState int

const (
	StateIdle PlayerState = iota
	StatePlaying
	StatePaused
	StateStopped
)

func (s PlayerState) String() string {
	return []string{"idle", "playing", "paused", "stopped"}[s]
}

type PlayEventKind int

const (
	EvStateChanged PlayEventKind = iota
	EvTrackChanged
	EvRepeatChanged
	EvShuffleChanged
)

type PlayEvent struct {
	Kind   PlayEventKind
	Detail string
}

type nextAction struct {
	kind string // "advance" | "repeat" | "end"
	pos  int
}

type Player struct {
	Playlist      *Playlist
	QueueOrder    []int
	QueuePosition int
	PositionMs    int
	State         PlayerState
	Repeat        RepeatMode
	Shuffle       bool
	ShuffleSeed   uint64
	History       []int
	Events        []PlayEvent
}

func NewPlayer() *Player {
	return &Player{State: StateIdle, Repeat: RepeatOff, ShuffleSeed: 42}
}

func (p *Player) CurrentTrackID() (int, bool) {
	if p.QueuePosition < 0 || p.QueuePosition >= len(p.QueueOrder) {
		return 0, false
	}
	return p.QueueOrder[p.QueuePosition], true
}

func (p *Player) setState(s PlayerState) {
	if p.State == s {
		return
	}
	old := p.State
	p.State = s
	p.Events = append(p.Events, PlayEvent{EvStateChanged, fmt.Sprintf("%s -> %s", old, s)})
}

func (p *Player) emitTrackChanged() {
	if id, ok := p.CurrentTrackID(); ok {
		p.Events = append(p.Events, PlayEvent{EvTrackChanged, fmt.Sprintf("track %d", id)})
	}
}

func (p *Player) LoadPlaylist(pl *Playlist) {
	p.Playlist = pl
	p.QueueOrder = append([]int(nil), pl.TrackIDs...)
	p.QueuePosition = 0
	p.PositionMs = 0
	if p.Shuffle {
		p.applyShuffle()
	}
	p.setState(StateStopped)
}

func (p *Player) Play() {
	if len(p.QueueOrder) > 0 {
		p.setState(StatePlaying)
	}
}

func (p *Player) Pause() {
	if p.State == StatePlaying {
		p.setState(StatePaused)
	}
}

func (p *Player) Stop() {
	p.setState(StateStopped)
	p.PositionMs = 0
}

func (p *Player) computeNext() nextAction {
	n := len(p.QueueOrder)
	switch p.Repeat {
	case RepeatOne:
		return nextAction{"repeat", 0}
	case RepeatAll:
		return nextAction{"advance", (p.QueuePosition + 1) % n}
	}
	if p.QueuePosition+1 < n {
		return nextAction{"advance", p.QueuePosition + 1}
	}
	return nextAction{"end", 0}
}

func (p *Player) Tick(ms int, lib *Library) {
	if p.State != StatePlaying || len(p.QueueOrder) == 0 {
		return
	}
	curID, ok := p.CurrentTrackID()
	if !ok {
		return
	}
	tr := lib.Get(curID)
	if tr == nil {
		return
	}
	remaining := tr.DurationMs - p.PositionMs
	if remaining < 0 {
		remaining = 0
	}
	if ms >= remaining {
		p.History = append(p.History, curID)
		p.PositionMs = 0
		action := p.computeNext()
		switch action.kind {
		case "advance":
			p.QueuePosition = action.pos
			p.emitTrackChanged()
		case "repeat":
			p.emitTrackChanged()
		case "end":
			p.setState(StateStopped)
		}
	} else {
		p.PositionMs += ms
	}
}

func (p *Player) Next(lib *Library) bool {
	curID, ok := p.CurrentTrackID()
	if !ok {
		return false
	}
	p.History = append(p.History, curID)
	p.PositionMs = 0
	action := p.computeNext()
	switch action.kind {
	case "advance":
		p.QueuePosition = action.pos
		p.emitTrackChanged()
		return true
	case "repeat":
		p.emitTrackChanged()
		return true
	}
	p.setState(StateStopped)
	return false
}

func (p *Player) Prev(lib *Library) bool {
	if p.PositionMs > 3000 {
		p.PositionMs = 0
		return true
	}
	if len(p.History) > 0 {
		prevID := p.History[len(p.History)-1]
		p.History = p.History[:len(p.History)-1]
		for i, id := range p.QueueOrder {
			if id == prevID {
				p.QueuePosition = i
				p.PositionMs = 0
				p.emitTrackChanged()
				return true
			}
		}
	}
	p.PositionMs = 0
	return false
}

func (p *Player) SetRepeat(mode RepeatMode) {
	if p.Repeat == mode {
		return
	}
	p.Repeat = mode
	p.Events = append(p.Events, PlayEvent{EvRepeatChanged, mode.String()})
}

func (p *Player) SetShuffle(on bool) {
	if p.Shuffle == on {
		return
	}
	p.Shuffle = on
	detail := "off"
	if on {
		detail = "on"
	}
	p.Events = append(p.Events, PlayEvent{EvShuffleChanged, detail})
	if on {
		p.applyShuffle()
	} else {
		p.unshuffle()
	}
}

func (p *Player) applyShuffle() {
	if p.Playlist == nil {
		return
	}
	p.QueueOrder = append([]int(nil), p.Playlist.TrackIDs...)
	state := p.ShuffleSeed + 1
	n := len(p.QueueOrder)
	for i := n - 1; i > 0; i-- {
		state = state*6364136223846793005 + 1442695040888963407
		r := uint32(state >> 33)
		j := int(r % uint32(i+1))
		p.QueueOrder[i], p.QueueOrder[j] = p.QueueOrder[j], p.QueueOrder[i]
	}
	curID, ok := p.CurrentTrackID()
	if !ok {
		return
	}
	for i, id := range p.QueueOrder {
		if id == curID {
			p.QueueOrder[0], p.QueueOrder[i] = p.QueueOrder[i], p.QueueOrder[0]
			p.QueuePosition = 0
			return
		}
	}
}

func (p *Player) unshuffle() {
	if p.Playlist == nil {
		return
	}
	curID, ok := p.CurrentTrackID()
	p.QueueOrder = append([]int(nil), p.Playlist.TrackIDs...)
	if ok {
		for i, id := range p.QueueOrder {
			if id == curID {
				p.QueuePosition = i
				return
			}
		}
	}
}

func Mp3PlayerDemo() {
	lib := NewLibrary()
	for _, row := range []struct {
		id    int
		title string
		dur   int
	}{{1, "Alpha", 120000}, {2, "Beta", 180000}, {3, "Gamma", 150000}, {4, "Delta", 200000}} {
		lib.Add(&Track{ID: row.id, Title: row.title, Artist: "Various", DurationMs: row.dur})
	}
	p := NewPlayer()
	p.LoadPlaylist(&Playlist{ID: 1, Title: "Mix", TrackIDs: []int{1, 2, 3, 4}})
	p.Play()
	id, _ := p.CurrentTrackID()
	fmt.Printf("start: track=%d state=%s\n", id, p.State)
	p.Tick(125000, lib)
	id, _ = p.CurrentTrackID()
	fmt.Printf("after 125s: track=%d pos=%dms\n", id, p.PositionMs)
	p.SetRepeat(RepeatAll)
	p.QueuePosition = 3
	p.PositionMs = 0
	p.Tick(210000, lib)
	id, _ = p.CurrentTrackID()
	fmt.Printf("after wrap: track=%d state=%s\n", id, p.State)
	p.SetShuffle(true)
	fmt.Printf("shuffle order: %v\n", p.QueueOrder)
}
