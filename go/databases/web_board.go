// Web Board / Forum — threaded posts with vote tallies + hot-score ranking.
package databases

import (
	"fmt"
	"math"
	"sort"
)

// WBThreadSort selects thread ordering.
type WBThreadSort int

const (
	WBSortNew WBThreadSort = 0
	WBSortTop WBThreadSort = 1
	WBSortHot WBThreadSort = 2
)

// WBThread is a top-level thread.
type WBThread struct {
	ID        int64
	Title     string
	Author    string
	CreatedMs int64
	Pinned    bool
	Locked    bool
}

// WBPost is a single post within a thread. ParentID == nil for top-level.
type WBPost struct {
	ID        int64
	ThreadID  int64
	ParentID  *int64
	Author    string
	Body      string
	CreatedMs int64
	Upvotes   int64
	Downvotes int64
}

// Score returns upvotes - downvotes.
func (p *WBPost) Score() int64 { return p.Upvotes - p.Downvotes }

// WBHotScore is the Reddit-style ranking: log-magnitude - age penalty.
func WBHotScore(p *WBPost, nowMs int64) float64 {
	s := p.Score()
	var sign float64
	if s > 0 { sign = 1 } else if s < 0 { sign = -1 }
	abs := s
	if abs < 0 { abs = -abs }
	if abs < 1 { abs = 1 }
	magnitude := math.Log10(float64(abs))
	ageDays := float64(nowMs-p.CreatedMs) / 86_400_000.0
	if ageDays < 0 { ageDays = 0 }
	return sign*magnitude - ageDays*0.2
}

// WBPostWithDepth pairs a post with its depth in the reply tree.
type WBPostWithDepth struct {
	Post  *WBPost
	Depth int
}

// WBBoard holds threads and posts.
type WBBoard struct {
	Threads map[int64]*WBThread
	Posts   map[int64]*WBPost
}

// NewWBBoard returns an empty board.
func NewWBBoard() *WBBoard {
	return &WBBoard{
		Threads: map[int64]*WBThread{},
		Posts:   map[int64]*WBPost{},
	}
}

// AddThread registers a thread.
func (b *WBBoard) AddThread(t *WBThread) { b.Threads[t.ID] = t }

// AddPost appends a post after validation.
func (b *WBBoard) AddPost(p *WBPost) error {
	if b.Threads[p.ThreadID] == nil {
		return fmt.Errorf("unknown thread %d", p.ThreadID)
	}
	if p.ParentID != nil {
		parent := b.Posts[*p.ParentID]
		if parent == nil {
			return fmt.Errorf("unknown parent %d", *p.ParentID)
		}
		if parent.ThreadID != p.ThreadID {
			return fmt.Errorf("parent %d is in a different thread", *p.ParentID)
		}
	}
	b.Posts[p.ID] = p
	return nil
}

// ThreadScore is the sum of per-post scores.
func (b *WBBoard) ThreadScore(threadID int64) int64 {
	var sum int64
	for _, p := range b.Posts {
		if p.ThreadID == threadID { sum += p.Score() }
	}
	return sum
}

// ThreadHotScore is the sum of per-post hot scores.
func (b *WBBoard) ThreadHotScore(threadID int64, nowMs int64) float64 {
	sum := 0.0
	for _, p := range b.Posts {
		if p.ThreadID == threadID { sum += WBHotScore(p, nowMs) }
	}
	return sum
}

// TopLevelPosts returns the root posts in creation order.
func (b *WBBoard) TopLevelPosts(threadID int64) []*WBPost {
	var out []*WBPost
	for _, p := range b.Posts {
		if p.ThreadID == threadID && p.ParentID == nil {
			out = append(out, p)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].CreatedMs < out[j].CreatedMs })
	return out
}

// ChildrenOf returns direct replies to a post in creation order.
func (b *WBBoard) ChildrenOf(postID int64) []*WBPost {
	var out []*WBPost
	for _, p := range b.Posts {
		if p.ParentID != nil && *p.ParentID == postID {
			out = append(out, p)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].CreatedMs < out[j].CreatedMs })
	return out
}

// ThreadTree DFS-walks the thread producing (post, depth) pairs.
func (b *WBBoard) ThreadTree(threadID int64) []WBPostWithDepth {
	var out []WBPostWithDepth
	for _, top := range b.TopLevelPosts(threadID) {
		b.walkSubtree(top, 0, &out)
	}
	return out
}

func (b *WBBoard) walkSubtree(post *WBPost, depth int, out *[]WBPostWithDepth) {
	*out = append(*out, WBPostWithDepth{Post: post, Depth: depth})
	for _, child := range b.ChildrenOf(post.ID) {
		b.walkSubtree(child, depth+1, out)
	}
}

// PostCount returns the number of posts in a thread.
func (b *WBBoard) PostCount(threadID int64) int {
	n := 0
	for _, p := range b.Posts {
		if p.ThreadID == threadID { n++ }
	}
	return n
}

// ListThreads returns threads sorted; pinned always first.
func (b *WBBoard) ListThreads(s WBThreadSort, nowMs int64) []*WBThread {
	out := make([]*WBThread, 0, len(b.Threads))
	for _, t := range b.Threads {
		out = append(out, t)
	}
	sort.SliceStable(out, func(i, j int) bool {
		a, b2 := out[i], out[j]
		if a.Pinned && !b2.Pinned { return true }
		if !a.Pinned && b2.Pinned { return false }
		switch s {
		case WBSortNew:
			return a.CreatedMs > b2.CreatedMs
		case WBSortTop:
			return b.ThreadScore(a.ID) > b.ThreadScore(b2.ID)
		case WBSortHot:
			return b.ThreadHotScore(a.ID, nowMs) > b.ThreadHotScore(b2.ID, nowMs)
		}
		return false
	})
	return out
}
