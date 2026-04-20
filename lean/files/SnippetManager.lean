-- Code Snippet Manager — templated code fragments with tab-stop expansion.

namespace Files.SNIP

structure TabStop where
  number : Nat
  offset : Nat
  length : Nat
  deriving Repr, BEq

structure Expansion where
  text : String
  stops : List TabStop
  deriving Repr

structure Snippet where
  trigger : String
  language : String
  tags : List String := []
  body : String
  description : String := ""
  deriving Repr

structure SnippetLibrary where
  snippets : List Snippet := []
  deriving Repr

def SnippetLibrary.add (l : SnippetLibrary) (s : Snippet) : SnippetLibrary :=
  { snippets := l.snippets ++ [s] }

def SnippetLibrary.findByTrigger (l : SnippetLibrary) (trigger lang : String) : Option Snippet :=
  l.snippets.find? fun s => s.trigger == trigger ∧ s.language == lang

def SnippetLibrary.findByLanguage (l : SnippetLibrary) (lang : String) : List Snippet :=
  (l.snippets.filter fun s => s.language == lang).mergeSort (fun a b => a.trigger < b.trigger)

def SnippetLibrary.findByTag (l : SnippetLibrary) (tag : String) : List Snippet :=
  (l.snippets.filter fun s => s.tags.contains tag).mergeSort (fun a b => a.trigger < b.trigger)

partial def findMatching (chars : List Char) (open : Nat) : Option Nat :=
  let rec loop (i : Nat) (depth : Nat) : Option Nat :=
    match chars.get? i with
    | none => none
    | some '{' => loop (i + 1) (depth + 1)
    | some '}' =>
      if depth == 1 then some i
      else loop (i + 1) (depth - 1)
    | some _ => loop (i + 1) depth
  loop (open + 1) 1

partial def parseAndExpand (body : String) : Expansion :=
  let chars := body.toList
  let n := chars.length
  let rec go (i : Nat) (out : String) (stops : List TabStop) : (String × List TabStop) :=
    if i ≥ n then (out, stops)
    else
      match chars.get? i, chars.get? (i + 1) with
      | some '$', some '{' =>
        match findMatching chars (i + 1) with
        | some close =>
          let inside := String.mk (chars.drop (i + 2) |>.take (close - i - 2))
          let (numStr, def') :=
            if inside.contains ':' then
              let colon := inside.splitOn ":"
              (colon.head!, ":".intercalate colon.tail!)
            else (inside, "")
          let number := numStr.toNat?.getD 0
          let offset := out.length
          let length := def'.length
          go (close + 1) (out ++ def')
             (stops ++ [{ number, offset, length }])
        | none => go (i + 1) (out ++ String.mk [chars.get! i]) stops
      | some '$', some c =>
        if c.isDigit then
          let rec scanDigits (j : Nat) (acc : String) : (Nat × String) :=
            match chars.get? j with
            | some ch => if ch.isDigit then scanDigits (j + 1) (acc.push ch) else (j, acc)
            | none => (j, acc)
          let (j, numStr) := scanDigits (i + 1) ""
          let number := numStr.toNat?.getD 0
          go j out (stops ++ [{ number, offset := out.length, length := 0 }])
        else go (i + 1) (out ++ String.mk [chars.get! i]) stops
      | some c, _ => go (i + 1) (out ++ String.mk [c]) stops
      | none, _ => (out, stops)
  let (text, stops) := go 0 "" []
  let sorted := stops.mergeSort fun a b =>
    if a.number == 0 ∧ b.number == 0 then false
    else if a.number == 0 then false
    else if b.number == 0 then true
    else a.number < b.number
  { text, stops := sorted }

def Snippet.expand (s : Snippet) : Expansion := parseAndExpand s.body

end Files.SNIP
