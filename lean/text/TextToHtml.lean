/- Text to HTML Generator — convert formatted plain text to HTML. -/

namespace Text

/-- Escape HTML entities: &, <, > -/
def escapeHtml (s : String) : String :=
  s.replace "&" "&amp;"
  |>.replace "<" "&lt;"
  |>.replace ">" "&gt;"

/-- Count leading '#' characters in a string. -/
private def countHashes (s : String) : Nat :=
  s.data.takeWhile (· == '#') |>.length

/-- Apply inline formatting: code spans, bold, italic, links.
    Uses simple character-by-character scanning. -/
partial def applyInline (s : String) : String :=
  let s := applyPattern s '`' '`' "<code>" "</code>"
  let s := applyPatternDouble s '*' "<strong>" "</strong>"
  let s := applyPattern s '*' '*' "<em>" "</em>"
  applyLinks s
where
  /-- Replace `delim`..`delim` with open/close tags. -/
  applyPattern (input : String) (open close : Char) (tagOpen tagClose : String) : String :=
    let rec go (chars : List Char) (acc : List Char) (inside : Bool) : List Char :=
      match chars with
      | [] => acc.reverse
      | c :: rest =>
        if c == (if inside then close else open) && !rest.isEmpty then
          if inside then
            go rest (tagClose.data.reverse ++ acc) false
          else
            go rest (tagOpen.data.reverse ++ acc) true
        else
          go c :: rest |>.drop 1 |> fun r => go r (c :: acc) inside
    -- Simple regex-free approach: scan for delimiters
    ⟨go input.data [] false⟩
  /-- Replace **..** with strong tags (two-char delimiter). -/
  applyPatternDouble (input : String) (delim : Char) (tagOpen tagClose : String) : String :=
    let rec go (chars : List Char) (acc : List Char) (inside : Bool) : List Char :=
      match chars with
      | [] => acc.reverse
      | c1 :: c2 :: rest =>
        if c1 == delim && c2 == delim then
          if inside then
            go rest (tagClose.data.reverse ++ acc) false
          else
            go rest (tagOpen.data.reverse ++ acc) true
        else
          go (c2 :: rest) (c1 :: acc) inside
      | c :: rest => go rest (c :: acc) inside
    ⟨go input.data [] false⟩
  /-- Replace [text](url) with <a href="url">text</a>. -/
  applyLinks (input : String) : String :=
    let rec go (chars : List Char) (acc : List Char) : List Char :=
      match chars with
      | [] => acc.reverse
      | '[' :: rest =>
        match rest.dropWhile (· != ']') with
        | ']' :: '(' :: afterBracket =>
          let linkText := rest.takeWhile (· != ']')
          let url := afterBracket.takeWhile (· != ')')
          let remaining := afterBracket.dropWhile (· != ')') |>.drop 1
          let tag := s!"<a href=\"{⟨url⟩}\">{⟨linkText⟩}</a>"
          go remaining (tag.data.reverse ++ acc)
        | _ => go rest ('[' :: acc)
      | c :: rest => go rest (c :: acc)
    ⟨go input.data []⟩

/-- Check if a trimmed line is a header. Returns (level, content) or none. -/
private def parseHeader (line : String) : Option (Nat × String) :=
  let hashes := countHashes line
  if hashes >= 1 && hashes <= 3 && line.data.length > hashes + 1
      && line.data[hashes]! == ' ' then
    some (hashes, ⟨line.data.drop (hashes + 1)⟩)
  else
    none

/-- Build an HTML unordered list from items. -/
private def buildList (items : List String) : String :=
  let inner := items.map fun item => s!"<li>{applyInline item}</li>"
  s!"<ul>\n{"\n".intercalate inner}\n</ul>"

/-- Convert formatted plain text to HTML.

Supports paragraphs, bold, italic, headers (H1-H3), links,
unordered lists, inline code, line breaks, and HTML entity escaping. -/
def textToHtml (text : String) : String :=
  let lines := text.splitOn "\n"
  let (blocks, para, listItems) := lines.foldl (init := ([], [], [])) fun (blocks, para, listItems) line =>
    let trimmed := line.trim
    if trimmed.isEmpty then
      -- Blank line: flush list and paragraph
      let blocks := match listItems.reverse with
        | [] => blocks
        | items => blocks ++ [buildList items]
      let blocks := match para.reverse with
        | [] => blocks
        | parts => blocks ++ [s!"<p>{applyInline ("<br>\n".intercalate parts)}</p>"]
      (blocks, [], [])
    else match parseHeader trimmed with
      | some (level, content) =>
        let blocks := match listItems.reverse with
          | [] => blocks
          | items => blocks ++ [buildList items]
        let blocks := match para.reverse with
          | [] => blocks
          | parts => blocks ++ [s!"<p>{applyInline ("<br>\n".intercalate parts)}</p>"]
        let escaped := escapeHtml content
        (blocks ++ [s!"<h{level}>{applyInline escaped}</h{level}>"], [], [])
      | none =>
        if trimmed.startsWith "- " then
          let blocks := match para.reverse with
            | [] => blocks
            | parts => blocks ++ [s!"<p>{applyInline ("<br>\n".intercalate parts)}</p>"]
          (blocks, [], listItems ++ [escapeHtml (trimmed.drop 2)])
        else
          let blocks := match listItems.reverse with
            | [] => blocks
            | items => blocks ++ [buildList items]
          (blocks, para ++ [escapeHtml trimmed], [])
  -- Final flush
  let blocks := match listItems.reverse with
    | [] => blocks
    | items => blocks ++ [buildList items]
  let blocks := match para.reverse with
    | [] => blocks
    | parts => blocks ++ [s!"<p>{applyInline ("<br>\n".intercalate parts)}</p>"]
  "\n".intercalate blocks

end Text
