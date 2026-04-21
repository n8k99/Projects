// Traffic Light — coordinated super-state FSM + preemption + adaptive timing.
package graphics

import "fmt"

type Phase int

const (
	PhaseNSGreen Phase = iota
	PhaseNSYellow
	PhaseAllRed
	PhaseEWGreen
	PhaseEWYellow
	PhaseEmergencyAllRed
)

func (p Phase) String() string {
	return []string{"ns_green", "ns_yellow", "all_red", "ew_green", "ew_yellow", "emergency_all_red"}[p]
}

type LightColor int

const (
	LightRed LightColor = iota
	LightYellow
	LightGreen
)

type PedSignal int

const (
	PedDontWalk PedSignal = iota
	PedWalk
	PedFlashing
)

type TLDirection int

const (
	DirNS TLDirection = iota
	DirEW
)

type TLConfig struct {
	NSGreenMs            uint64
	NSYellowMs           uint64
	EWGreenMs            uint64
	EWYellowMs           uint64
	AllRedMs             uint64
	MaxGreenExtensionMs  uint64
	PedWalkMs            uint64
	PedFlashingMs        uint64
}

type TLEventKind int

const (
	TLPhaseChanged TLEventKind = iota
	TLGreenExtended
	TLPedRequested
	TLPedWalkStarted
	TLPedWalkEnded
	TLEmergencyStarted
	TLEmergencyEnded
)

type TLEvent struct {
	Kind   TLEventKind
	AtMs   uint64
	Detail string
}

type pedState int

const (
	pedInactive pedState = iota
	pedWalk
	pedFlashing
)

type Intersection struct {
	Phase            Phase
	PhaseElapsedMs   uint64
	PhaseDurationMs  uint64
	CycleSide        TLDirection
	Config           TLConfig
	NSQueue          uint32
	EWQueue          uint32
	PedRequested     bool
	pedSt            pedState
	pedElapsedMs     uint64
	savedPhase       *struct{ p Phase; elapsed uint64; side TLDirection }
	TotalElapsedMs   uint64
	Events           []TLEvent
}

func NewIntersection(cfg TLConfig) *Intersection {
	i := &Intersection{
		Phase: PhaseNSGreen, CycleSide: DirEW, Config: cfg,
	}
	i.PhaseDurationMs = i.durationFor(PhaseNSGreen)
	return i
}

func (i *Intersection) durationFor(p Phase) uint64 {
	switch p {
	case PhaseNSGreen: return i.Config.NSGreenMs
	case PhaseNSYellow: return i.Config.NSYellowMs
	case PhaseEWGreen: return i.Config.EWGreenMs
	case PhaseEWYellow: return i.Config.EWYellowMs
	case PhaseAllRed: return i.Config.AllRedMs
	case PhaseEmergencyAllRed: return ^uint64(0)
	}
	return 0
}

func (i *Intersection) NSLight() LightColor {
	switch i.Phase {
	case PhaseNSGreen: return LightGreen
	case PhaseNSYellow: return LightYellow
	}
	return LightRed
}

func (i *Intersection) EWLight() LightColor {
	switch i.Phase {
	case PhaseEWGreen: return LightGreen
	case PhaseEWYellow: return LightYellow
	}
	return LightRed
}

func (i *Intersection) PedSignalValue() PedSignal {
	switch i.pedSt {
	case pedWalk: return PedWalk
	case pedFlashing: return PedFlashing
	}
	return PedDontWalk
}

func (i *Intersection) SetNSQueue(n uint32) { i.NSQueue = n }
func (i *Intersection) SetEWQueue(n uint32) { i.EWQueue = n }

func (i *Intersection) RequestPedCrossing() {
	if i.pedSt != pedInactive { return }
	if !i.PedRequested {
		i.PedRequested = true
		i.Events = append(i.Events, TLEvent{Kind: TLPedRequested, AtMs: i.TotalElapsedMs})
	}
}

