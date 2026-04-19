-- Web Browser with Tabs — state model for tabbed navigation.
-- Pure-functional: every operation returns a new Browser.

namespace Web

abbrev TabId := Nat

structure Tab where
  id : TabId
  title : String
  url : String
  back : List String := []
  forward : List String := []
  deriving Repr

structure ClosedTab where
  url : String
  title : String
  back : List String
  forward : List String
  deriving Repr

structure Browser where
  tabs : List Tab := []
  active : Option TabId := none
  closed : List ClosedTab := []            -- newest first
  bookmarks : List (String × String) := []
  nextId : TabId := 1
  deriving Repr

def Browser.empty : Browser := {}

def Browser.openTab (b : Browser) (url : String) : Browser × TabId :=
  let id := b.nextId
  let tab : Tab := { id, title := url, url }
  ({ b with
       tabs := b.tabs ++ [tab],
       active := some id,
       nextId := id + 1 }, id)

def Browser.activeTab (b : Browser) : Option Tab :=
  b.active.bind fun id => b.tabs.find? (·.id == id)

private def updateTab (tabs : List Tab) (id : TabId) (f : Tab → Tab) : List Tab :=
  tabs.map fun t => if t.id == id then f t else t

def Browser.navigate (b : Browser) (url : String) : Browser :=
  match b.active with
  | none => b
  | some id =>
    { b with tabs := updateTab b.tabs id fun t =>
      { t with back := t.back ++ [t.url], forward := [],
               url, title := url } }

def Browser.goBack (b : Browser) : Browser × Option String :=
  match b.activeTab with
  | none => (b, none)
  | some t =>
    if t.back.isEmpty then (b, none)
    else
      let prev := t.back.getLast!
      let newBack := t.back.dropLast
      let newForward := t.forward ++ [t.url]
      let newB := { b with tabs := updateTab b.tabs t.id fun tab =>
        { tab with back := newBack, forward := newForward, url := prev } }
      (newB, some prev)

def Browser.goForward (b : Browser) : Browser × Option String :=
  match b.activeTab with
  | none => (b, none)
  | some t =>
    if t.forward.isEmpty then (b, none)
    else
      let nxt := t.forward.getLast!
      let newForward := t.forward.dropLast
      let newBack := t.back ++ [t.url]
      let newB := { b with tabs := updateTab b.tabs t.id fun tab =>
        { tab with back := newBack, forward := newForward, url := nxt } }
      (newB, some nxt)

def Browser.closeTab (b : Browser) (id : TabId) : Browser :=
  match b.tabs.find? (·.id == id) with
  | none => b
  | some t =>
    let ct : ClosedTab := { url := t.url, title := t.title,
                            back := t.back, forward := t.forward }
    let remaining := b.tabs.filter (·.id ≠ id)
    let newActive :=
      if b.active = some id then
        (remaining.head?).map (·.id)
      else b.active
    { b with tabs := remaining, active := newActive, closed := ct :: b.closed }

def Browser.reopenClosed (b : Browser) : Browser × Option TabId :=
  match b.closed with
  | [] => (b, none)
  | ct :: rest =>
    let id := b.nextId
    let t : Tab := { id, title := ct.title, url := ct.url,
                     back := ct.back, forward := ct.forward }
    ({ b with tabs := b.tabs ++ [t],
              active := some id,
              closed := rest,
              nextId := id + 1 }, some id)

def Browser.addBookmark (b : Browser) : Browser :=
  match b.activeTab with
  | none => b
  | some t =>
    let entry := (t.title, t.url)
    if b.bookmarks.contains entry then b
    else { b with bookmarks := b.bookmarks ++ [entry] }

end Web
