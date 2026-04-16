/- Check if Palindrome — detect and find palindromic strings. -/

namespace Text

/-- Check whether a string reads the same forwards and backwards (exact). -/
def isPalindromeExact (s : String) : Bool :=
  s == ⟨s.data.reverse⟩

/-- Strip non-alphanumeric characters and convert to lowercase. -/
private def normalize (s : String) : String :=
  let filtered := s.data.filter fun c => c.isAlpha || c.isDigit
  ⟨filtered.map Char.toLower⟩

/-- Check whether a string is a palindrome, ignoring case, spaces, and punctuation. -/
def isPalindromeNormalized (s : String) : Bool :=
  let n := normalize s
  n == ⟨n.data.reverse⟩

/-- Expand around center indices to find a palindromic span.
    Returns (start, length) of the palindrome found. -/
private def expandAroundCenter (chars : Array Char) (left right : Nat) : Nat × Nat :=
  let rec go (l r : Nat) (fuel : Nat) : Nat × Nat :=
    match fuel with
    | 0 => (l + 1, if r > l + 1 then r - l - 1 else 0)
    | fuel' + 1 =>
      if l > 0 && r < chars.size && chars[l]! == chars[r]! then
        go (l - 1) (r + 1) fuel'
      else
        (l + 1, if r > l + 1 then r - l - 1 else 0)
  -- Handle initial check before recursing
  if right < chars.size && chars[left]! == chars[right]! then
    let rec goInner (l r : Nat) (fuel : Nat) : Nat × Nat :=
      match fuel with
      | 0 => (l, r - l + 1)
      | fuel' + 1 =>
        if l > 0 && r + 1 < chars.size && chars[l - 1]! == chars[r + 1]! then
          goInner (l - 1) (r + 1) fuel'
        else
          (l, r - l + 1)
    goInner left right chars.size
  else
    (left, 0)

/-- Find the longest palindromic substring using expand-around-center. -/
def longestPalindromeSubstring (s : String) : String :=
  if s.length < 2 then s
  else
    let chars := s.data.toArray
    let n := chars.size
    let rec search (i : Nat) (bestStart bestLen : Nat) (fuel : Nat) : Nat × Nat :=
      match fuel with
      | 0 => (bestStart, bestLen)
      | fuel' + 1 =>
        if i >= n then (bestStart, bestLen)
        else
          -- Odd-length
          let (s1, l1) := expandAroundCenter chars i i
          let (bs2, bl2) := if l1 > bestLen then (s1, l1) else (bestStart, bestLen)
          -- Even-length
          let (s2, l2) := expandAroundCenter chars i (i + 1)
          let (bs3, bl3) := if l2 > bl2 then (s2, l2) else (bs2, bl2)
          search (i + 1) bs3 bl3 fuel'
    let (start, len) := search 0 0 1 n
    ⟨(chars.toList.drop start).take len⟩

end Text