func (i *Intersection) BeginEmergency() {
	if i.Phase == PhaseEmergencyAllRed { return }
	i.savedPhase = &struct{ p Phase; elapsed uint64; side TLDirection }{i.Phase, i.PhaseElapsedMs, i.CycleSide}
	at := i.TotalElapsedMs
	from := i.Phase
	i.Phase = PhaseEmergencyAllRed
	i.PhaseElapsedMs = 0
	i.PhaseDurationMs = ^uint64(0)
	i.pedSt = pedInactive
	i.Events = append(i.Events,
		TLEvent{Kind: TLEmergencyStarted, AtMs: at},
		TLEvent{Kind: TLPhaseChanged, AtMs: at, Detail: fmt.Sprintf("%s -> emergency_all_red", from)})
}

func (i *Intersection) EndEmergency() {
	if i.Phase != PhaseEmergencyAllRed { return }
	at := i.TotalElapsedMs
	from := i.Phase
	if i.savedPhase != nil {
		s := i.savedPhase
		i.savedPhase = nil
		i.Phase = s.p
		i.PhaseElapsedMs = s.elapsed
		i.PhaseDurationMs = i.durationFor(s.p)
		i.CycleSide = s.side
	} else {
		i.Phase = PhaseAllRed
		i.PhaseElapsedMs = 0
		i.PhaseDurationMs = i.Config.AllRedMs
	}
	i.Events = append(i.Events,
		TLEvent{Kind: TLEmergencyEnded, AtMs: at},
		TLEvent{Kind: TLPhaseChanged, AtMs: at, Detail: fmt.Sprintf("%s -> %s", from, i.Phase)})
}

func (i *Intersection) considerExtension() {
	var direction TLDirection
	var hasDir bool
	if i.Phase == PhaseNSGreen {
		direction = DirNS; hasDir = true
	} else if i.Phase == PhaseEWGreen {
		direction = DirEW; hasDir = true
	}
	if !hasDir { return }
	var queue uint32
	if direction == DirNS {
		queue = i.NSQueue
	} else {
		queue = i.EWQueue
	}
	if queue == 0 { return }
	var base uint64
	if direction == DirNS {
		base = i.Config.NSGreenMs
	} else {
		base = i.Config.EWGreenMs
	}
	var already uint64
	if i.PhaseDurationMs > base {
		already = i.PhaseDurationMs - base
	}
	if already >= i.Config.MaxGreenExtensionMs { return }
	remaining := i.Config.MaxGreenExtensionMs - already
	grant := uint64(queue) * 2000
	if grant > remaining {
		grant = remaining
	}
	if grant == 0 { return }
	i.PhaseDurationMs += grant
	dirStr := "ns"
	if direction == DirEW { dirStr = "ew" }
	i.Events = append(i.Events, TLEvent{
		Kind: TLGreenExtended, AtMs: i.TotalElapsedMs,
		Detail: fmt.Sprintf("%s +%dms", dirStr, grant),
	})
}

func (i *Intersection) nextPhase() (Phase, TLDirection) {
	switch i.Phase {
	case PhaseNSGreen: return PhaseNSYellow, i.CycleSide
	case PhaseNSYellow: return PhaseAllRed, DirEW
	case PhaseAllRed:
		if i.CycleSide == DirEW { return PhaseEWGreen, DirNS }
		return PhaseNSGreen, DirEW
	case PhaseEWGreen: return PhaseEWYellow, i.CycleSide
	case PhaseEWYellow: return PhaseAllRed, DirNS
	}
	return PhaseEmergencyAllRed, i.CycleSide
}

func (i *Intersection) transitionPhase() bool {
	newPhase, newSide := i.nextPhase()
	from := i.Phase
	i.Phase = newPhase
	i.CycleSide = newSide
	i.PhaseElapsedMs = 0
	i.PhaseDurationMs = i.durationFor(newPhase)
	i.Events = append(i.Events, TLEvent{
		Kind: TLPhaseChanged, AtMs: i.TotalElapsedMs,
		Detail: fmt.Sprintf("%s -> %s", from, newPhase),
	})
	if newPhase == PhaseAllRed && i.PedRequested {
		i.PedRequested = false
		i.pedSt = pedWalk
		i.pedElapsedMs = 0
		i.Events = append(i.Events, TLEvent{
			Kind: TLPedWalkStarted, AtMs: i.TotalElapsedMs,
		})
		return true
	}
	return false
}

