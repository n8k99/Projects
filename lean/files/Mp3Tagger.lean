-- Mp3 Tagger — file metadata as a queryable, indexed tag store.

namespace Files.MP3T

structure Tag where
  title : Option String := none
  artist : Option String := none
  album : Option String := none
  year : Option Nat := none
  genre : Option String := none
  track : Option Nat := none
  deriving Repr

def Tag.isEmpty (t : Tag) : Bool :=
  t.title.isNone ∧ t.artist.isNone ∧ t.album.isNone ∧
  t.year.isNone ∧ t.genre.isNone ∧ t.track.isNone

def Tag.missingFields (t : Tag) : List String :=
  let mut out : List String := []
  if t.title.isNone then out := out ++ ["title"]
  if t.artist.isNone then out := out ++ ["artist"]
  if t.album.isNone then out := out ++ ["album"]
  if t.year.isNone then out := out ++ ["year"]
  if t.genre.isNone then out := out ++ ["genre"]
  if t.track.isNone then out := out ++ ["track"]
  out

inductive Query where
  | byArtist (artist : String)
  | byAlbum (album : String)
  | byYear (year : Nat)
  | yearRange (from' : Nat) (to' : Nat)
  | titleContains (needle : String)
  | missingField (name : String)
  deriving Repr

structure TagStore where
  tags : List (String × Tag) := []
  deriving Repr

def TagStore.insert (s : TagStore) (path : String) (tag : Tag) : TagStore :=
  let filtered := s.tags.filter (fun p => p.1 ≠ path)
  { tags := filtered ++ [(path, tag)] }

def TagStore.get (s : TagStore) (path : String) : Option Tag :=
  s.tags.find? (fun p => p.1 == path) |>.map (·.2)

def TagStore.remove (s : TagStore) (path : String) : TagStore :=
  { tags := s.tags.filter (fun p => p.1 ≠ path) }

def fieldMissing (t : Tag) (name : String) : Bool :=
  match name with
  | "title" => t.title.isNone
  | "artist" => t.artist.isNone
  | "album" => t.album.isNone
  | "year" => t.year.isNone
  | "genre" => t.genre.isNone
  | "track" => t.track.isNone
  | _ => false

def TagStore.query (s : TagStore) (q : Query) : List String :=
  let matches := s.tags.filterMap fun (path, t) =>
    match q with
    | .byArtist a => if t.artist == some a then some path else none
    | .byAlbum a => if t.album == some a then some path else none
    | .byYear y => if t.year == some y then some path else none
    | .yearRange from' to' =>
      match t.year with
      | some y => if from' ≤ y ∧ y ≤ to' then some path else none
      | none => none
    | .titleContains needle =>
      match t.title with
      | some title =>
        if (title.toLower.splitOn needle.toLower).length > 1 ∨
           title.toLower == needle.toLower
        then some path else none
      | none => none
    | .missingField name =>
      if fieldMissing t name then some path else none
  matches.mergeSort (· < ·)

end Files.MP3T
