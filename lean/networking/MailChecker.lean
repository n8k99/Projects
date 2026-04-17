-- Mail Checker — monitor an inbox for new messages with filtering and notification.
-- Pure data model: messages, mailboxes, filters, and check operations.

namespace Networking

structure MailAddr where
  name : String := ""
  address : String := ""
  deriving Repr, BEq

def MailAddr.display (a : MailAddr) : String :=
  if a.name.isEmpty then a.address else s!"{a.name} <{a.address}>"

def parseMailAddr (raw : String) : MailAddr :=
  let raw := raw.trim
  match raw.findIdx (· == '<'), raw.findIdx (· == '>') with
  | lt, gt =>
    if lt < gt && lt < raw.length then
      let name := (raw.take lt).trim.trim "\"".toList.head!  -- simplified
      let addr := (raw.extract ⟨lt + 1⟩ ⟨gt⟩).trim
      { name := (raw.take lt).trim, address := addr }
    else { address := raw }

structure MailMsg' where
  uid : String
  sender : MailAddr
  recipients : List MailAddr := []
  subject : String
  body : String
  date : Nat := 0
  isRead : Bool := false
  isFlagged : Bool := false
  deriving Repr

structure MailFilter' where
  senderContains : String := ""
  subjectContains : String := ""
  bodyContains : String := ""
  isUnreadOnly : Bool := false
  since : Option Nat := none
  deriving Repr

private def containsCI (haystack needle : String) : Bool :=
  haystack.toLower.containsSubstr needle.toLower

def MailMsg'.matchesFilter (m : MailMsg') (f : MailFilter') : Bool :=
  (f.senderContains.isEmpty || containsCI m.sender.address f.senderContains) &&
  (f.subjectContains.isEmpty || containsCI m.subject f.subjectContains) &&
  (f.bodyContains.isEmpty || containsCI m.body f.bodyContains) &&
  (!f.isUnreadOnly || !m.isRead) &&
  (match f.since with | none => true | some s => m.date >= s)

structure MailCheckResult where
  totalMessages : Nat
  newMessages : Nat
  matchedMessages : List MailMsg'
  deriving Repr

structure MailboxState where
  owner : String
  messages : List MailMsg' := []
  seenUIDs : List String := []
  deriving Repr

namespace MailboxState

def deliver (mb : MailboxState) (msg : MailMsg') : MailboxState :=
  { mb with messages := mb.messages ++ [msg] }

def check (mb : MailboxState) (filter : Option MailFilter' := none) : MailboxState × MailCheckResult :=
  let currentUIDs := mb.messages.map (·.uid)
  let newUIDs := currentUIDs.filter (!mb.seenUIDs.contains ·)
  let mb' := { mb with seenUIDs := currentUIDs }
  let sorted := mb'.messages.reverse  -- newest first (assuming append order)
  let matched := match filter with
    | some f => sorted.filter (·.matchesFilter f)
    | none => sorted
  let result : MailCheckResult := {
    totalMessages := mb'.messages.length
    newMessages := newUIDs.length
    matchedMessages := matched }
  (mb', result)

def markRead (mb : MailboxState) (uid : String) : MailboxState :=
  { mb with messages := mb.messages.map fun m =>
    if m.uid == uid then { m with isRead := true } else m }

def markFlagged (mb : MailboxState) (uid : String) (flagged : Bool := true) : MailboxState :=
  { mb with messages := mb.messages.map fun m =>
    if m.uid == uid then { m with isFlagged := flagged } else m }

def delete (mb : MailboxState) (uid : String) : MailboxState :=
  { mb with
    messages := mb.messages.filter (·.uid != uid)
    seenUIDs := mb.seenUIDs.filter (· != uid) }

def unreadCount (mb : MailboxState) : Nat :=
  mb.messages.filter (!·.isRead) |>.length

end MailboxState

-- Server: routes messages between mailboxes
structure MailServerState where
  mailboxes : List (String × MailboxState) := []
  nextUID : Nat := 0
  deriving Repr

namespace MailServerState

def createMailbox (s : MailServerState) (owner : String) : MailServerState :=
  { s with mailboxes := s.mailboxes ++ [(owner, { owner })] }

def send (s : MailServerState) (sender : MailAddr) (recipients : List MailAddr)
    (subject body : String) : MailServerState × String :=
  let uid := s!"msg-{s.nextUID + 1}"
  let msg : MailMsg' := { uid, sender, recipients, subject, body, date := s.nextUID + 1 }
  let mailboxes := s.mailboxes.map fun (owner, mb) =>
    if recipients.any (·.address == owner) then (owner, mb.deliver msg)
    else (owner, mb)
  ({ s with mailboxes, nextUID := s.nextUID + 1 }, uid)

def getMailbox (s : MailServerState) (owner : String) : Option MailboxState :=
  (s.mailboxes.find? (·.1 == owner)).map (·.2)

end MailServerState

-- Demo
def mailCheckerDemo : IO Unit := do
  let s := MailServerState.createMailbox ({} : MailServerState) "nathan@em.com"
  let s := s.createMailbox "kathryn@em.com"
  let (s, _) := s.send { name := "Kathryn", address := "kathryn@em.com" }
    [{ address := "nathan@em.com" }] "Q2 Finance" "Positions tracking ahead."
  let (s, _) := s.send { name := "Jay", address := "jay@em.com" }
    [{ address := "nathan@em.com" }] "Git Summary" "12 commits, G036-G038."
  match s.getMailbox "nathan@em.com" with
  | some mb =>
    let (_, result) := mb.check (some { senderContains := "em.com" })
    IO.println s!"Total: {result.totalMessages}, New: {result.newMessages}, Matched: {result.matchedMessages.length}"
  | none => IO.println "No mailbox"

end Networking
