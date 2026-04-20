// TV Show Tracker — hierarchical show/season/episode state + next-up.
package databases

import "sort"

// TVEpisodeKey uniquely identifies an episode.
type TVEpisodeKey struct {
	ShowID  int64
	Season  int
	Episode int
}

// TVEpisode is one episode record.
type TVEpisode struct {
	ShowID      int64
	Season      int
	Episode     int
	Title       string
	DurationMin int
	AiredMs     int64
}

// Key extracts the hash-ready key.
func (e *TVEpisode) Key() TVEpisodeKey {
	return TVEpisodeKey{e.ShowID, e.Season, e.Episode}
}

// TVShow holds metadata and episodes.
type TVShow struct {
	ID       int64
	Title    string
	Episodes []TVEpisode
}

// TotalEpisodes is just the count.
func (s *TVShow) TotalEpisodes() int { return len(s.Episodes) }

// SortedEpisodes returns copies sorted by (season, episode).
func (s *TVShow) SortedEpisodes() []TVEpisode {
	out := make([]TVEpisode, len(s.Episodes))
	copy(out, s.Episodes)
	sort.Slice(out, func(i, j int) bool {
		if out[i].Season != out[j].Season {
			return out[i].Season < out[j].Season
		}
		return out[i].Episode < out[j].Episode
	})
	return out
}

// TotalRuntimeMin sums all durations.
func (s *TVShow) TotalRuntimeMin() int {
	total := 0
	for _, e := range s.Episodes {
		total += e.DurationMin
	}
	return total
}

// Seasons returns distinct season numbers sorted ascending.
func (s *TVShow) Seasons() []int {
	set := map[int]bool{}
	for _, e := range s.Episodes {
		set[e.Season] = true
	}
	out := make([]int, 0, len(set))
	for n := range set {
		out = append(out, n)
	}
	sort.Ints(out)
	return out
}

// TVLibrary stores all shows by id.
type TVLibrary struct {
	Shows map[int64]*TVShow
}

// NewTVLibrary returns an empty library.
func NewTVLibrary() *TVLibrary {
	return &TVLibrary{Shows: map[int64]*TVShow{}}
}

// AddShow registers a show.
func (lib *TVLibrary) AddShow(show *TVShow) {
	lib.Shows[show.ID] = show
}

// AddEpisode appends or replaces by key.
func (lib *TVLibrary) AddEpisode(ep TVEpisode) error {
	show, ok := lib.Shows[ep.ShowID]
	if !ok {
		return tvErr("unknown show")
	}
	key := ep.Key()
	filtered := show.Episodes[:0]
	for _, existing := range show.Episodes {
		if existing.Key() != key {
			filtered = append(filtered, existing)
		}
	}
	show.Episodes = append(filtered, ep)
	return nil
}

// GetShow returns a show by ID.
func (lib *TVLibrary) GetShow(id int64) *TVShow { return lib.Shows[id] }

// AiringSince returns all episodes aired ≥ sinceMs, sorted.
func (lib *TVLibrary) AiringSince(sinceMs int64) []TVEpisode {
	var out []TVEpisode
	for _, show := range lib.Shows {
		for _, e := range show.Episodes {
			if e.AiredMs >= sinceMs {
				out = append(out, e)
			}
		}
	}
	sort.SliceStable(out, func(i, j int) bool {
		if out[i].AiredMs != out[j].AiredMs {
			return out[i].AiredMs < out[j].AiredMs
		}
		if out[i].ShowID != out[j].ShowID {
			return out[i].ShowID < out[j].ShowID
		}
		if out[i].Season != out[j].Season {
			return out[i].Season < out[j].Season
		}
		return out[i].Episode < out[j].Episode
	})
	return out
}

// TVWatchlist is the user's watched-episodes set.
type TVWatchlist struct {
	Watched map[TVEpisodeKey]bool
}

// NewTVWatchlist returns an empty watchlist.
func NewTVWatchlist() *TVWatchlist {
	return &TVWatchlist{Watched: map[TVEpisodeKey]bool{}}
}

// MarkWatched flags an episode as watched.
func (w *TVWatchlist) MarkWatched(showID int64, season, episode int) {
	w.Watched[TVEpisodeKey{showID, season, episode}] = true
}

// MarkUnwatched removes the flag.
func (w *TVWatchlist) MarkUnwatched(showID int64, season, episode int) {
	delete(w.Watched, TVEpisodeKey{showID, season, episode})
}

// IsWatched tests.
func (w *TVWatchlist) IsWatched(showID int64, season, episode int) bool {
	return w.Watched[TVEpisodeKey{showID, season, episode}]
}

// MarkShowWatched marks every episode in a show.
func (w *TVWatchlist) MarkShowWatched(show *TVShow) {
	for _, e := range show.Episodes {
		w.Watched[e.Key()] = true
	}
}

// MarkSeasonWatched marks every episode in a season.
func (w *TVWatchlist) MarkSeasonWatched(show *TVShow, season int) {
	for _, e := range show.Episodes {
		if e.Season == season {
			w.Watched[e.Key()] = true
		}
	}
}

// WatchedCount for a show.
func (w *TVWatchlist) WatchedCount(showID int64) int {
	n := 0
	for k := range w.Watched {
		if k.ShowID == showID {
			n++
		}
	}
	return n
}

// TVProgress is the computed per-show view.
type TVProgress struct {
	ShowID  int64
	Watched int
	Total   int
	Percent int
}

// ShowProgress rolls up progress.
func ShowProgress(show *TVShow, wl *TVWatchlist) TVProgress {
	total := show.TotalEpisodes()
	watched := wl.WatchedCount(show.ID)
	percent := 0
	if total > 0 {
		percent = watched * 100 / total
	}
	return TVProgress{ShowID: show.ID, Watched: watched, Total: total, Percent: percent}
}

// NextUp returns the earliest unwatched episode.
func NextUp(show *TVShow, wl *TVWatchlist) *TVEpisode {
	for _, e := range show.SortedEpisodes() {
		if !wl.IsWatched(e.ShowID, e.Season, e.Episode) {
			return &e
		}
	}
	return nil
}

// CurrentlyWatching lists shows started but not finished.
func CurrentlyWatching(lib *TVLibrary, wl *TVWatchlist) []*TVShow {
	var out []*TVShow
	for _, show := range lib.Shows {
		w := wl.WatchedCount(show.ID)
		if w > 0 && w < show.TotalEpisodes() {
			out = append(out, show)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].ID < out[j].ID })
	return out
}

type tvError string

func (e tvError) Error() string { return string(e) }
func tvErr(s string) error { return tvError(s) }
