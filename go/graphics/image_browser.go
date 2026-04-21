// Image Browser — 2D grid navigation + LRU cache + filter/sort query.
package graphics

import (
	"container/list"
	"fmt"
	"sort"
	"strings"
)

type ImageEntry struct {
	ID         uint64
	Name       string
	Path       string
	Width      uint32
	Height     uint32
	SizeBytes  uint64
	Tags       []string
	CreatedMs  uint64
}

type FilterKind int

const (
	FilterNameContains FilterKind = iota
	FilterTagHas
	FilterSizeUnder
	FilterSizeOver
	FilterWidthAtLeast
	FilterCreatedAfter
)

type FilterPredicate struct {
	Kind FilterKind
	S    string
	N    uint64
}

func (p FilterPredicate) Matches(e *ImageEntry) bool {
	switch p.Kind {
	case FilterNameContains:
		return strings.Contains(e.Name, p.S)
	case FilterTagHas:
		for _, t := range e.Tags {
			if t == p.S { return true }
		}
		return false
	case FilterSizeUnder:
		return e.SizeBytes < p.N
	case FilterSizeOver:
		return e.SizeBytes > p.N
	case FilterWidthAtLeast:
		return uint64(e.Width) >= p.N
	case FilterCreatedAfter:
		return e.CreatedMs > p.N
	}
	return false
}

func FilterNameContainsNew(s string) FilterPredicate { return FilterPredicate{Kind: FilterNameContains, S: s} }
func FilterTagHasNew(s string) FilterPredicate { return FilterPredicate{Kind: FilterTagHas, S: s} }
func FilterSizeOverNew(n uint64) FilterPredicate { return FilterPredicate{Kind: FilterSizeOver, N: n} }
func FilterSizeUnderNew(n uint64) FilterPredicate { return FilterPredicate{Kind: FilterSizeUnder, N: n} }

type SortByKind int

const (
	SortByName SortByKind = iota
	SortBySize
	SortByCreated
	SortByWidth
)

type OrderKind int

const (
	OrderAsc OrderKind = iota
	OrderDesc
)

type Query struct {
	Filters []FilterPredicate
	SortBy  SortByKind
	Order   OrderKind
}

func ApplyQuery(entries []*ImageEntry, query Query) []*ImageEntry {
	filtered := make([]*ImageEntry, 0, len(entries))
	for _, e := range entries {
		ok := true
		for _, p := range query.Filters {
			if !p.Matches(e) { ok = false; break }
		}
		if ok { filtered = append(filtered, e) }
	}
	sort.SliceStable(filtered, func(i, j int) bool {
		a, b := filtered[i], filtered[j]
		var less bool
		switch query.SortBy {
		case SortByName: less = a.Name < b.Name
		case SortBySize: less = a.SizeBytes < b.SizeBytes
		case SortByCreated: less = a.CreatedMs < b.CreatedMs
		case SortByWidth: less = a.Width < b.Width
		}
		if query.Order == OrderDesc { return !less && a != b }
		return less
	})
	return filtered
}

type GridMoveKind int

const (
	GridLeft GridMoveKind = iota
	GridRight
	GridUp
	GridDown
	GridHome
	GridEnd
)

type IBGrid struct {
	Cols     uint32
	Total    uint32
	Selected uint32
}

func NewIBGrid(cols, total uint32) *IBGrid { return &IBGrid{Cols: cols, Total: total} }

func (g *IBGrid) Rows() uint32 {
	if g.Cols == 0 || g.Total == 0 { return 0 }
	return (g.Total + g.Cols - 1) / g.Cols
}

func (g *IBGrid) RowCol() (uint32, uint32) {
	return g.Selected / g.Cols, g.Selected % g.Cols
}

func (g *IBGrid) Move(mv GridMoveKind) {
	if g.Total == 0 { return }
	row, col := g.RowCol()
	switch mv {
	case GridLeft:
		if g.Selected > 0 { g.Selected-- }
	case GridRight:
		if g.Selected+1 < g.Total { g.Selected++ }
	case GridUp:
		if row > 0 { g.Selected = (row-1)*g.Cols + col }
	case GridDown:
		next := (row + 1) * g.Cols + col
		if next < g.Total { g.Selected = next
		} else if row+1 < g.Rows() {
			g.Selected = g.Total - 1
		}
	case GridHome:
		g.Selected = 0
	case GridEnd:
		g.Selected = g.Total - 1
	}
}

type LruCache struct {
	Capacity int
	order    *list.List
	lookup   map[uint64]*list.Element
	values   map[uint64][]byte
}

type lruItem struct {
	key uint64
}

func NewLruCache(cap int) *LruCache {
	return &LruCache{
		Capacity: cap,
		order:    list.New(),
		lookup:   map[uint64]*list.Element{},
		values:   map[uint64][]byte{},
	}
}

func (c *LruCache) Get(k uint64) ([]byte, bool) {
	e, ok := c.lookup[k]
	if !ok { return nil, false }
	c.order.MoveToBack(e)
	return c.values[k], true
}

// Put returns (evicted_key, true) if an eviction occurred.
func (c *LruCache) Put(k uint64, v []byte) (uint64, bool) {
	if e, ok := c.lookup[k]; ok {
		c.order.MoveToBack(e)
		c.values[k] = v
		return 0, false
	}
	if len(c.values) >= c.Capacity {
		front := c.order.Front()
		if front != nil {
			evictedKey := front.Value.(*lruItem).key
			c.order.Remove(front)
			delete(c.lookup, evictedKey)
			delete(c.values, evictedKey)
			c.values[k] = v
			elem := c.order.PushBack(&lruItem{key: k})
			c.lookup[k] = elem
			return evictedKey, true
		}
	}
	c.values[k] = v
	elem := c.order.PushBack(&lruItem{key: k})
	c.lookup[k] = elem
	return 0, false
}

