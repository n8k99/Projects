/- Regex Query Tool — pattern matching, extraction, and replacement. -/

namespace Text

/-- A match result with the matched text and its position span. -/
structure RegexMatch where
  text : String
  start : Nat
  end_ : Nat
  deriving Repr, BEq

/-- Check whether `c` is in the character class `[a-zA-Z0-9]`. -/
private def isAlphaNum (c : Char) : Bool :=
  c.isAlpha || c.isDigit

/-- Check whether `c` is in the set of email local-part characters. -/
private def isEmailLocal (c : Char) : Bool :=
  isAlphaNum c || "._%+-".data.contains c

/-- Check whether `c` is in the set of email domain characters. -/
private def isEmailDomain (c : Char) : Bool :=
  isAlphaNum c || ".-".data.contains c

/-- Simple exact substring search — return starting positions of `needle` in `haystack`. -/
def findSubstringPositions (haystack needle : String) : List Nat :=
  if needle.isEmpty then []
  else
    let hLen := haystack.length
    let nLen := needle.length
    let rec go (i : Nat) (acc : List Nat) : List Nat :=
      if i + nLen > hLen then acc.reverse
      else
        let sub := haystack.extract ⟨i⟩ ⟨i + nLen⟩
        if sub == needle then go (i + nLen) (i :: acc)
        else go (i + 1) acc
    go 0 []

/-- Split `text` on every occurrence of literal `sep`. -/
def splitOn (text sep : String) : List String :=
  if sep.isEmpty then [text]
  else
    let positions := findSubstringPositions text sep
    match positions with
    | [] => [text]
    | _ =>
      let rec go (start : Nat) (ps : List Nat) (acc : List String) : List String :=
        match ps with
        | [] => (acc ++ [text.extract ⟨start⟩ ⟨text.length⟩]).filter (· != "")
        | p :: rest => go (p + sep.length) rest (acc ++ [text.extract ⟨start⟩ ⟨p⟩])
      go 0 positions []

/-- Replace all literal occurrences of `old` with `new_` in `text`. -/
def replaceAll (text old new_ : String) : String :=
  (splitOn text old) |> String.intercalate new_

/-- Check if `text` contains literal `pattern`. -/
def containsSubstring (text pattern : String) : Bool :=
  !(findSubstringPositions text pattern).isEmpty

-- ===========================================================================
-- Pre-built pattern validators (character-level, no regex engine)
-- ===========================================================================

/-- Validate an email address (simple heuristic). -/
def isEmail (s : String) : Bool :=
  match s.splitOn "@" with
  | [local_, domain] =>
    !local_.isEmpty && !domain.isEmpty
    && domain.data.contains '.'
    && local_.data.all isEmailLocal
    && domain.data.all isEmailDomain
    && match domain.splitOn "." |>.getLast? with
       | some tld => tld.length >= 2
       | none => false
  | _ => false

/-- Validate an IPv4 address. -/
def isIPv4 (s : String) : Bool :=
  let parts := s.splitOn "."
  parts.length == 4 && parts.all fun p =>
    !p.isEmpty && p.data.all Char.isDigit &&
    match p.toNat? with
    | some n => n <= 255 && (p.length == 1 || p.data.head? != some '0')
    | none => false

/-- Validate a YYYY-MM-DD date string. -/
def isDate (s : String) : Bool :=
  s.length == 10
  && s.data.get? 4 == some '-'
  && s.data.get? 7 == some '-'
  && (s.extract ⟨0⟩ ⟨4⟩).data.all Char.isDigit
  && (s.extract ⟨5⟩ ⟨7⟩).data.all Char.isDigit
  && (s.extract ⟨8⟩ ⟨10⟩).data.all Char.isDigit

/-- Extract tokens from text that satisfy a predicate, splitting on whitespace. -/
def extractMatching (text : String) (pred : String -> Bool) : List String :=
  let words := text.splitOn " "
  words.map (fun w => w.data.filter (fun c => isAlphaNum c || ".@:/-_%+".data.contains c) |> List.asString)
    |>.filter pred

/-- Find email addresses in text. -/
def findEmails (text : String) : List String :=
  extractMatching text isEmail

/-- Find IPv4 addresses in text. -/
def findIPv4s (text : String) : List String :=
  extractMatching text isIPv4

/-- Find YYYY-MM-DD dates in text. -/
def findDates (text : String) : List String :=
  extractMatching text isDate

-- ===========================================================================
-- Demo
-- ===========================================================================

def demo : IO Unit := do
  let sample := "Contact alice@example.com. Server at 192.168.1.1. Date: 2026-04-16."

  IO.println "=== Find emails ==="
  for e in findEmails sample do
    IO.println s!"  {e}"

  IO.println "\n=== Find IPs ==="
  for ip in findIPv4s sample do
    IO.println s!"  {ip}"

  IO.println "\n=== Find dates ==="
  for d in findDates sample do
    IO.println s!"  {d}"

  IO.println "\n=== Split on \". \" ==="
  for part in splitOn sample ". " do
    IO.println s!"  \"{part}\""

  IO.println "\n=== Replace ==="
  IO.println s!"  {replaceAll sample "192.168.1.1" "[REDACTED]"}"

  IO.println "\n=== Contains ==="
  IO.println s!"  contains 'alice': {containsSubstring sample "alice"}"
  IO.println s!"  contains 'bob': {containsSubstring sample "bob"}"

end Text
