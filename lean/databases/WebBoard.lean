-- Web Board / Forum — threaded posts with vote tallies + hot-score ranking.

namespace Databases.WB

inductive ThreadSort where | new | top | hot
  deriving Repr, BEq

structure ThreadEntry where
  id : Int
  title : String
  author : String
  createdMs : Int
  pinned : Bool := false
  locked : Bool := false
  deriving Repr

structure Post where
  id : Int
  threadId : Int
  parentId : Option Int := none
  author : String
  body : String
  createdMs : Int
  upvotes : Nat := 0
  downvotes : Nat := 0
  deriving Repr

def Post.score (p : Post) : Int :=
  p.upvotes.toInt - p.downvotes.toInt

def hotScore (p : Post) (nowMs : Int) : Float :=
  let s := p.score
  let sign : Float := if s > 0 then 1.0 else if s < 0 then -1.0 else 0.0
  let absV := if s < 0 then -s else s
  let magnitude := (max 1 absV.toNat).toFloat.log10
  let ageDays := max 0.0 ((nowMs - p.createdMs).toFloat / 86400000.0)
  sign * magnitude - ageDays * 0.2

structure PostWithDepth where
  post : Post
  depth : Nat
  deriving Repr

structure Board where
  threads : List (Int × ThreadEntry) := []
  posts : List (Int × Post) := []
  deriving Repr

def Board.addThread (b : Board) (t : ThreadEntry) : Board :=
  { b with threads := (b.threads.filter (fun p => p.1 ≠ t.id)) ++ [(t.id, t)] }

def Board.getThread (b : Board) (id : Int) : Option ThreadEntry :=
  (b.threads.find? (fun p => p.1 == id)).map Prod.snd

def Board.getPost (b : Board) (id : Int) : Option Post :=
  (b.posts.find? (fun p => p.1 == id)).map Prod.snd

def Board.addPost (b : Board) (p : Post) : Except String Board := do
  if (b.getThread p.threadId).isNone then
    throw s!"unknown thread {p.threadId}"
  match p.parentId with
  | some pid =>
    match b.getPost pid with
    | none => throw s!"unknown parent {pid}"
    | some parent =>
      if parent.threadId ≠ p.threadId then
        throw s!"parent {pid} is in a different thread"
  | none => pure ()
  return { b with posts := (b.posts.filter (fun q => q.1 ≠ p.id)) ++ [(p.id, p)] }

def Board.topLevelPosts (b : Board) (threadId : Int) : List Post :=
  (b.posts.filterMap fun (_, p) =>
    if p.threadId == threadId ∧ p.parentId.isNone then some p else none)
  |>.mergeSort (fun a b => a.createdMs < b.createdMs)

def Board.childrenOf (b : Board) (postId : Int) : List Post :=
  (b.posts.filterMap fun (_, p) =>
    match p.parentId with
    | some pid => if pid == postId then some p else none
    | none => none)
  |>.mergeSort (fun a b => a.createdMs < b.createdMs)

partial def Board.walkSubtree (b : Board) (p : Post) (depth : Nat) : List PostWithDepth :=
  let here : PostWithDepth := { post := p, depth := depth }
  let children := b.childrenOf p.id
  here :: children.bind (fun c => b.walkSubtree c (depth + 1))

def Board.threadTree (b : Board) (threadId : Int) : List PostWithDepth :=
  (b.topLevelPosts threadId).bind (fun p => b.walkSubtree p 0)

def Board.threadScore (b : Board) (threadId : Int) : Int :=
  (b.posts.foldl (init := (0 : Int)) fun acc (_, p) =>
    if p.threadId == threadId then acc + p.score else acc)

def Board.threadHotScore (b : Board) (threadId : Int) (nowMs : Int) : Float :=
  (b.posts.foldl (init := (0.0 : Float)) fun acc (_, p) =>
    if p.threadId == threadId then acc + hotScore p nowMs else acc)

def Board.postCount (b : Board) (threadId : Int) : Nat :=
  (b.posts.filter (fun (_, p) => p.threadId == threadId)).length

def Board.listThreads (b : Board) (sort : ThreadSort) (nowMs : Int) : List ThreadEntry :=
  let threads := b.threads.map Prod.snd
  threads.mergeSort fun a t2 =>
    if a.pinned ∧ !t2.pinned then true
    else if !a.pinned ∧ t2.pinned then false
    else match sort with
      | .new => a.createdMs > t2.createdMs
      | .top => b.threadScore a.id > b.threadScore t2.id
      | .hot => b.threadHotScore a.id nowMs > b.threadHotScore t2.id nowMs

end Databases.WB
