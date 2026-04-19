-- Content Management System — lifecycle FSM + revisions + scheduled publish.
-- Pure-functional: every operation returns a new CMS.

namespace Web.CMS

inductive ContentState where
  | draft | inReview | scheduled | published | archived
  deriving Repr, DecidableEq, BEq

structure Article where
  id : Nat
  slug : String
  title : String
  body : String
  author : String
  state : ContentState := .draft
  tags : List String := []
  createdAtMs : Nat := 0
  modifiedAtMs : Nat := 0
  scheduledPublishAtMs : Option Nat := none
  publishedAtMs : Option Nat := none
  deriving Repr

structure Revision where
  articleId : Nat
  version : Nat
  title : String
  body : String
  savedAtMs : Nat
  savedBy : String
  deriving Repr

inductive Error where
  | unknownArticle (id : Nat)
  | unknownRevision (id ver : Nat)
  | illegalTransition (from_ : ContentState) (action : String)
  | archivedIsFrozen
  | slugTaken (slug : String)
  deriving Repr

structure CMS where
  articles : List Article := []
  revisions : List (Nat × List Revision) := []
  nextId : Nat := 1
  deriving Repr

def CMS.empty : CMS := {}

private def revisionsOf (c : CMS) (id : Nat) : List Revision :=
  (c.revisions.find? (·.1 == id)).map (·.2) |>.getD []

private def setRevisions (c : CMS) (id : Nat) (revs : List Revision) : CMS :=
  if c.revisions.any (·.1 == id) then
    { c with revisions := c.revisions.map fun p =>
      if p.1 == id then (id, revs) else p }
  else
    { c with revisions := c.revisions ++ [(id, revs)] }

private def updateArticle (c : CMS) (id : Nat) (f : Article → Article) : CMS :=
  { c with articles := c.articles.map fun a =>
    if a.id == id then f a else a }

def CMS.createArticle (c : CMS) (author slug title body : String) (nowMs : Nat)
    : Except Error (CMS × Nat) := do
  if c.articles.any (·.slug == slug) then throw (.slugTaken slug)
  let id := c.nextId
  let a : Article := {
    id, slug, title, body, author,
    state := .draft,
    createdAtMs := nowMs, modifiedAtMs := nowMs
  }
  let rev : Revision := {
    articleId := id, version := 1, title, body,
    savedAtMs := nowMs, savedBy := author
  }
  let c' : CMS := { c with
    articles := c.articles ++ [a],
    revisions := c.revisions ++ [(id, [rev])],
    nextId := id + 1
  }
  pure (c', id)

def CMS.article (c : CMS) (id : Nat) : Option Article :=
  c.articles.find? (·.id == id)

def CMS.saveRevision (c : CMS) (id : Nat) (user title body : String) (nowMs : Nat)
    : Except Error (CMS × Nat) := do
  let a ← (c.article id).toExcept (.unknownArticle id)
  if a.state == .archived then throw .archivedIsFrozen
  let revs := revisionsOf c id
  let version := match revs.getLast? with | some r => r.version + 1 | none => 1
  let newRev : Revision := {
    articleId := id, version, title, body,
    savedAtMs := nowMs, savedBy := user
  }
  let c' := updateArticle c id (fun x => { x with title, body, modifiedAtMs := nowMs })
  let c'' := setRevisions c' id (revs ++ [newRev])
  pure (c'', version)

def CMS.tick (c : CMS) (nowMs : Nat) : CMS × List Nat :=
  let mut cur := c
  let mut published : List Nat := []
  for a in c.articles do
    if a.state == .scheduled then
      match a.scheduledPublishAtMs with
      | some at_ms =>
        if at_ms ≤ nowMs then
          cur := updateArticle cur a.id (fun x => {
            x with state := .published,
            publishedAtMs := some nowMs,
            modifiedAtMs := nowMs,
            scheduledPublishAtMs := none
          })
          published := a.id :: published
      | none => pure ()
  (cur, published.reverse)

end Web.CMS
