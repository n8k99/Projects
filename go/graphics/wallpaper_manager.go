// Wallpaper Manager — multi-monitor rotation + resolution best-fit + recency.
package graphics

import (
	"fmt"
	"math"
)

type WmResolution struct{ Width, Height uint32 }

func (r WmResolution) AspectRatio() float64 {
	return float64(r.Width) / float64(r.Height)
}

func (r WmResolution) PixelCount() uint64 {
	return uint64(r.Width) * uint64(r.Height)
}

type WmMonitor struct {
	ID         uint32
	Resolution WmResolution
}

type WallpaperVariant struct {
	Resolution WmResolution
	Path       string
}

type Wallpaper struct {
	ID       uint32
	Name     string
	Tags     []string
	Variants []WallpaperVariant
}

func ResolutionFitScore(monitor, variant WmResolution) int64 {
	if monitor == variant {
		return 1000
	}
	mAr := monitor.AspectRatio()
	vAr := variant.AspectRatio()
	arDiff := math.Abs(mAr - vAr)
	if arDiff > 0.5 {
		arDiff = 0.5
	}
	arScore := int64((0.5 - arDiff) / 0.5 * 500)

	mPx := float64(monitor.PixelCount())
	vPx := float64(variant.PixelCount())
	ratio := vPx / mPx
	var pxScore int64
	if ratio >= 1.0 {
		denom := math.Max(1.0, math.Min(2.0, math.Max(1.0, math.Log2(ratio))))
		pxScore = int64(300.0 / denom)
	} else {
		pxScore = int64(math.Max(0.0, ratio-0.5) * 600.0)
	}
	return arScore + pxScore
}

func BestVariantForMonitor(w *Wallpaper, monitor WmResolution) *WallpaperVariant {
	if len(w.Variants) == 0 {
		return nil
	}
	best := &w.Variants[0]
	bestScore := ResolutionFitScore(monitor, best.Resolution)
	for i := 1; i < len(w.Variants); i++ {
		s := ResolutionFitScore(monitor, w.Variants[i].Resolution)
		if s > bestScore {
			best = &w.Variants[i]
			bestScore = s
		}
	}
	return best
}

type ScheduleKind int

const (
	ScheduleInterval ScheduleKind = iota
	ScheduleFixedList
	ScheduleTagWindow
)

type WmSchedule struct {
	Kind              ScheduleKind
	IntervalMs        uint64
	RotationTimesMs   []uint64
	Tag               string
	StartMs           uint64
	EndMs             uint64
	InnerMs           uint64
}

func ScheduleInt(ms uint64) WmSchedule {
	return WmSchedule{Kind: ScheduleInterval, IntervalMs: ms}
}
func ScheduleFixed(times []uint64) WmSchedule {
	return WmSchedule{Kind: ScheduleFixedList, RotationTimesMs: times}
}
func ScheduleTagWin(tag string, start, end, inner uint64) WmSchedule {
	return WmSchedule{Kind: ScheduleTagWindow, Tag: tag,
		StartMs: start, EndMs: end, InnerMs: inner}
}

type WmEventKind int

const (
	WmEventRotated WmEventKind = iota
	WmEventSkippedNoEligible
)

type WmManagerEvent struct {
	Kind        WmEventKind
	MonitorID   uint32
	AtMs        uint64
	WallpaperID uint32
}

type MonitorState struct {
	CurrentWallpaperID *uint32
	LastRotationMs     uint64
	History            []uint32
}

type WmManager struct {
	Schedule       WmSchedule
	RecencyWindow  int
	LcgState       uint64
	Monitors       []WmMonitor
	Wallpapers     []*Wallpaper
	Assignments    map[uint32]*MonitorState
	ElapsedMs      uint64
	Events         []WmManagerEvent
}

const (
	wmLcgMul uint64 = 6364136223846793005
	wmLcgInc uint64 = 1442695040888963407
)

func NewWmManager(schedule WmSchedule, recency int, seed uint64) *WmManager {
	return &WmManager{
		Schedule: schedule, RecencyWindow: recency,
		LcgState: seed + 1, Assignments: map[uint32]*MonitorState{},
	}
}

func (m *WmManager) AddMonitor(mon WmMonitor) {
	m.Monitors = append(m.Monitors, mon)
	m.Assignments[mon.ID] = &MonitorState{}
}

func (m *WmManager) AddWallpaper(w *Wallpaper) {
	m.Wallpapers = append(m.Wallpapers, w)
}

func (m *WmManager) lcgNext() uint64 {
	m.LcgState = m.LcgState*wmLcgMul + wmLcgInc
	return m.LcgState
}

func (m *WmManager) matchesTag(w *Wallpaper, tag string) bool {
	if tag == "" {
		return true
	}
	for _, t := range w.Tags {
		if t == tag {
			return true
		}
	}
	return false
}

func containsU32(xs []uint32, x uint32) bool {
	for _, v := range xs {
		if v == x {
			return true
		}
	}
	return false
}

func (m *WmManager) PickNextForMonitor(monitorID uint32, tagFilter string) (uint32, bool) {
	state := m.Assignments[monitorID]
	// Take last N from history, reversed
	recent := []uint32{}
	for i := len(state.History) - 1; i >= 0 && len(recent) < m.RecencyWindow; i-- {
		recent = append(recent, state.History[i])
	}

	var eligible []uint32
	for _, w := range m.Wallpapers {
		if !containsU32(recent, w.ID) && m.matchesTag(w, tagFilter) {
			eligible = append(eligible, w.ID)
		}
	}
	if len(eligible) == 0 {
		for _, w := range m.Wallpapers {
			if m.matchesTag(w, tagFilter) {
				eligible = append(eligible, w.ID)
			}
		}
		if len(eligible) == 0 {
			return 0, false
		}
	}
	r := m.lcgNext()
	return eligible[int(r%uint64(len(eligible)))], true
}

