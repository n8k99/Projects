/-
  News Ticker and Game Scores — real-time prioritized message stream.
-/

namespace Text

inductive Category where
  | news
  | sports
  | finance
  | alert
  deriving Repr, BEq, Inhabited

instance : ToString Category where
  toString
    | .news    => "news"
    | .sports  => "sports"
    | .finance => "finance"
    | .alert   => "alert"

structure TickerItem where
  text      : String
  category  : Category
  priority  : Nat  -- 1-5, 5 = highest
  timestamp : Nat  -- epoch seconds
  expiresAt : Option Nat := none
  deriving Repr, Inhabited

def TickerItem.isExpired (item : TickerItem) (now : Nat) : Bool :=
  match item.expiresAt with
  | none       => false
  | some expAt => now >= expAt

structure NewsTicker where
  items : List TickerItem := []
  deriving Repr, Inhabited

def NewsTicker.post (ticker : NewsTicker) (text : String) (category : Category)
    (priority : Nat) (now : Nat) (ttlSeconds : Option Nat := none) : NewsTicker × TickerItem :=
  let expiresAt := ttlSeconds.map (· + now)
  let item : TickerItem := {
    text := text
    category := category
    priority := min priority 5 |> max 1
    timestamp := now
    expiresAt := expiresAt
  }
  ({ ticker with items := ticker.items ++ [item] }, item)

def NewsTicker.breaking (ticker : NewsTicker) (text : String) (now : Nat) : NewsTicker × TickerItem :=
  ticker.post text .alert 5 now

def NewsTicker.getCurrent (ticker : NewsTicker) (now : Nat) : List TickerItem :=
  let active := ticker.items.filter (fun item => !item.isExpired now)
  -- Sort by priority descending, then timestamp ascending
  active.toArray
    |>.qsort (fun a b =>
      if a.priority != b.priority then a.priority > b.priority
      else a.timestamp < b.timestamp)
    |>.toList

def NewsTicker.getByCategory (ticker : NewsTicker) (cat : Category) (now : Nat) : List TickerItem :=
  (ticker.getCurrent now).filter (fun item => item.category == cat)

def NewsTicker.expireOld (ticker : NewsTicker) (now : Nat) : NewsTicker × Nat :=
  let before := ticker.items.length
  let kept := ticker.items.filter (fun item => !item.isExpired now)
  ({ ticker with items := kept }, before - kept.length)

def NewsTicker.display (ticker : NewsTicker) (now : Nat) : String :=
  let current := ticker.getCurrent now
  match current with
  | [] => ">>> No items >>>"
  | items =>
    let texts := items.map (·.text)
    ">>> " ++ " | ".intercalate texts ++ " >>>"

-- Demo
def demo : IO Unit := do
  let now : Nat := 1000

  let ticker := NewsTicker.mk []
  let (ticker, _) := ticker.post "Markets rally on strong earnings" .finance 3 now
  let (ticker, _) := ticker.post "Lakers defeat Celtics 112-108" .sports 2 (now + 1)
  let (ticker, _) := ticker.post "New climate accord signed by 40 nations" .news 4 (now + 2)
  let (ticker, _) := ticker.post "Flash sale: tech stocks surge 5%" .finance 2 (now + 3) (some 10)

  IO.println "=== Full Ticker ==="
  IO.println (ticker.display now)

  IO.println "\n=== Sports Only ==="
  for item in ticker.getByCategory .sports now do
    IO.println s!"  [{item.priority}] {item.text}"

  -- Simulate time passing beyond TTL
  let futureNow := now + 20
  let (ticker, removed) := ticker.expireOld futureNow
  IO.println s!"\n=== After expiry ({removed} removed) ==="
  IO.println (ticker.display futureNow)

  let (ticker, _) := ticker.breaking "EARTHQUAKE: 7.2 magnitude off Pacific coast" futureNow
  IO.println "\n=== BREAKING NEWS ==="
  IO.println (ticker.display futureNow)

end Text
