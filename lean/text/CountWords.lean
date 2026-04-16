/- Count Words in a String — word-level text analysis. -/

namespace Text

/-- Returns true if the character is ASCII punctuation. -/
def isPunct (c : Char) : Bool :=
  "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".toList.contains c

/-- Strip leading and trailing punctuation from a string. -/
def stripPunct (s : String) : String :=
  let chars := s.toList
  let trimmed := chars.dropWhile isPunct |>.reverse |>.dropWhile isPunct |>.reverse
  ⟨trimmed⟩

/-- Clean a word: strip punctuation and lowercase. -/
def cleanWord (word : String) : String :=
  (stripPunct word).toLower

/-- Count words in text (split on whitespace). -/
def wordCount (text : String) : Nat :=
  let words := text.splitOn " " |>.filter (· ≠ "")
  words.length

/-- Count occurrences of each word (case-insensitive, strip punctuation).
    Returns a list of (word, count) pairs. -/
def wordFrequency (text : String) : List (String × Nat) :=
  let words := text.splitOn " "
    |>.map cleanWord
    |>.filter (· ≠ "")
  let unique := words.eraseDups
  unique.map fun w => (w, words.filter (· == w) |>.length)

/-- Count distinct words (case-insensitive, strip punctuation). -/
def uniqueWords (text : String) : Nat :=
  (wordFrequency text).length

/-- Return the mean word length (punctuation stripped).
    Returns 0.0 if there are no words. -/
def averageWordLength (text : String) : Float :=
  let words := text.splitOn " "
    |>.map cleanWord
    |>.filter (· ≠ "")
  if words.isEmpty then 0.0
  else
    let totalLen := words.map (·.length) |>.foldl (· + ·) 0
    totalLen.toFloat / words.length.toFloat

-- Demo
#eval do
  let demos := [
    "The quick brown fox jumps over the lazy dog",
    "To be or not to be, that is the question."
  ]
  for phrase in demos do
    IO.println s!"\n--- \"{phrase}\" ---"
    IO.println s!"  word count:        {wordCount phrase}"
    IO.println s!"  word frequency:    {wordFrequency phrase}"
    IO.println s!"  unique words:      {uniqueWords phrase}"
    IO.println s!"  avg word length:   {averageWordLength phrase}"

end Text
