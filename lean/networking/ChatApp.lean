-- Chat Application (Networking) — multi-room chat with users and history.

namespace Networking

structure Message where
  sender : String
  room : String
  text : String
  timestamp : Nat  -- monotonic counter (no IO clock needed)
  deriving Repr, BEq

structure ChatRoom where
  name : String
  members : List String := []
  messages : List Message := []
  deriving Repr

structure ChatUser where
  name : String
  rooms : List String := []
  deriving Repr

structure ChatServer where
  rooms : List ChatRoom := []
  users : List ChatUser := []
  clock : Nat := 0  -- monotonic message counter for timestamps
  deriving Repr

-- Helper: find and update an element in a list
private def updateIn [BEq α] (xs : List α) (pred : α → Bool) (f : α → α) : List α :=
  xs.map fun x => if pred x then f x else x

namespace ChatServer

def registerUser (s : ChatServer) (name : String) : Except String ChatServer :=
  if s.users.any (·.name == name) then
    .error s!"User '{name}' already exists"
  else
    .ok { s with users := s.users ++ [{ name }] }

def createRoom (s : ChatServer) (roomName : String) : Except String ChatServer :=
  if s.rooms.any (·.name == roomName) then
    .error s!"Room '{roomName}' already exists"
  else
    .ok { s with rooms := s.rooms ++ [{ name := roomName }] }

def joinRoom (s : ChatServer) (userName roomName : String) : Except String ChatServer := do
  unless s.users.any (·.name == userName) do
    throw s!"No such user: '{userName}'"
  unless s.rooms.any (·.name == roomName) do
    throw s!"No such room: '{roomName}'"
  let rooms := updateIn s.rooms (·.name == roomName) fun r =>
    if r.members.contains userName then r
    else { r with members := r.members ++ [userName] }
  let users := updateIn s.users (·.name == userName) fun u =>
    if u.rooms.contains roomName then u
    else { u with rooms := u.rooms ++ [roomName] }
  .ok { s with rooms, users }

def leaveRoom (s : ChatServer) (userName roomName : String) : Except String ChatServer := do
  unless s.users.any (·.name == userName) do
    throw s!"No such user: '{userName}'"
  unless s.rooms.any (·.name == roomName) do
    throw s!"No such room: '{roomName}'"
  let rooms := updateIn s.rooms (·.name == roomName) fun r =>
    { r with members := r.members.filter (· != userName) }
  let users := updateIn s.users (·.name == userName) fun u =>
    { u with rooms := u.rooms.filter (· != roomName) }
  .ok { s with rooms, users }

def sendMessage (s : ChatServer) (userName roomName text : String) : Except String (ChatServer × Message) := do
  unless s.users.any (·.name == userName) do
    throw s!"No such user: '{userName}'"
  let room ← match s.rooms.find? (·.name == roomName) with
    | some r => pure r
    | none => throw s!"No such room: '{roomName}'"
  unless room.members.contains userName do
    throw s!"User '{userName}' is not a member of room '{roomName}'"
  let msg : Message := {
    sender := userName
    room := roomName
    text := text
    timestamp := s.clock
  }
  let rooms := updateIn s.rooms (·.name == roomName) fun r =>
    { r with messages := r.messages ++ [msg] }
  .ok ({ s with rooms, clock := s.clock + 1 }, msg)

def getMessages (s : ChatServer) (roomName : String) (since : Option Nat := none) : Except String (List Message) := do
  let room ← match s.rooms.find? (·.name == roomName) with
    | some r => pure r
    | none => throw s!"No such room: '{roomName}'"
  match since with
  | none => .ok room.messages
  | some ts => .ok (room.messages.filter (·.timestamp >= ts))

def listRooms (s : ChatServer) : List String :=
  s.rooms.map (·.name) |>.mergeSort

def listUsers (s : ChatServer) (roomName : String) : Except String (List String) := do
  let room ← match s.rooms.find? (·.name == roomName) with
    | some r => pure r
    | none => throw s!"No such room: '{roomName}'"
  .ok (room.members.mergeSort)

end ChatServer

-- Demo
def demo : IO Unit := do
  let mut s := ChatServer.mk

  -- Register users
  let .ok s' := s.registerUser "alice" | IO.println "error"
  s := s'
  let .ok s' := s.registerUser "bob" | IO.println "error"
  s := s'
  let .ok s' := s.registerUser "carol" | IO.println "error"
  s := s'

  -- Create rooms
  let .ok s' := s.createRoom "#general" | IO.println "error"
  s := s'
  let .ok s' := s.createRoom "#engineering" | IO.println "error"
  s := s'

  -- Join rooms
  let .ok s' := s.joinRoom "alice" "#general" | IO.println "error"
  s := s'
  let .ok s' := s.joinRoom "bob" "#general" | IO.println "error"
  s := s'
  let .ok s' := s.joinRoom "carol" "#general" | IO.println "error"
  s := s'
  let .ok s' := s.joinRoom "alice" "#engineering" | IO.println "error"
  s := s'
  let .ok s' := s.joinRoom "bob" "#engineering" | IO.println "error"
  s := s'

  -- Send messages
  let .ok (s', _) := s.sendMessage "alice" "#general" "Hello everyone!" | IO.println "error"
  s := s'
  let .ok (s', _) := s.sendMessage "bob" "#general" "Hey Alice!" | IO.println "error"
  s := s'
  let .ok (s', _) := s.sendMessage "carol" "#general" "Good morning!" | IO.println "error"
  s := s'
  let .ok (s', _) := s.sendMessage "alice" "#engineering" "Sprint planning at 2pm" | IO.println "error"
  s := s'
  let .ok (s', _) := s.sendMessage "bob" "#engineering" "Sounds good" | IO.println "error"
  s := s'

  -- List rooms
  IO.println s!"Rooms: {s.listRooms}"

  -- List members
  for room in s.listRooms do
    match s.listUsers room with
    | .ok members => IO.println s!"Members of {room}: {members}"
    | .error e => IO.println e

  -- Show messages
  for room in s.listRooms do
    IO.println s!"--- {room} ---"
    match s.getMessages room with
    | .ok msgs =>
      for msg in msgs do
        IO.println s!"  [{msg.timestamp}] {msg.sender}: {msg.text}"
    | .error e => IO.println e

  -- Carol leaves
  match s.leaveRoom "carol" "#general" with
  | .ok s' =>
    s := s'
    match s.listUsers "#general" with
    | .ok members => IO.println s!"After carol leaves #general: {members}"
    | .error e => IO.println e
  | .error e => IO.println e

end Networking
