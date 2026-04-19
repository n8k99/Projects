// Bookmark Collector and Sorter — multi-axis organisation over URL-keyed collection.
package web

import (
	"sort"
	"strings"
)

// BMSortBy selects a sort criterion.
type BMSortBy int

const (
	BMSortTitle BMSortBy = iota
	BMSortURL
	BMSortAddedAt
	BMSortFolder
)

// Bookmark is one URL-keyed entry.
type Bookmark struct {
	ID         int64
	URL        string
	Title      string
	Tags       []string // sorted, deduplicated
	Folder     string
	AddedAtMs  int64
}

// BMCollection is the bookmark collection.
type BMCollection struct {
	bookmarks []*Bookmark
	nextID    int64
}

// NewBMCollection returns an empty collection.
func NewBMCollection() *BMCollection { return &BMCollection{nextID: 1} }

func canonTags(in []string) []string {
	seen := map[string]bool{}
	out := make([]string, 0, len(in))
	for _, t := range in {
		if !seen[t] {
			seen[t] = true
			out = append(out, t)
		}
	}
	sort.Strings(out)
	return out
}

// Add or update a bookmark; URL is the key. Returns the id.
func (c *BMCollection) Add(url, title, folder string, tags []string, addedAtMs int64) int64 {
	for _, b := range c.bookmarks {
		if b.URL == url {
			b.Title = title
			b.Folder = folder
			b.Tags = canonTags(tags)
			return b.ID
		}
	}
	id := c.nextID
	c.nextID++
	c.bookmarks = append(c.bookmarks, &Bookmark{
		ID: id, URL: url, Title: title, Folder: folder,
		Tags: canonTags(tags), AddedAtMs: addedAtMs,
	})
	return id
}

// Remove removes a bookmark by id; returns whether it was present.
func (c *BMCollection) Remove(id int64) bool {
	for i, b := range c.bookmarks {
		if b.ID == id {
			c.bookmarks = append(c.bookmarks[:i], c.bookmarks[i+1:]...)
			return true
		}
	}
	return false
}

// Count returns the number of bookmarks.
func (c *BMCollection) Count() int { return len(c.bookmarks) }

// Get returns a bookmark by id, or nil.
func (c *BMCollection) Get(id int64) *Bookmark {
	for _, b := range c.bookmarks {
		if b.ID == id {
			return b
		}
	}
	return nil
}

// SetTags replaces a bookmark's tag set.
func (c *BMCollection) SetTags(id int64, tags []string) bool {
	b := c.Get(id)
	if b == nil {
		return false
	}
	b.Tags = canonTags(tags)
	return true
}

// MoveToFolder changes a bookmark's folder.
func (c *BMCollection) MoveToFolder(id int64, folder string) bool {
	b := c.Get(id)
	if b == nil {
		return false
	}
	b.Folder = folder
	return true
}

// Search returns bookmarks whose title, url, or any tag contains query (case-insensitive).
func (c *BMCollection) Search(query string) []*Bookmark {
	q := strings.ToLower(query)
	var out []*Bookmark
	for _, b := range c.bookmarks {
		if strings.Contains(strings.ToLower(b.Title), q) ||
			strings.Contains(strings.ToLower(b.URL), q) {
			out = append(out, b)
			continue
		}
		for _, t := range b.Tags {
			if strings.Contains(strings.ToLower(t), q) {
				out = append(out, b)
				break
			}
		}
	}
	return out
}

// ByTag returns bookmarks with the given tag.
func (c *BMCollection) ByTag(tag string) []*Bookmark {
	var out []*Bookmark
	for _, b := range c.bookmarks {
		for _, t := range b.Tags {
			if t == tag {
				out = append(out, b)
				break
			}
		}
	}
	return out
}

// ByFolder returns bookmarks in folder; recursive also matches subfolders.
func (c *BMCollection) ByFolder(folder string, recursive bool) []*Bookmark {
	var out []*Bookmark
	var prefix string
	if recursive {
		if strings.HasSuffix(folder, "/") {
			prefix = folder
		} else {
			prefix = folder + "/"
		}
	}
	for _, b := range c.bookmarks {
		if b.Folder == folder {
			out = append(out, b)
			continue
		}
		if recursive && strings.HasPrefix(b.Folder, prefix) {
			out = append(out, b)
		}
	}
	return out
}

// SortBy returns a stable-sorted view over the collection.
func (c *BMCollection) SortBy(criterion BMSortBy) []*Bookmark {
	view := make([]*Bookmark, len(c.bookmarks))
	copy(view, c.bookmarks)
	sort.SliceStable(view, func(i, j int) bool {
		switch criterion {
		case BMSortTitle:
			return view[i].Title < view[j].Title
		case BMSortURL:
			return view[i].URL < view[j].URL
		case BMSortAddedAt:
			return view[i].AddedAtMs < view[j].AddedAtMs
		case BMSortFolder:
			return view[i].Folder < view[j].Folder
		}
		return false
	})
	return view
}

// AllTags returns all distinct tags, sorted.
func (c *BMCollection) AllTags() []string {
	seen := map[string]bool{}
	for _, b := range c.bookmarks {
		for _, t := range b.Tags {
			seen[t] = true
		}
	}
	out := make([]string, 0, len(seen))
	for t := range seen {
		out = append(out, t)
	}
	sort.Strings(out)
	return out
}

// AllFolders returns all distinct folders, sorted.
func (c *BMCollection) AllFolders() []string {
	seen := map[string]bool{}
	for _, b := range c.bookmarks {
		seen[b.Folder] = true
	}
	out := make([]string, 0, len(seen))
	for f := range seen {
		out = append(out, f)
	}
	sort.Strings(out)
	return out
}