func (i *Intersection) advancePedestrian(ms uint64) {
	if i.pedSt == pedInactive { return }
	i.pedElapsedMs += ms
	switch i.pedSt {
	case pedWalk:
		if i.pedElapsedMs >= i.Config.PedWalkMs {
			i.pedSt = pedFlashing
			i.pedElapsedMs = 0
		}
	case pedFlashing:
		if i.pedElapsedMs >= i.Config.PedFlashingMs {
			i.pedSt = pedInactive
			i.pedElapsedMs = 0
			i.Events = append(i.Events, TLEvent{
				Kind: TLPedWalkEnded, AtMs: i.TotalElapsedMs,
			})
		}
	}
}

func (i *Intersection) Tick(ms uint64) {
	i.TotalElapsedMs += ms
	if i.Phase == PhaseEmergencyAllRed { return }
	i.PhaseElapsedMs += ms
	if (i.Phase == PhaseNSGreen || i.Phase == PhaseEWGreen) &&
		i.PhaseElapsedMs >= i.PhaseDurationMs {
		i.considerExtension()
	}
	var pedJustStartedRemaining int64 = -1
	for i.PhaseElapsedMs >= i.PhaseDurationMs && i.Phase != PhaseEmergencyAllRed {
		overflow := i.PhaseElapsedMs - i.PhaseDurationMs
		started := i.transitionPhase()
		if overflow < i.PhaseDurationMs {
			i.PhaseElapsedMs = overflow
		} else {
			i.PhaseElapsedMs = i.PhaseDurationMs
		}
		if started {
			pedJustStartedRemaining = int64(overflow)
		}
	}
	var pedAdvance uint64
	switch {
	case i.pedSt == pedInactive:
		pedAdvance = 0
	case pedJustStartedRemaining >= 0:
		pedAdvance = uint64(pedJustStartedRemaining)
	default:
		pedAdvance = ms
	}
	i.advancePedestrian(pedAdvance)
}

func TrafficLightDemo() {
	cfg := TLConfig{
		NSGreenMs: 1000, NSYellowMs: 200, EWGreenMs: 1000, EWYellowMs: 200,
		AllRedMs: 100, MaxGreenExtensionMs: 500,
		PedWalkMs: 500, PedFlashingMs: 300,
	}
	i := NewIntersection(cfg)
	fmt.Printf("initial: phase=%s ns=%v ew=%v\n", i.Phase, i.NSLight(), i.EWLight())
	i.Tick(1000)
	fmt.Printf("after 1000ms: phase=%s\n", i.Phase)
	i.Tick(200)
	fmt.Printf("after +200ms: phase=%s\n", i.Phase)
	i.Tick(100)
	fmt.Printf("after +100ms: phase=%s\n", i.Phase)

	i2 := NewIntersection(cfg)
	i2.SetNSQueue(3)
	i2.Tick(1000)
	fmt.Printf("with queue: phase=%s duration=%d\n", i2.Phase, i2.PhaseDurationMs)

	i3 := NewIntersection(cfg)
	i3.Tick(300)
	i3.BeginEmergency()
	fmt.Printf("emergency started: phase=%s\n", i3.Phase)
	i3.Tick(10000)
	fmt.Printf("frozen: phase=%s\n", i3.Phase)
	i3.EndEmergency()
	fmt.Printf("resumed: phase=%s elapsed=%d\n", i3.Phase, i3.PhaseElapsedMs)

	i4 := NewIntersection(cfg)
	i4.RequestPedCrossing()
	fmt.Printf("ped requested, signal=%v\n", i4.PedSignalValue())
	i4.Tick(1200)
	fmt.Printf("after 1200ms: phase=%s ped=%v\n", i4.Phase, i4.PedSignalValue())
	i4.Tick(500)
	fmt.Printf("after +500ms: ped=%v\n", i4.PedSignalValue())
	i4.Tick(300)
	fmt.Printf("after +300ms: ped=%v\n", i4.PedSignalValue())
}
