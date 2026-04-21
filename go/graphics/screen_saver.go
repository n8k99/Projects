// Screen Saver — idle FSM + bouncing-line physics + dim ramp + activity reset.
package graphics

import "fmt"

type SaverState int

const (
	SaverActive SaverState = iota
	SaverDimming
	SaverSaverRunning
	SaverLocked
	SaverWaking
)

func (s SaverState) String() string {
	return []string{"active", "dimming", "saver", "locked", "waking"}[s]
}

type SSConfig struct {
	IdleMs        uint64
	DimMs         uint64
	AutoLockMs    uint64
	WakeMs        uint64
	CanvasWidth   int64
	CanvasHeight  int64
}

type Endpoint struct {
	X, Y   int64
	Vx, Vy int64
}

type SSRgb struct { R, G, B uint8 }

type BouncingLine struct {
	P1, P2 Endpoint
	Color  SSRgb
}

type SSEventKind int

const (
	SSEventStateChanged SSEventKind = iota
	SSEventActivityReset
	SSEventLineBounced
)

type SaverEvent struct {
	Kind   SSEventKind
	AtMs   uint64
	Detail string
}

type ScreenSaver struct {
	Config          SSConfig
	State           SaverState
	Lines           []*BouncingLine
	ElapsedMs       uint64
	LastActivityMs  uint64
	StateEnteredMs  uint64
	Events          []SaverEvent
}

func NewScreenSaver(cfg SSConfig) *ScreenSaver {
	return &ScreenSaver{Config: cfg, State: SaverActive}
}

func (s *ScreenSaver) AddLine(line *BouncingLine) {
	s.Lines = append(s.Lines, line)
}

func (s *ScreenSaver) DimOpacity() uint16 {
	if s.State != SaverDimming { return 0 }
	since := s.ElapsedMs - s.StateEnteredMs
	if s.Config.DimMs == 0 { return 255 }
	if since > s.Config.DimMs { since = s.Config.DimMs }
	return uint16(since * 255 / s.Config.DimMs)
}

func (s *ScreenSaver) transitionTo(newState SaverState) {
	if s.State == newState { return }
	s.Events = append(s.Events, SaverEvent{
		Kind: SSEventStateChanged, AtMs: s.ElapsedMs,
		Detail: fmt.Sprintf("%s -> %s", s.State, newState),
	})
	s.State = newState
	s.StateEnteredMs = s.ElapsedMs
}

func (s *ScreenSaver) RecordActivity() {
	s.LastActivityMs = s.ElapsedMs
	s.Events = append(s.Events, SaverEvent{
		Kind: SSEventActivityReset, AtMs: s.ElapsedMs,
	})
	if s.State == SaverDimming || s.State == SaverSaverRunning {
		s.transitionTo(SaverWaking)
	}
}

func (s *ScreenSaver) Unlock() {
	if s.State == SaverLocked {
		s.transitionTo(SaverWaking)
		s.LastActivityMs = s.ElapsedMs
	}
}

func (s *ScreenSaver) reflectEndpoint(ep *Endpoint, idx int, ms int64, cw, ch int64) {
	ep.X += ep.Vx * ms / 100
	ep.Y += ep.Vy * ms / 100
	if ep.X < 0 {
		ep.X = -ep.X
		ep.Vx = -ep.Vx
		s.Events = append(s.Events, SaverEvent{
			Kind: SSEventLineBounced, AtMs: s.ElapsedMs,
			Detail: fmt.Sprintf("line %d axis x", idx),
		})
	}
	if ep.X > cw {
		ep.X = 2*cw - ep.X
		ep.Vx = -ep.Vx
		s.Events = append(s.Events, SaverEvent{
			Kind: SSEventLineBounced, AtMs: s.ElapsedMs,
			Detail: fmt.Sprintf("line %d axis x", idx),
		})
	}
	if ep.Y < 0 {
		ep.Y = -ep.Y
		ep.Vy = -ep.Vy
		s.Events = append(s.Events, SaverEvent{
			Kind: SSEventLineBounced, AtMs: s.ElapsedMs,
			Detail: fmt.Sprintf("line %d axis y", idx),
		})
	}
	if ep.Y > ch {
		ep.Y = 2*ch - ep.Y
		ep.Vy = -ep.Vy
		s.Events = append(s.Events, SaverEvent{
			Kind: SSEventLineBounced, AtMs: s.ElapsedMs,
			Detail: fmt.Sprintf("line %d axis y", idx),
		})
	}
}

func (s *ScreenSaver) integratePhysics(ms int64) {
	cw := s.Config.CanvasWidth * 100
	ch := s.Config.CanvasHeight * 100
	for idx, line := range s.Lines {
		s.reflectEndpoint(&line.P1, idx, ms, cw, ch)
		s.reflectEndpoint(&line.P2, idx, ms, cw, ch)
	}
}

func (s *ScreenSaver) Tick(ms uint64) {
	s.ElapsedMs += ms
	sinceState := s.ElapsedMs - s.StateEnteredMs
	switch s.State {
	case SaverActive:
		idle := s.ElapsedMs - s.LastActivityMs
		if idle >= s.Config.IdleMs {
			s.transitionTo(SaverDimming)
		}
	case SaverDimming:
		if sinceState >= s.Config.DimMs {
			s.transitionTo(SaverSaverRunning)
		}
	case SaverSaverRunning:
		if sinceState >= s.Config.AutoLockMs {
			s.transitionTo(SaverLocked)
		}
		s.integratePhysics(int64(ms))
	case SaverWaking:
		if sinceState >= s.Config.WakeMs {
			s.transitionTo(SaverActive)
			s.LastActivityMs = s.ElapsedMs
		}
	}
}

func ScreenSaverDemo() {
	cfg := SSConfig{
		IdleMs: 1000, DimMs: 200, AutoLockMs: 500, WakeMs: 100,
		CanvasWidth: 100, CanvasHeight: 100,
	}
	s := NewScreenSaver(cfg)
	s.AddLine(&BouncingLine{
		P1: Endpoint{X: 9500, Y: 5000, Vx: 10000, Vy: 0},
		P2: Endpoint{X: 5000, Y: 5000, Vx: 0, Vy: 0},
		Color: SSRgb{R: 255, G: 255, B: 255},
	})
	fmt.Printf("initial: state=%s\n", s.State)
	s.Tick(500)
	fmt.Printf("at 500ms: state=%s\n", s.State)
	s.Tick(600)
	fmt.Printf("at 1100ms: state=%s dim_opacity=%d\n", s.State, s.DimOpacity())
	s.Tick(100)
	fmt.Printf("at 1200ms: state=%s dim_opacity=%d\n", s.State, s.DimOpacity())
	s.Tick(100)
	fmt.Printf("at 1300ms: state=%s\n", s.State)
	s.Tick(10)
	fmt.Printf("at 1310ms: state=%s p1.x=%d p1.vx=%d\n",
		s.State, s.Lines[0].P1.X, s.Lines[0].P1.Vx)
	s.Tick(600)
	fmt.Printf("at 1910ms: state=%s\n", s.State)
	s.RecordActivity()
	fmt.Printf("after activity: state=%s\n", s.State)
	s.Unlock()
	fmt.Printf("after unlock: state=%s\n", s.State)
	s.Tick(150)
	fmt.Printf("after 150ms in Waking: state=%s\n", s.State)
}
