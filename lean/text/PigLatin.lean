/- Pig Latin — translate English text to Pig Latin. -/

namespace Text

def isVowel (c : Char) : Bool :=
  "aeiouAEIOU".data.contains c

def isAlpha (c : Char) : Bool :=
  (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')

/-- Split trailing non-alpha punctuation from a word. Returns (core, trail). -/
def splitTrail (word : String) : String × String :=
  let chars := word.data
  let revChars := chars.reverse
  let trailRev := revChars.takeWhile (fun c => !isAlpha c)
  let coreLen := chars.length - trailRev.length
  let core := String.mk (chars.take coreLen)
  let trail := String.mk trailRev.reverse
  (core, trail)

/-- Convert a single English word to Pig Latin.
    Consonant cluster start: move cluster to end, add "ay".
    Vowel start: add "yay".
    Preserves capitalization and trailing punctuation. -/
def pigLatinWord (word : String) : String :=
  if word.isEmpty then word
  else
    let (core, trail) := splitTrail word
    if core.isEmpty then word
    else
      let wasCap := core.data.head!.isUpper
      let lower := core.toLower
      let chars := lower.data
      let result :=
        if isVowel chars.head! then
          lower ++ "yay"
        else
          let clusterEnd := chars.takeWhile (fun c => !isVowel c) |>.length
          let rest := String.mk (chars.drop clusterEnd)
          let cluster := String.mk (chars.take clusterEnd)
          rest ++ cluster ++ "ay"
      let result :=
        if wasCap then
          let resultChars := result.data
          match resultChars with
          | [] => result
          | (h :: t) => String.mk (h.toUpper :: t)
        else result
      result ++ trail

/-- Convert an English sentence to Pig Latin. -/
def pigLatin (text : String) : String :=
  let words := text.splitOn " "
  String.intercalate " " (words.map pigLatinWord)

end Text
