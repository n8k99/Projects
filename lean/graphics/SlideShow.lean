-- Slide Show — presentation navigation with dual-track content.

namespace Graphics.SS

inductive ElementKind where | heading | bullet | image | spacer | code
  deriving Repr, BEq

structure SlideElement where
  kind : ElementKind
  text : String := ""
  level : Nat := 1
  indent : Nat := 0
  src : String := ""
  alt : String := ""
  language : String := ""
  lines : Nat := 0
  deriving Repr

inductive Transition where | none | fade | slide | dissolve
  deriving Repr, BEq

structure Slide where
  title : String
  content : List SlideElement := []
  notes : String := ""
  transition : Transition := .none
  deriving Repr

def Slide.heading (s : Slide) (text : String) (level : Nat) : Slide :=
  { s with content := s.content ++ [{ kind := .heading, text, level }] }

def Slide.bullet (s : Slide) (text : String) (indent : Nat) : Slide :=
  { s with content := s.content ++ [{ kind := .bullet, text, indent }] }

def Slide.image (s : Slide) (src alt : String) : Slide :=
  { s with content := s.content ++ [{ kind := .image, src, alt }] }

def Slide.code (s : Slide) (text language : String) : Slide :=
  { s with content := s.content ++ [{ kind := .code, text, language }] }

def Slide.withNotes (s : Slide) (notes : String) : Slide :=
  { s with notes }

inductive Mode where | speaker | audience | handout
  deriving Repr, BEq

structure Presentation where
  title : String
  slides : List Slide := []
  currentIndex : Nat := 0
  deriving Repr

def Presentation.addSlide (p : Presentation) (s : Slide) : Presentation :=
  { p with slides := p.slides ++ [s] }

def Presentation.slideCount (p : Presentation) : Nat := p.slides.length

def Presentation.current (p : Presentation) : Option Slide :=
  p.slides.get? p.currentIndex

def Presentation.advance (p : Presentation) : (Bool × Presentation) :=
  if p.currentIndex + 1 < p.slides.length then
    (true, { p with currentIndex := p.currentIndex + 1 })
  else (false, p)

def Presentation.back (p : Presentation) : (Bool × Presentation) :=
  if p.currentIndex > 0 then
    (true, { p with currentIndex := p.currentIndex - 1 })
  else (false, p)

def Presentation.goto (p : Presentation) (i : Nat) : (Bool × Presentation) :=
  if i < p.slides.length then (true, { p with currentIndex := i })
  else (false, p)

def Presentation.progress (p : Presentation) : (Nat × Nat) :=
  if p.slides.isEmpty then (0, 0)
  else (p.currentIndex + 1, p.slides.length)

def elementText (e : SlideElement) : String :=
  match e.kind with
  | .heading | .bullet | .code => e.text
  | .image => e.alt
  | .spacer => ""

def renderElement (e : SlideElement) : String :=
  match e.kind with
  | .heading =>
    let n := max 1 (min 6 e.level)
    (String.mk (List.replicate n '#')) ++ " " ++ e.text
  | .bullet =>
    (String.mk (List.replicate (2 * e.indent) ' ')) ++ "- " ++ e.text
  | .image => s!"![{e.alt}]({e.src})"
  | .code => s!"```{e.language}\n{e.text}\n```"
  | .spacer => String.mk (List.replicate e.lines '\n')

def renderSlide (s : Slide) (mode : Mode) : String :=
  let titleLine := s.title ++ "\n"
  let width := min 80 s.title.length
  let sep := (String.mk (List.replicate width '=')) ++ "\n"
  let body := s.content.foldl (init := "") fun acc e =>
    acc ++ renderElement e ++ "\n"
  let notes :=
    if (mode == .speaker ∨ mode == .handout) ∧ !s.notes.isEmpty then
      "\n[Speaker notes]\n" ++ s.notes ++ "\n"
    else ""
  titleLine ++ sep ++ body ++ notes

def Presentation.renderCurrent (p : Presentation) (mode : Mode) : String :=
  match p.current with
  | some s => renderSlide s mode
  | none => ""

def Presentation.renderHandout (p : Presentation) : String :=
  let header := s!"# {p.title}\n\n"
  let slides := p.slides.mapIdx fun i s =>
    s!"--- Slide {i + 1} ---\n" ++ renderSlide s .handout ++ "\n"
  header ++ String.join slides

def Presentation.search (p : Presentation) (needle : String) : List Nat :=
  let lc := needle.toLower
  let filter := fun (i : Nat) (s : Slide) =>
    if s.title.toLower.splitOn lc |>.length > 1 ∨ s.title.toLower == lc then some i
    else if s.content.any (fun e => (elementText e).toLower.splitOn lc |>.length > 1)
         then some i
    else none
  (p.slides.mapIdx filter).filterMap id

end Graphics.SS
