-- CD Key Generator -- generate and validate software license keys.

namespace Text

def charset : String := "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
def groupLen : Nat := 5
def dataGroups : Nat := 4
def prime : Nat := 65521

/-- Compute a 5-char checksum from the first 4 groups. -/
def computeChecksum (groups : List String) : String :=
  let raw := String.join groups
  let bytes := raw.toList.map Char.toNat
  let h := bytes.enum.foldl (fun acc (i, b) => acc + b * (i + 1)) 0 % prime
  let charList := charset.toList
  let len := charList.length
  let rec go (h : Nat) (n : Nat) (acc : List Char) : List Char :=
    match n with
    | 0 => acc.reverse
    | n + 1 =>
      let c := charList.getD (h % len) 'A'
      go (h / len) n (c :: acc)
  String.mk (go h groupLen [])

/-- Generate a random group of 5 characters using IO for randomness. -/
def randomGroup : IO String := do
  let charList := charset.toList
  let len := charList.length
  let mut chars : List Char := []
  for _ in List.range groupLen do
    let r <- IO.rand 0 (len - 1)
    chars := charList.getD r 'A' :: chars
  return String.mk chars.reverse

/-- Generate a single CD key. -/
def generateKey : IO String := do
  let mut groups : List String := []
  for _ in List.range dataGroups do
    let g <- randomGroup
    groups := groups ++ [g]
  let checksum := computeChecksum groups
  let allGroups := groups ++ [checksum]
  return String.intercalate "-" allGroups

/-- Split a string by a separator character. -/
def splitBy (s : String) (sep : Char) : List String :=
  let rec go (chars : List Char) (current : List Char) (acc : List String) : List String :=
    match chars with
    | [] => (acc ++ [String.mk current.reverse]).reverse.reverse
    | c :: rest =>
      if c == sep then
        go rest [] (acc ++ [String.mk current.reverse])
      else
        go rest (c :: current) acc
  go s.toList [] []

/-- Validate a CD key by recomputing its checksum. -/
def validateKey (key : String) : Bool :=
  let upper := key.trim.toUpper
  let parts := splitBy upper '-'
  if parts.length != dataGroups + 1 then false
  else if parts.any (fun p => p.length != groupLen) then false
  else
    let charList := charset.toList
    if parts.any (fun p => p.toList.any (fun c => !charList.contains c)) then false
    else
      let dataGroupsList := parts.take dataGroups
      let expected := computeChecksum dataGroupsList
      parts.getD dataGroups "" == expected

/-- Generate a batch of n CD keys. -/
def generateBatch (n : Nat) : IO (List String) := do
  let mut keys : List String := []
  for _ in List.range n do
    let k <- generateKey
    keys := keys ++ [k]
  return keys

/-- Demo entry point. -/
def demo : IO Unit := do
  IO.println "=== CD Key Generator ==="
  IO.println ""
  let keys <- generateBatch 5
  for key in keys do
    IO.println s!"  {key}  valid={validateKey key}"
  match keys.head? with
  | none => pure ()
  | some first =>
    let tampered := if first.get 0 == 'Z'
      then String.mk ('A' :: (first.toList.drop 1))
      else String.mk ('Z' :: (first.toList.drop 1))
    IO.println s!"\n  Tampered: {tampered}  valid={validateKey tampered}"

end Text
