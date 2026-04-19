-- Chat Application — bidirectional broadcast across N concurrent actors.
--
-- Every actor is both a producer and a consumer. A send fans out to every
-- joined user's inbox. Total ordering is preserved by assigning a monotonic
-- sequence counter under the room mutex before enqueuing to inboxes — all
-- recipients then see messages in the same order.

namespace Threading.Chat

abbrev UserId := Nat

structure Message where
  seq : Nat
  fromId : UserId
  fromName : String
  text : String
  deriving Repr

structure Inbox where
  name : String
  messages : List Message := []   -- oldest first
  deriving Repr

structure RoomState where
  users : List (UserId × Inbox) := []
  log : List Message := []         -- newest first (prepend)
  nextId : UserId := 1
  nextSeq : Nat := 1
  deriving Repr

abbrev Room := IO.Mutex RoomState

def newRoom : IO Room := IO.Mutex.new {}

def join (r : Room) (name : String) : IO UserId :=
  r.atomically do
    let s ← get
    let id := s.nextId
    let newUser : Inbox := { name }
    set { s with
      users := s.users ++ [(id, newUser)],
      nextId := id + 1
    }
    return id

def leave (r : Room) (id : UserId) : IO Unit :=
  r.atomically do
    modify fun s => { s with users := s.users.filter (·.1 ≠ id) }

/-- Find and return (index, inbox) for `id`, or none. -/
private def findUser (users : List (UserId × Inbox)) (id : UserId)
    : Option Inbox :=
  (users.find? (·.1 == id)).map (·.2)

def send (r : Room) (sender : UserId) (text : String) : IO Bool :=
  r.atomically do
    let s ← get
    match findUser s.users sender with
    | none => return false
    | some senderInbox =>
      let seq := s.nextSeq
      let msg : Message := {
        seq, fromId := sender, fromName := senderInbox.name, text
      }
      let newUsers := s.users.map fun (id, inbox) =>
        (id, { inbox with messages := inbox.messages ++ [msg] })
      set { s with
        users := newUsers,
        log := msg :: s.log,
        nextSeq := seq + 1
      }
      return true

def read (r : Room) (id : UserId) : IO (List Message) :=
  r.atomically do
    let s ← get
    match findUser s.users id with
    | none => return []
    | some inbox =>
      let drained := inbox.messages
      let newUsers := s.users.map fun (uid, ib) =>
        if uid == id then (uid, { ib with messages := [] }) else (uid, ib)
      set { s with users := newUsers }
      return drained

def history (r : Room) : IO (List Message) :=
  r.atomically do return (← get).log.reverse

def userCount (r : Room) : IO Nat :=
  r.atomically do return (← get).users.length

end Threading.Chat
