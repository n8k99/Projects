-- Remote Login — authenticated session management with credentials and tokens.

namespace Networking

-- Simple hash for credential verification (not cryptographic)
private def simpleHash (s : String) : String :=
  let h := s.foldl (init := 5381) fun acc c => (acc * 33 + c.toNat) % (2^64)
  s!"{h}"

structure AuthCredential where
  username : String
  passwordHash : String
  salt : String
  role : String := "user"
  locked : Bool := false
  failedAttempts : Nat := 0
  lastLogin : Option Nat := none
  deriving Repr

def AuthCredential.create (username password : String) (role : String := "user") : AuthCredential :=
  let salt := s!"salt-{username}-{password.length}"  -- deterministic for pure model
  { username, passwordHash := simpleHash (salt ++ password), salt, role }

def AuthCredential.verify (c : AuthCredential) (password : String) : Bool :=
  simpleHash (c.salt ++ password) == c.passwordHash

structure AuthSession' where
  token : String
  username : String
  role : String
  created : Nat
  expires : Nat
  ipAddress : String := ""
  isActive : Bool := true
  deriving Repr

def AuthSession'.isValid (s : AuthSession') (now : Nat) : Bool :=
  s.isActive && now < s.expires

structure AuthLoginResult' where
  success : Bool
  token : Option String := none
  error : String := ""
  attemptsRemaining : Option Nat := none
  deriving Repr

structure AuthLoginAttempt' where
  username : String
  ipAddress : String
  timestamp : Nat
  success : Bool
  reason : String
  deriving Repr

structure AuthServerState where
  sessionTTL : Nat := 1800
  maxFailed : Nat := 5
  lockoutDuration : Nat := 900
  credentials : List AuthCredential := []
  sessions : List AuthSession' := []
  auditLog : List AuthLoginAttempt' := []
  lockoutUntil : List (String × Nat) := []
  clock : Nat := 0
  nextToken : Nat := 0
  deriving Repr

namespace AuthServerState

def register (s : AuthServerState) (username password : String) (role : String := "user")
    : Except String AuthServerState :=
  if s.credentials.any (·.username == username) then .error "User exists"
  else .ok { s with credentials := s.credentials ++ [AuthCredential.create username password role] }

def login (s : AuthServerState) (username password ip : String)
    : AuthServerState × AuthLoginResult' :=
  let now := s.clock
  -- Check lockout
  match s.lockoutUntil.find? (·.1 == username) with
  | some (_, until) =>
    if now < until then
      let s := { s with auditLog := s.auditLog ++ [{ username, ipAddress := ip, timestamp := now, success := false, reason := "locked" }] }
      (s, { success := false, error := "Account locked" })
    else
      let s := { s with lockoutUntil := s.lockoutUntil.filter (·.1 != username) }
      loginInner s username password ip now
  | none => loginInner s username password ip now
where
  loginInner (s : AuthServerState) (username password ip : String) (now : Nat)
      : AuthServerState × AuthLoginResult' :=
    match s.credentials.find? (·.username == username) with
    | none =>
      let s := { s with auditLog := s.auditLog ++ [{ username, ipAddress := ip, timestamp := now, success := false, reason := "unknown" }] }
      (s, { success := false, error := "Invalid credentials" })
    | some cred =>
      if !cred.verify password then
        let newFailed := cred.failedAttempts + 1
        let s := { s with credentials := s.credentials.map fun c =>
          if c.username == username then { c with failedAttempts := newFailed } else c }
        if newFailed >= s.maxFailed then
          let s := { s with lockoutUntil := s.lockoutUntil ++ [(username, now + s.lockoutDuration)] }
          (s, { success := false, error := "Account locked", attemptsRemaining := some 0 })
        else
          (s, { success := false, error := "Invalid credentials", attemptsRemaining := some (s.maxFailed - newFailed) })
      else
        let token := s!"token-{s.nextToken + 1}"
        let session : AuthSession' := { token, username, role := cred.role, created := now, expires := now + s.sessionTTL, ipAddress := ip }
        let s := { s with
          nextToken := s.nextToken + 1
          sessions := s.sessions ++ [session]
          credentials := s.credentials.map fun c =>
            if c.username == username then { c with failedAttempts := 0, lastLogin := some now } else c
          auditLog := s.auditLog ++ [{ username, ipAddress := ip, timestamp := now, success := true, reason := "ok" }] }
        (s, { success := true, token := some token })

def validateSession (s : AuthServerState) (token : String) : Option AuthSession' :=
  s.sessions.find? fun sess => sess.token == token && sess.isValid s.clock

def logout (s : AuthServerState) (token : String) : AuthServerState :=
  { s with sessions := s.sessions.map fun sess =>
    if sess.token == token then { sess with isActive := false } else sess }

end AuthServerState

def remoteLoginDemo : IO Unit := do
  let s : AuthServerState := { sessionTTL := 1800, maxFailed := 3 }
  match s.register "nathan" "rosetta" with
  | .ok s =>
    let (s, result) := s.login "nathan" "rosetta" "127.0.0.1"
    IO.println s!"Login: {result.success}, token: {result.token.getD \"none\"}"
    let (_, result2) := s.login "nathan" "wrong" "10.0.0.1"
    IO.println s!"Bad login: {result2.success}, error: {result2.error}"
  | .error e => IO.println s!"Register failed: {e}"

end Networking
