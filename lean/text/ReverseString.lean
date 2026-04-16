/- Reverse a String — reverse character order. -/

namespace Text

/-- Reverse the characters in a string. -/
def reverseString (s : String) : String :=
  ⟨s.data.reverse⟩

/-- Reverse the order of words in a string, keeping each word intact. -/
def reverseWords (s : String) : String :=
  let words := s.splitOn " " |>.filter (· ≠ "")
  " ".intercalate words.reverse

/-- Check whether a string reads the same forwards and backwards. -/
def isPalindrome (s : String) : Bool :=
  s == reverseString s

end Text
