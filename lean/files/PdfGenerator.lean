-- PDF Generator — document pagination via block flow layout.

namespace Files.PDFG

inductive Block where
  | heading (text : String) (level : Nat)
  | paragraph (text : String)
  | spacer (height : Nat)
  | pageBreak
  deriving Repr

structure PageSize where
  widthChars : Nat
  heightLines : Nat
  deriving Repr

def a4Like : PageSize := { widthChars := 80, heightLines := 66 }

structure RenderedLine where
  text : String
  y : Nat
  isHeading : Bool
  deriving Repr

structure Page where
  lines : List RenderedLine := []
  number : Nat := 1
  deriving Repr

structure Document where
  pageSize : PageSize
  blocks : List Block := []
  deriving Repr

def renderHeading (text : String) (level : Nat) : String :=
  match level with
  | 1 => "# " ++ text
  | 2 => "## " ++ text
  | _ => "### " ++ text

def wrapText (text : String) (width : Nat) : List String :=
  let words := text.splitOn " " |>.filter (· ≠ "")
  let (lines, cur) := words.foldl (init := (([] : List String), ""))
    fun (lines, cur) word =>
      if cur == "" then (lines, word)
      else if cur.length + 1 + word.length ≤ width then
        (lines, cur ++ " " ++ word)
      else (lines ++ [cur], word)
  let all := if cur == "" then lines else lines ++ [cur]
  if all.isEmpty then [""] else all

partial def render (doc : Document) : List Page :=
  let rec go (blocks : List Block) (pages : List Page) (current : Page) (y : Nat)
      : List Page :=
    let pushPage (pages : List Page) (current : Page) (y : Nat)
        : (List Page × Page × Nat) :=
      if current.lines.isEmpty then (pages, current, y)
      else (pages ++ [current],
            { lines := [], number := current.number + 1 }, 0)
    match blocks with
    | [] => if current.lines.isEmpty then pages else pages ++ [current]
    | block :: rest =>
      match block with
      | .pageBreak =>
        let (pages, current, y) := pushPage pages current y
        go rest pages current y
      | .spacer h =>
        if y + h > doc.pageSize.heightLines then
          let (pages, current, y) := pushPage pages current y
          go rest pages current y
        else go rest pages current (y + h)
      | .heading text level =>
        let (pages, current, y) :=
          if y + 2 > doc.pageSize.heightLines then pushPage pages current y
          else (pages, current, y)
        let line : RenderedLine :=
          { text := renderHeading text level, y := y, isHeading := true }
        let current := { current with lines := current.lines ++ [line] }
        go rest pages current (y + 2)
      | .paragraph text =>
        let lines := wrapText text doc.pageSize.widthChars
        let (pages, current, y) := lines.foldl
          (init := (pages, current, y)) fun (pages, current, y) line =>
            let (pages, current, y) :=
              if y ≥ doc.pageSize.heightLines then pushPage pages current y
              else (pages, current, y)
            let rl : RenderedLine := { text := line, y := y, isHeading := false }
            (pages, { current with lines := current.lines ++ [rl] }, y + 1)
        let y := if y < doc.pageSize.heightLines then y + 1 else y
        go rest pages current y
  go doc.blocks [] { number := 1 } 0

def pageCount (doc : Document) : Nat := (render doc).length

end Files.PDFG
