-- TV Show Tracker — hierarchical show/season/episode state + next-up.

namespace Databases.TV

structure EpisodeKey where
  showId : Int
  season : Nat
  episode : Nat
  deriving Repr, BEq

structure Episode where
  showId : Int
  season : Nat
  episode : Nat
  title : String
  durationMin : Nat
  airedMs : Int
  deriving Repr

def Episode.key (e : Episode) : EpisodeKey :=
  { showId := e.showId, season := e.season, episode := e.episode }

structure Show where
  id : Int
  title : String
  episodes : List Episode := []
  deriving Repr

def Show.totalEpisodes (s : Show) : Nat := s.episodes.length

def Show.sortedEpisodes (s : Show) : List Episode :=
  s.episodes.mergeSort fun a b =>
    if a.season ≠ b.season then a.season < b.season else a.episode < b.episode

def Show.totalRuntimeMin (s : Show) : Nat :=
  s.episodes.foldl (init := 0) fun acc e => acc + e.durationMin

def Show.seasons (s : Show) : List Nat :=
  (s.episodes.map Episode.season).eraseDups.mergeSort (· < ·)

structure Library where
  shows : List (Int × Show) := []
  deriving Repr

def Library.addShow (lib : Library) (s : Show) : Library :=
  { shows := (lib.shows.filter (fun p => p.1 ≠ s.id)) ++ [(s.id, s)] }

def Library.get (lib : Library) (id : Int) : Option Show :=
  (lib.shows.find? (fun p => p.1 == id)).map Prod.snd

def Library.airingSince (lib : Library) (sinceMs : Int) : List Episode :=
  let all := lib.shows.bind (fun p => p.2.episodes)
  let filtered := all.filter (fun e => e.airedMs ≥ sinceMs)
  filtered.mergeSort fun a b =>
    if a.airedMs ≠ b.airedMs then a.airedMs < b.airedMs
    else if a.showId ≠ b.showId then a.showId < b.showId
    else if a.season ≠ b.season then a.season < b.season
    else a.episode < b.episode

structure Watchlist where
  watched : List EpisodeKey := []
  deriving Repr

def Watchlist.markWatched (wl : Watchlist) (showId : Int) (season episode : Nat) : Watchlist :=
  let k : EpisodeKey := { showId, season, episode }
  if wl.watched.contains k then wl
  else { watched := wl.watched ++ [k] }

def Watchlist.markUnwatched (wl : Watchlist) (showId : Int) (season episode : Nat) : Watchlist :=
  let k : EpisodeKey := { showId, season, episode }
  { watched := wl.watched.filter (fun x => !(x == k)) }

def Watchlist.isWatched (wl : Watchlist) (showId : Int) (season episode : Nat) : Bool :=
  wl.watched.contains { showId, season, episode }

def Watchlist.markShowWatched (wl : Watchlist) (show : Show) : Watchlist :=
  show.episodes.foldl (init := wl) fun acc e =>
    acc.markWatched e.showId e.season e.episode

def Watchlist.markSeasonWatched (wl : Watchlist) (show : Show) (season : Nat) : Watchlist :=
  show.episodes.foldl (init := wl) fun acc e =>
    if e.season == season then acc.markWatched e.showId e.season e.episode else acc

def Watchlist.watchedCount (wl : Watchlist) (showId : Int) : Nat :=
  (wl.watched.filter (fun k => k.showId == showId)).length

structure Progress where
  showId : Int
  watched : Nat
  total : Nat
  percent : Nat
  deriving Repr

def showProgress (s : Show) (wl : Watchlist) : Progress :=
  let total := s.totalEpisodes
  let w := wl.watchedCount s.id
  let percent := if total == 0 then 0 else w * 100 / total
  { showId := s.id, watched := w, total := total, percent }

def nextUp (s : Show) (wl : Watchlist) : Option Episode :=
  s.sortedEpisodes.find? fun e => !wl.isWatched e.showId e.season e.episode

def currentlyWatching (lib : Library) (wl : Watchlist) : List Show :=
  let out := lib.shows.filterMap fun (_, s) =>
    let w := wl.watchedCount s.id
    if w > 0 ∧ w < s.totalEpisodes then some s else none
  out.mergeSort (fun a b => a.id < b.id)

end Databases.TV