func (m *WmManager) RotateMonitor(monitorID uint32, tagFilter string) bool {
	pick, ok := m.PickNextForMonitor(monitorID, tagFilter)
	if !ok {
		m.Events = append(m.Events, WmManagerEvent{
			Kind: WmEventSkippedNoEligible, MonitorID: monitorID, AtMs: m.ElapsedMs,
		})
		return false
	}
	state := m.Assignments[monitorID]
	state.CurrentWallpaperID = &pick
	state.LastRotationMs = m.ElapsedMs
	state.History = append(state.History, pick)
	m.Events = append(m.Events, WmManagerEvent{
		Kind: WmEventRotated, MonitorID: monitorID, AtMs: m.ElapsedMs, WallpaperID: pick,
	})
	return true
}

func (m *WmManager) shouldRotate(monitorID uint32) bool {
	state := m.Assignments[monitorID]
	switch m.Schedule.Kind {
	case ScheduleInterval:
		return state.CurrentWallpaperID == nil ||
			m.ElapsedMs-state.LastRotationMs >= m.Schedule.IntervalMs
	case ScheduleFixedList:
		for _, t := range m.Schedule.RotationTimesMs {
			if state.LastRotationMs < t && t <= m.ElapsedMs {
				return true
			}
		}
		return false
	case ScheduleTagWindow:
		return state.CurrentWallpaperID == nil ||
			m.ElapsedMs-state.LastRotationMs >= m.Schedule.InnerMs
	}
	return false
}

func (m *WmManager) activeTag() string {
	if m.Schedule.Kind != ScheduleTagWindow {
		return ""
	}
	dayMs := uint64(24 * 60 * 60 * 1000)
	inDay := m.ElapsedMs % dayMs
	if inDay >= m.Schedule.StartMs && inDay < m.Schedule.EndMs {
		return m.Schedule.Tag
	}
	return ""
}

func (m *WmManager) Tick(ms uint64) {
	m.ElapsedMs += ms
	for _, mon := range m.Monitors {
		if m.shouldRotate(mon.ID) {
			m.RotateMonitor(mon.ID, m.activeTag())
		}
	}
}

func (m *WmManager) CurrentVariantForMonitor(monitorID uint32) *WallpaperVariant {
	state, ok := m.Assignments[monitorID]
	if !ok || state.CurrentWallpaperID == nil {
		return nil
	}
	var monitor *WmMonitor
	for i := range m.Monitors {
		if m.Monitors[i].ID == monitorID {
			monitor = &m.Monitors[i]
			break
		}
	}
	var wallpaper *Wallpaper
	for _, w := range m.Wallpapers {
		if w.ID == *state.CurrentWallpaperID {
			wallpaper = w
			break
		}
	}
	if monitor == nil || wallpaper == nil {
		return nil
	}
	return BestVariantForMonitor(wallpaper, monitor.Resolution)
}

func WallpaperManagerDemo() {
	mgr := NewWmManager(ScheduleInt(1000), 2, 42)
	mgr.AddMonitor(WmMonitor{ID: 1, Resolution: WmResolution{1920, 1080}})
	mgr.AddMonitor(WmMonitor{ID: 2, Resolution: WmResolution{2560, 1440}})

	specs := []struct {
		id   uint32
		tags []string
	}{
		{10, []string{"nature"}}, {11, []string{"abstract"}},
		{12, []string{"nature"}}, {13, []string{"abstract"}},
		{14, []string{"nature"}},
	}
	for _, s := range specs {
		mgr.AddWallpaper(&Wallpaper{
			ID: s.id, Name: fmt.Sprintf("wp%d", s.id), Tags: s.tags,
			Variants: []WallpaperVariant{
				{Resolution: WmResolution{1920, 1080}, Path: fmt.Sprintf("/wp/%d/1080.png", s.id)},
				{Resolution: WmResolution{2560, 1440}, Path: fmt.Sprintf("/wp/%d/1440.png", s.id)},
			},
		})
	}

	mgr.Tick(1)
	fmt.Printf("t=1ms: m1=%d m2=%d\n",
		*mgr.Assignments[1].CurrentWallpaperID,
		*mgr.Assignments[2].CurrentWallpaperID)
	mgr.Tick(1100)
	fmt.Printf("t=1101ms: m1=%d m2=%d\n",
		*mgr.Assignments[1].CurrentWallpaperID,
		*mgr.Assignments[2].CurrentWallpaperID)
	mgr.Tick(1100)
	fmt.Printf("t=2201ms: m1=%d m2=%d\n",
		*mgr.Assignments[1].CurrentWallpaperID,
		*mgr.Assignments[2].CurrentWallpaperID)

	v := mgr.CurrentVariantForMonitor(2)
	fmt.Printf("m2 variant: %dx%d\n", v.Resolution.Width, v.Resolution.Height)

	m1080 := WmResolution{1920, 1080}
	fmt.Printf("score 1920x1080 vs 1920x1080: %d\n", ResolutionFitScore(m1080, WmResolution{1920, 1080}))
	fmt.Printf("score 1920x1080 vs 1280x720:  %d\n", ResolutionFitScore(m1080, WmResolution{1280, 720}))
	fmt.Printf("score 1920x1080 vs 2000x1500: %d\n", ResolutionFitScore(m1080, WmResolution{2000, 1500}))

	fmt.Printf("history m1: %v\n", mgr.Assignments[1].History)
	fmt.Printf("history m2: %v\n", mgr.Assignments[2].History)
}
