-- Bookmark Collector and Sorter — multi-axis organisation over URL-keyed collection.
-- Pure-functional: every operation returns a new BMCollection.

namespace Web.BM

inductive SortBy where
  | title | url | addedAt | folder
  deriving Repr, DecidableEq

structure Bookmark where
  id : Nat
  url : String
  title : String
  tags : List String := []          -- sorted, deduplicated
  folder : String := "/"
  addedAtMs : Nat := 0
  deriving Repr

structure Collection where
  bookmarks : List Bookmark := []
  nextId : Nat := 1
  deriving Repr

def Collection.empty : Collection := {}

private def canonTags (ts : List String) : List String :=
  ts.eraseDups.mergeSort (· < ·)

/-- Add a bookmark; if URL already exists, update and return existing id. -/
def Collection.add (c : Collection) (url title folder : String)
    (tags : List String) (addedAtMs : Nat) : Collection × Nat :=
  match c.bookmarks.find? (·.url == url) with
  | some existing =>
    let updated : Bookmark := { existing with title, folder, tags := canonTags tags }
    let newList := c.bookmarks.map fun b =>
      if b.id == existing.id then updated else b
    ({ c with bookmarks := newList }, existing.id)
  | none =>
    let id := c.nextId
    let b : Bookmark := { id, url, title, folder,
                          tags := canonTags tags, addedAtMs }
    ({ c with bookmarks := c.bookmarks ++ [b], nextId := id + 1 }, id)

def Collection.remove (c : Collection) (id : Nat) : Collection × Bool :=
  if c.bookmarks.any (·.id == id) then
    ({ c with bookmarks := c.bookmarks.filter (·.id ≠ id) }, true)
  else (c, false)

def Collection.count (c : Collection) : Nat := c.bookmarks.length

def Collection.get (c : Collection) (id : Nat) : Option Bookmark :=
  c.bookmarks.find? (·.id == id)

private def strContains (haystack needle : String) : Bool :=
  let h := haystack.toLower
  let n := needle.toLower
  (h.splitOn n).length > 1

def Collection.search (c : Collection) (q : String) : List Bookmark :=
  c.bookmarks.filter fun b =>
    strContains b.title q || strContains b.url q ||
    b.tags.any (strContains · q)

def Collection.byTag (c : Collection) (tag : String) : List Bookmark :=
  c.bookmarks.filter (·.tags.contains tag)

def Collection.byFolder (c : Collection) (folder : String) (recursive : Bool)
    : List Bookmark :=
  if recursive then
    let prefix := if folder.endsWith "/" then folder else folder ++ "/"
    c.bookmarks.filter fun b =>
      b.folder == folder || b.folder.startsWith prefix
  else
    c.bookmarks.filter (·.folder == folder)

def Collection.sortBy (c : Collection) (s : SortBy) : List Bookmark :=
  match s with
  | .title   => c.bookmarks.mergeSort (fun a b => a.title < b.title)
  | .url     => c.bookmarks.mergeSort (fun a b => a.url < b.url)
  | .addedAt => c.bookmarks.mergeSort (fun a b => a.addedAtMs < b.addedAtMs)
  | .folder  => c.bookmarks.mergeSort (fun a b => a.folder < b.folder)

def Collection.allTags (c : Collection) : List String :=
  (c.bookmarks.flatMap (·.tags)).eraseDups.mergeSort (· < ·)

def Collection.allFolders (c : Collection) : List String :=
  (c.bookmarks.map (·.folder)).eraseDups.mergeSort (· < ·)

end Web.BM
