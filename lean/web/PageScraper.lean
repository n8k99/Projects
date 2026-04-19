-- Page Scraper — parse HTML-like markup into a tree, query with selectors.
-- Forgiving parser: unknown constructs become text, stray close tags drop.
-- Selectors: CSS subset (tag, .class, #id, combos, descendant via whitespace).

namespace Web

inductive ScrapedNode where
  | element (tag : String) (attrs : List (String × String)) (children : List ScrapedNode)
  | text (s : String)
  deriving Repr

def voidTags : List String :=
  ["area", "base", "br", "col", "embed", "hr", "img", "input",
   "link", "meta", "source", "track", "wbr"]

structure ParseState where
  s : String
  i : Nat := 0

private def peek (st : ParseState) : Option Char :=
  st.s.get? ⟨st.i⟩

private def startsWith (st : ParseState) (prefix : String) : Bool :=
  (st.s.drop st.i).startsWith prefix

private partial def consumeWhile (st : ParseState) (pred : Char → Bool)
    : String × ParseState :=
  let rec go (i : Nat) : Nat :=
    match st.s.get? ⟨i⟩ with
    | some c => if pred c then go (i + 1) else i
    | none => i
  let end_ := go st.i
  (st.s.extract ⟨st.i⟩ ⟨end_⟩, { st with i := end_ })

private def isAlnumDash (c : Char) : Bool :=
  c.isAlphanum || c == '-'

private def isAttrNameChar (c : Char) : Bool :=
  c.isAlphanum || c == '-' || c == '_'

-- Simplified: produce a flat list of elements/text, ignoring self-closing
-- and descendant semantics for brevity. Full parser is in Rust/Python/Go/CL.
-- Lean version demonstrates the tree shape; full parsing ergonomics are
-- unpleasant in pure Lean and not the primary teaching point.

partial def parseHTML (html : String) : List ScrapedNode :=
  let rec go (st : ParseState) (stopTag : Option String)
      : List ScrapedNode × ParseState :=
    let mut acc : List ScrapedNode := []
    let mut cur := st
    let mut running := true
    while running do
      match peek cur with
      | none => running := false
      | some '<' =>
        -- Check stop tag
        match stopTag with
        | some t =>
          if startsWith cur s!"</{t}" then
            running := false
          else
            let (node?, next) := parseTagOrText cur
            cur := next
            match node? with
            | some n => acc := n :: acc
            | none => pure ()
        | none =>
          let (node?, next) := parseTagOrText cur
          cur := next
          match node? with
          | some n => acc := n :: acc
          | none => pure ()
      | _ =>
        let (txt, next) := consumeWhile cur (· ≠ '<')
        cur := next
        if !txt.isEmpty then acc := .text txt :: acc
    (acc.reverse, cur)
  and parseTagOrText (st : ParseState) : Option ScrapedNode × ParseState :=
    if startsWith st "</" then
      -- Stray close tag — skip
      let (_, next) := consumeWhile st (· ≠ '>')
      let next := if peek next = some '>' then { next with i := next.i + 1 } else next
      (none, next)
    else
      let afterLt := { st with i := st.i + 1 }
      match peek afterLt with
      | some c => if c.isAlpha then parseElement st else (none, { st with i := st.i + 1 })
      | none => (none, st)
  and parseElement (st : ParseState) : Option ScrapedNode × ParseState :=
    let st := { st with i := st.i + 1 } -- consume <
    let (tag, st) := consumeWhile st isAlnumDash
    if tag.isEmpty then (none, st)
    else
      let tag := tag.toLower
      let (attrs, st) := parseAttrs st []
      if startsWith st "/>" then
        let st := { st with i := st.i + 2 }
        (some (.element tag attrs []), st)
      else if startsWith st ">" then
        let st := { st with i := st.i + 1 }
        if voidTags.contains tag then
          (some (.element tag attrs []), st)
        else
          let (children, st) := go st (some tag)
          let st :=
            if startsWith st s!"</{tag}" then
              let (_, s2) := consumeWhile st (· ≠ '>')
              if peek s2 = some '>' then { s2 with i := s2.i + 1 } else s2
            else st
          (some (.element tag attrs children), st)
      else (none, st)
  and parseAttrs (st : ParseState) (acc : List (String × String))
      : List (String × String) × ParseState :=
    let (_, st) := consumeWhile st Char.isWhitespace
    match peek st with
    | none => (acc.reverse, st)
    | some '>' | some '/' => (acc.reverse, st)
    | _ =>
      let (name, st) := consumeWhile st isAttrNameChar
      if name.isEmpty then
        parseAttrs { st with i := st.i + 1 } acc
      else
        let name := name.toLower
        let (_, st) := consumeWhile st Char.isWhitespace
        let (value, st) :=
          if startsWith st "=" then
            let st := { st with i := st.i + 1 }
            let (_, st) := consumeWhile st Char.isWhitespace
            match peek st with
            | some q => if q == '"' || q == '\'' then
                let st := { st with i := st.i + 1 }
                let (v, st) := consumeWhile st (· ≠ q)
                let st := if peek st = some q then { st with i := st.i + 1 } else st
                (v, st)
              else
                consumeWhile st fun c => !c.isWhitespace && c ≠ '>' && c ≠ '/'
            | none => ("", st)
          else ("", st)
        parseAttrs st ((name, value) :: acc)
  (go { s := html } none).1

partial def textOf : ScrapedNode → String
  | .text s => s
  | .element _ _ children => children.foldl (· ++ textOf ·) ""

structure SimpleSel where
  tag : Option String := none
  cls : Option String := none
  id : Option String := none

private def parseSimple (s : String) : SimpleSel :=
  let mut simp : SimpleSel := {}
  let mut buf := ""
  let mut kind : Char := ' '
  let flush : Unit → Unit := fun _ =>
    if !buf.isEmpty then
      match kind with
      | '.' => simp := { simp with cls := some buf }
      | '#' => simp := { simp with id := some buf }
      | _   => simp := { simp with tag := some buf.toLower }
      buf := ""
  for c in s.toList do
    if c = '.' || c = '#' then
      flush ()
      kind := c
    else
      buf := buf.push c
  flush ()
  simp

private def lookupAttr (attrs : List (String × String)) (k : String) : Option String :=
  (attrs.find? (·.1 == k)).map (·.2)

private def simpMatches (simp : SimpleSel) : ScrapedNode → Bool
  | .element tag attrs _ =>
    (simp.tag.all (· == tag)) &&
    (simp.id.all (fun i => lookupAttr attrs "id" == some i)) &&
    (simp.cls.all (fun c =>
      match lookupAttr attrs "class" with
      | none => false
      | some cs => cs.splitOn " " |>.contains c))
  | _ => false

partial def select (nodes : List ScrapedNode) (selector : String)
    : List ScrapedNode :=
  let chain := (selector.splitOn " ").filter (!·.isEmpty) |>.map parseSimple
  let rec rec (ns : List ScrapedNode) (idx : Nat) : List ScrapedNode :=
    ns.foldl (init := []) fun acc n =>
      match n with
      | .element _ _ children =>
        let m := simpMatches (chain.get! idx) n
        if m && idx == chain.length - 1 then
          acc ++ [n] ++ rec children idx
        else if m then
          acc ++ rec children (idx + 1)
        else
          acc ++ rec children idx
      | _ => acc
  if chain.isEmpty then [] else rec nodes 0

end Web
