// Mp3 Tagger — file metadata as a queryable, indexed tag store.
package files

import (
	"sort"
	"strings"
)

// Mp3Tag is file metadata. nil-pointer fields mean "not set".
type Mp3Tag struct {
	Title  *string
	Artist *string
	Album  *string
	Year   *int
	Genre  *string
	Track  *int
}

// IsEmpty returns true if no field is set.
func (t *Mp3Tag) IsEmpty() bool {
	return t.Title == nil && t.Artist == nil && t.Album == nil &&
		t.Year == nil && t.Genre == nil && t.Track == nil
}

// MissingFields returns names of nil fields in fixed order.
func (t *Mp3Tag) MissingFields() []string {
	out := []string{}
	if t.Title == nil { out = append(out, "title") }
	if t.Artist == nil { out = append(out, "artist") }
	if t.Album == nil { out = append(out, "album") }
	if t.Year == nil { out = append(out, "year") }
	if t.Genre == nil { out = append(out, "genre") }
	if t.Track == nil { out = append(out, "track") }
	return out
}

// QueryKind tags a query type.
type MTQueryKind int

const (
	MQByArtist     MTQueryKind = 0
	MQByAlbum      MTQueryKind = 1
	MQByYear       MTQueryKind = 2
	MQYearRange    MTQueryKind = 3
	MQTitleContains MTQueryKind = 4
	MQMissingField MTQueryKind = 5
)

// MTQuery is a parameterised query.
type MTQuery struct {
	Kind      MTQueryKind
	Artist    string
	Album     string
	Year      int
	YearFrom  int
	YearTo    int
	Needle    string
	FieldName string
}

// TagStore is the indexed metadata store.
type TagStore struct {
	Tags      map[string]*Mp3Tag
	artistIdx map[string][]string
	albumIdx  map[string][]string
	yearIdx   map[int][]string
}

// NewTagStore returns an empty store.
func NewTagStore() *TagStore {
	return &TagStore{
		Tags:      map[string]*Mp3Tag{},
		artistIdx: map[string][]string{},
		albumIdx:  map[string][]string{},
		yearIdx:   map[int][]string{},
	}
}

// Insert adds or replaces a tag for a path. Keeps indexes fresh.
func (s *TagStore) Insert(path string, tag *Mp3Tag) {
	if _, exists := s.Tags[path]; exists {
		s.removeFromIndexes(path)
	}
	if tag.Artist != nil {
		s.artistIdx[*tag.Artist] = append(s.artistIdx[*tag.Artist], path)
	}
	if tag.Album != nil {
		s.albumIdx[*tag.Album] = append(s.albumIdx[*tag.Album], path)
	}
	if tag.Year != nil {
		s.yearIdx[*tag.Year] = append(s.yearIdx[*tag.Year], path)
	}
	s.Tags[path] = tag
}

// Get returns the tag at a path.
func (s *TagStore) Get(path string) *Mp3Tag { return s.Tags[path] }

// Len returns the number of tagged paths.
func (s *TagStore) Len() int { return len(s.Tags) }

// Remove deletes a tag from store and indexes.
func (s *TagStore) Remove(path string) *Mp3Tag {
	old := s.Tags[path]
	s.removeFromIndexes(path)
	delete(s.Tags, path)
	return old
}

func (s *TagStore) removeFromIndexes(path string) {
	tag := s.Tags[path]
	if tag == nil {
		return
	}
	if tag.Artist != nil {
		s.artistIdx[*tag.Artist] = removeString(s.artistIdx[*tag.Artist], path)
	}
	if tag.Album != nil {
		s.albumIdx[*tag.Album] = removeString(s.albumIdx[*tag.Album], path)
	}
	if tag.Year != nil {
		s.yearIdx[*tag.Year] = removeInt(s.yearIdx[*tag.Year], path)
	}
}

func removeString(list []string, path string) []string {
	out := list[:0]
	for _, p := range list {
		if p != path {
			out = append(out, p)
		}
	}
	return out
}

func removeInt(list []string, path string) []string {
	return removeString(list, path)
}

// Query runs a query and returns sorted matching paths.
func (s *TagStore) Query(q MTQuery) []string {
	var paths []string
	switch q.Kind {
	case MQByArtist:
		paths = append(paths, s.artistIdx[q.Artist]...)
	case MQByAlbum:
		paths = append(paths, s.albumIdx[q.Album]...)
	case MQByYear:
		paths = append(paths, s.yearIdx[q.Year]...)
	case MQYearRange:
		for y, ps := range s.yearIdx {
			if q.YearFrom <= y && y <= q.YearTo {
				paths = append(paths, ps...)
			}
		}
	case MQTitleContains:
		needle := strings.ToLower(q.Needle)
		for p, t := range s.Tags {
			if t.Title != nil && strings.Contains(strings.ToLower(*t.Title), needle) {
				paths = append(paths, p)
			}
		}
	case MQMissingField:
		for p, t := range s.Tags {
			if mp3FieldMissing(t, q.FieldName) {
				paths = append(paths, p)
			}
		}
	}
	sort.Strings(paths)
	return paths
}

func mp3FieldMissing(t *Mp3Tag, name string) bool {
	switch name {
	case "title":  return t.Title == nil
	case "artist": return t.Artist == nil
	case "album":  return t.Album == nil
	case "year":   return t.Year == nil
	case "genre":  return t.Genre == nil
	case "track":  return t.Track == nil
	}
	return false
}
