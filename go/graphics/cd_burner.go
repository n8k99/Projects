// CD Burning App — multi-session disc model + FFD packing + ISO names.
package graphics

import (
	"fmt"
	"sort"
	"strings"
	"unicode"
)

const (
	LeadinBytes  int64 = 13_000_000
	LeadoutBytes int64 = 22_000_000
)

type BurnFile struct {
	Name      string
	SizeBytes int64
}

type SessionState int

const (
	SessionOpenState SessionState = iota
	SessionClosedState
)

type Session struct {
	Tracks []BurnFile
	State  SessionState
}

func (s *Session) OverheadBytes() int64 { return LeadinBytes + LeadoutBytes }
func (s *Session) TotalTrackBytes() int64 {
	var t int64
	for _, f := range s.Tracks {
		t += f.SizeBytes
	}
	return t
}
func (s *Session) TotalBytes() int64 { return s.TotalTrackBytes() + s.OverheadBytes() }

type DiscState int

const (
	DiscBlank DiscState = iota
	DiscSessionOpen
	DiscSessionClosed
	DiscFinalized
)

type Disc struct {
	CapacityBytes int64
	Sessions      []*Session
	State         DiscState
}

type BurnError struct{ Reason string }

func (e *BurnError) Error() string { return e.Reason }

func NewDisc(capacity int64) *Disc {
	return &Disc{CapacityBytes: capacity, State: DiscBlank}
}

func (d *Disc) UsedBytes() int64 {
	var t int64
	for _, s := range d.Sessions {
		t += s.TotalBytes()
	}
	return t
}

func (d *Disc) RemainingBytes() int64 {
	r := d.CapacityBytes - d.UsedBytes()
	if r < 0 {
		return 0
	}
	return r
}

func (d *Disc) OpenSession() error {
	switch d.State {
	case DiscFinalized:
		return &BurnError{"disc_finalized"}
	case DiscSessionOpen:
		return &BurnError{"session_already_open"}
	}
	d.Sessions = append(d.Sessions, &Session{State: SessionOpenState})
	d.State = DiscSessionOpen
	return nil
}

func (d *Disc) AddFile(f BurnFile) error {
	if d.State == DiscFinalized {
		return &BurnError{"disc_finalized"}
	}
	if d.State != DiscSessionOpen {
		return &BurnError{"no_open_session"}
	}
	avail := d.RemainingBytes()
	if f.SizeBytes > avail {
		return &BurnError{fmt.Sprintf("not_enough_space needed=%d available=%d", f.SizeBytes, avail)}
	}
	last := d.Sessions[len(d.Sessions)-1]
	last.Tracks = append(last.Tracks, f)
	return nil
}

func (d *Disc) CloseSession() error {
	if d.State == DiscFinalized {
		return &BurnError{"disc_finalized"}
	}
	if d.State != DiscSessionOpen {
		return &BurnError{"no_open_session"}
	}
	d.Sessions[len(d.Sessions)-1].State = SessionClosedState
	d.State = DiscSessionClosed
	return nil
}

func (d *Disc) Finalize() error {
	if d.State == DiscFinalized {
		return &BurnError{"disc_finalized"}
	}
	if d.State == DiscSessionOpen {
		if err := d.CloseSession(); err != nil {
			return err
		}
	}
	d.State = DiscFinalized
	return nil
}

type PackResult struct {
	Discs    [][]BurnFile
	Overflow []BurnFile
}

func PackFilesFFD(files []BurnFile, discCapacity int64, numDiscs int) PackResult {
	sorted := make([]BurnFile, len(files))
	copy(sorted, files)
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].SizeBytes > sorted[j].SizeBytes
	})
	usable := discCapacity - (LeadinBytes + LeadoutBytes)
	if usable < 0 {
		usable = 0
	}
	discs := make([][]BurnFile, numDiscs)
	remaining := make([]int64, numDiscs)
	for i := range remaining {
		remaining[i] = usable
	}
	var overflow []BurnFile
	for _, f := range sorted {
		placed := false
		for i := 0; i < numDiscs; i++ {
			if f.SizeBytes <= remaining[i] {
				remaining[i] -= f.SizeBytes
				discs[i] = append(discs[i], f)
				placed = true
				break
			}
		}
		if !placed {
			overflow = append(overflow, f)
		}
	}
	return PackResult{Discs: discs, Overflow: overflow}
}

func cleanIsoChars(s string) string {
	var b strings.Builder
	for _, r := range s {
		u := unicode.ToUpper(r)
		if (u >= 'A' && u <= 'Z') || (u >= '0' && u <= '9') || u == '_' {
			b.WriteRune(u)
		}
	}
	return b.String()
}

func splitExt(name string) (string, string) {
	i := strings.LastIndex(name, ".")
	if i > 0 {
		return name[:i], name[i+1:]
	}
	return name, ""
}

func CanonicalizeIsoName(name string, existing map[string]bool) string {
	stem, ext := splitExt(name)
	stemClean := cleanIsoChars(stem)
	extClean := cleanIsoChars(ext)
	extTrunc := extClean
	if len(extTrunc) > 3 {
		extTrunc = extTrunc[:3]
	}
	stemBase := stemClean
	if len(stemBase) > 8 {
		stemBase = stemBase[:8]
	}
	base := stemBase
	if extTrunc != "" {
		base = stemBase + "." + extTrunc
	}
	if !existing[base] {
		return base
	}
	for n := 1; n <= 999; n++ {
		suffix := fmt.Sprintf("~%d", n)
		room := 8 - len(suffix)
		if room < 0 {
			room = 0
		}
		short := stemClean
		if len(short) > room {
			short = short[:room]
		}
		var candidate string
		if extTrunc == "" {
			candidate = short + suffix
		} else {
			candidate = short + suffix + "." + extTrunc
		}
		if !existing[candidate] {
			return candidate
		}
	}
	return base
}

func CdBurnerDemo() {
	d := NewDisc(700_000_000)
	d.OpenSession()
	d.AddFile(BurnFile{Name: "album1.flac", SizeBytes: 100_000_000})
	d.AddFile(BurnFile{Name: "album2.flac", SizeBytes: 150_000_000})
	fmt.Printf("after 2 files: used=%d remaining=%d\n", d.UsedBytes(), d.RemainingBytes())
	d.CloseSession()
	fmt.Printf("state: %d\n", d.State)
	d.OpenSession()
	d.AddFile(BurnFile{Name: "bonus.mp3", SizeBytes: 50_000_000})
	fmt.Printf("after session 2: sessions=%d used=%d\n", len(d.Sessions), d.UsedBytes())
	d.Finalize()
	fmt.Printf("final state: %d\n", d.State)

	files := []BurnFile{
		{Name: "big.iso", SizeBytes: 500_000_000},
		{Name: "mid.mp4", SizeBytes: 200_000_000},
		{Name: "small.txt", SizeBytes: 1_000_000},
	}
	result := PackFilesFFD(files, 700_000_000, 2)
	for i, pack := range result.Discs {
		names := []string{}
		for _, f := range pack {
			names = append(names, f.Name)
		}
		fmt.Printf("disc %d: %v\n", i, names)
	}
	overflowNames := []string{}
	for _, f := range result.Overflow {
		overflowNames = append(overflowNames, f.Name)
	}
	fmt.Printf("overflow: %v\n", overflowNames)

	existing := map[string]bool{}
	for _, name := range []string{"Report.Document.Txt", "report.txt", "report.txt", "my file@2026!.txt"} {
		c := CanonicalizeIsoName(name, existing)
		existing[c] = true
		fmt.Printf("  %s -> %s\n", name, c)
	}
}
