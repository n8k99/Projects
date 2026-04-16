/- Count Vowels — analyze vowel distribution in text. -/

namespace Text

/-- Returns true if the character is a standard vowel (a, e, i, o, u). -/
def isVowel (c : Char) : Bool :=
  let lower := c.toLower
  lower == 'a' || lower == 'e' || lower == 'i' || lower == 'o' || lower == 'u'

/-- Returns true if the character is a vowel, optionally treating 'y' as a vowel. -/
def isVowelOrY (c : Char) (includeY : Bool) : Bool :=
  isVowel c || (includeY && c.toLower == 'y')

/-- Count total vowels in a string. If `includeY` is true, 'y' counts as a vowel. -/
def countVowels (text : String) (includeY : Bool := false) : Nat :=
  text.toList.filter (isVowelOrY · includeY) |>.length

/-- Return a list of (vowel, count) pairs for the given text. -/
def vowelBreakdown (text : String) (includeY : Bool := false) : List (Char × Nat) :=
  let vowels := if includeY then ['a', 'e', 'i', 'o', 'u', 'y'] else ['a', 'e', 'i', 'o', 'u']
  let lower := text.toList.map Char.toLower
  vowels.map fun v => (v, lower.filter (· == v) |>.length)

/-- Count the number of alphabetic characters in a string. -/
def alphaCount (text : String) : Nat :=
  text.toList.filter Char.isAlpha |>.length

/-- Return the ratio of vowels to total alphabetic characters as a Float.
    Returns 0.0 if there are no alphabetic characters. -/
def vowelRatio (text : String) : Float :=
  let ac := alphaCount text
  if ac == 0 then 0.0
  else (countVowels text).toFloat / ac.toFloat

-- Demo
#eval do
  let demos := ["Hello World", "The quick brown fox", "Why try fly by my dry cry?"]
  for phrase in demos do
    IO.println s!"\n--- \"{phrase}\" ---"
    IO.println s!"  vowels:      {countVowels phrase}"
    IO.println s!"  vowels (+y): {countVowels phrase (includeY := true)}"
    IO.println s!"  breakdown:   {vowelBreakdown phrase}"
    IO.println s!"  breakdown (+y): {vowelBreakdown phrase (includeY := true)}"
    IO.println s!"  ratio:       {vowelRatio phrase}"

end Text