func (c *LruCache) Len() int { return len(c.values) }
func (c *LruCache) Contains(k uint64) bool { _, ok := c.lookup[k]; return ok }

type Browser struct {
	GridCols       uint32
	Breadcrumbs    []string
	AllEntries     []*ImageEntry
	Query          Query
	Grid           *IBGrid
	ThumbnailCache *LruCache
}

func NewBrowser(gridCols uint32, thumbnailCap int) *Browser {
	return &Browser{
		GridCols:       gridCols,
		Grid:           NewIBGrid(gridCols, 0),
		ThumbnailCache: NewLruCache(thumbnailCap),
	}
}

func (b *Browser) SetEntries(entries []*ImageEntry) {
	b.AllEntries = entries
	filtered := ApplyQuery(b.AllEntries, b.Query)
	b.Grid = NewIBGrid(b.GridCols, uint32(len(filtered)))
}

func (b *Browser) SetQuery(q Query) {
	b.Query = q
	filtered := ApplyQuery(b.AllEntries, b.Query)
	b.Grid = NewIBGrid(b.GridCols, uint32(len(filtered)))
}

func (b *Browser) FilteredEntries() []*ImageEntry {
	return ApplyQuery(b.AllEntries, b.Query)
}

func (b *Browser) SelectedEntry() *ImageEntry {
	filtered := b.FilteredEntries()
	idx := int(b.Grid.Selected)
	if idx >= 0 && idx < len(filtered) { return filtered[idx] }
	return nil
}

func (b *Browser) NavigateInto(subdir string) {
	b.Breadcrumbs = append(b.Breadcrumbs, subdir)
}

func (b *Browser) NavigateUp() bool {
	if len(b.Breadcrumbs) == 0 { return false }
	b.Breadcrumbs = b.Breadcrumbs[:len(b.Breadcrumbs)-1]
	return true
}

func (b *Browser) CurrentPath() string {
	return "/" + strings.Join(b.Breadcrumbs, "/")
}

func (b *Browser) MoveCursor(mv GridMoveKind) {
	b.Grid.Move(mv)
}

func (b *Browser) GetOrLoadThumbnail(id uint64, producer func() []byte) ([]byte, bool) {
	if v, ok := b.ThumbnailCache.Get(id); ok {
		return v, false
	}
	v := producer()
	b.ThumbnailCache.Put(id, v)
	return v, true
}

func ImageBrowserDemo() {
	br := NewBrowser(3, 3)
	entries := []*ImageEntry{
		{ID: 1, Name: "alpha.jpg", Path: "/img/alpha.jpg", Width: 1920, Height: 1080, SizeBytes: 200_000, Tags: []string{"nature"}, CreatedMs: 1000},
		{ID: 2, Name: "beta.jpg", Path: "/img/beta.jpg", Width: 800, Height: 600, SizeBytes: 50_000, Tags: []string{"abstract"}, CreatedMs: 2000},
		{ID: 3, Name: "gamma.png", Path: "/img/gamma.png", Width: 1024, Height: 768, SizeBytes: 150_000, Tags: []string{"nature"}, CreatedMs: 3000},
		{ID: 4, Name: "delta.jpg", Path: "/img/delta.jpg", Width: 2560, Height: 1440, SizeBytes: 500_000, Tags: []string{"nature"}, CreatedMs: 4000},
		{ID: 5, Name: "epsilon.png", Path: "/img/epsilon.png", Width: 1920, Height: 1080, SizeBytes: 100_000, Tags: []string{"abstract"}, CreatedMs: 5000},
	}
	br.SetEntries(entries)
	fmt.Printf("all: %d; grid total: %d\n", len(br.FilteredEntries()), br.Grid.Total)

	br.SetQuery(Query{
		Filters: []FilterPredicate{FilterNameContainsNew(".jpg"), FilterTagHasNew("nature")},
		SortBy: SortBySize, Order: OrderAsc,
	})
	names := []string{}
	for _, e := range br.FilteredEntries() { names = append(names, e.Name) }
	fmt.Printf("nature jpg by size asc: %v\n", names)
	fmt.Printf("grid total: %d\n", br.Grid.Total)

	fmt.Printf("selected: %s\n", br.SelectedEntry().Name)
	br.MoveCursor(GridRight)
	fmt.Printf("after right: %s\n", br.SelectedEntry().Name)
	br.MoveCursor(GridRight)
	fmt.Printf("after right at end: %s\n", br.SelectedEntry().Name)

	br.NavigateInto("photos")
	br.NavigateInto("2026")
	fmt.Printf("path: %s\n", br.CurrentPath())
	br.NavigateUp()
	fmt.Printf("after up: %s\n", br.CurrentPath())

	calls := 0
	producer := func() []byte { calls++; return []byte("bytes") }
	_, miss1 := br.GetOrLoadThumbnail(1, producer)
	_, miss2 := br.GetOrLoadThumbnail(1, producer)
	_, miss3 := br.GetOrLoadThumbnail(2, producer)
	fmt.Printf("cache miss: %v hit: %v miss2: %v\n", miss1, !miss2, miss3)
	fmt.Printf("producer calls: %d\n", calls)

	lru := NewLruCache(2)
	lru.Put(1, []byte("a"))
	lru.Put(2, []byte("b"))
	lru.Get(1)
	evicted, _ := lru.Put(3, []byte("c"))
	fmt.Printf("LRU evicted key: %d\n", evicted)
}
