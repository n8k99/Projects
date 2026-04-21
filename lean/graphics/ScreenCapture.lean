-- Screen Capture Program — region + delay FSM + annotations + ring history.

import Graphics.Grayscale

namespace Graphics.Capture

open Graphics.Gray

inductive Region where
  | fullScreen
  | rect (x y w h : Nat)
  | window (id : Nat)
  deriving Repr

inductive Annotation where
  | rectangle (x y w h : Nat) (color : Rgb) (stroke : Nat)
  | highlight (x y w h : Nat) (color : Rgb) (opacity : Nat)
  | text (x y : Nat) (txt : String) (color : Rgb)
  | arrow (x1 y1 x2 y2 : Nat) (color : Rgb)
  deriving Repr

structure CaptureSpec where
  region : Region
  delayMs : Nat := 0
  includeCursor : Bool := false
  annotations : List Annotation := []
  deriving Repr

inductive RecorderState where
  | idle | armed | counting | capturing | captured
  deriving Repr, BEq

structure Capture where
  id : Nat
  spec : CaptureSpec
  image : Image
  capturedAtMs : Nat
  deriving Repr

inductive RecorderEventKind where
  | armed | countdownTick | captured | evicted | captureFailed
  deriving Repr, BEq

structure RecorderEvent where
  kind : RecorderEventKind
  id : Nat
  detail : String := ""
  deriving Repr

structure Recorder where
  state : RecorderState := .idle
  pendingSpec : Option CaptureSpec := none
  pendingId : Nat := 0
  countdownRemainingMs : Nat := 0
  history : List Capture := []
  historyMax : Nat
  elapsedMs : Nat := 0
  events : List RecorderEvent := []
  nextId : Nat := 1
  deriving Repr

def Recorder.new (historyMax : Nat) : Recorder := { historyMax }

def Recorder.arm (r : Recorder) (spec : CaptureSpec) : Recorder × Nat :=
  let id := r.nextId
  let newState := if spec.delayMs == 0 then RecorderState.capturing else .counting
  let r' := { r with
    pendingId := id,
    pendingSpec := some spec,
    countdownRemainingMs := spec.delayMs,
    state := newState,
    events := r.events ++ [{
      kind := .armed, id,
      detail := s!"delay={spec.delayMs}ms"
    }],
    nextId := r.nextId + 1
  }
  (r', id)

def Recorder.tick (r : Recorder) (ms : Nat) : Recorder :=
  let r1 := { r with elapsedMs := r.elapsedMs + ms }
  if r1.state == .counting then
    let (newRem, newState) :=
      if r1.countdownRemainingMs > ms
      then (r1.countdownRemainingMs - ms, RecorderState.counting)
      else (0, RecorderState.capturing)
    { r1 with
      countdownRemainingMs := newRem,
      state := newState,
      events := r1.events ++ [{
        kind := .countdownTick, id := r1.pendingId,
        detail := s!"remaining={newRem}ms"
      }]
    }
  else r1

partial def getPixelOpt (img : Image) (x y : Nat) : Option Rgb :=
  if x < img.width ∧ y < img.height
  then img.pixels.get? (y * img.width + x)
  else none

partial def setPixel (img : Image) (x y : Nat) (color : Rgb) : Image :=
  if x < img.width ∧ y < img.height then
    let idx := y * img.width + x
    { img with pixels := img.pixels.modify idx (fun _ => color) }
  else img

def cropImg (img : Image) (x0 y0 w h : Nat) : Image :=
  let arr := (List.range h).foldl (init := #[]) fun acc y =>
    (List.range w).foldl (init := acc) fun acc2 x =>
      match getPixelOpt img (x0 + x) (y0 + y) with
      | none => acc2.push { r := 0, g := 0, b := 0 }
      | some p => acc2.push p
  { width := w, height := h, pixels := arr.toList }

def clampU8 (v : Int) : Nat :=
  if v < 0 then 0 else if v > 255 then 255 else v.toNat

def blendChan (a b alpha : Nat) : Nat :=
  (a * (255 - alpha) + b * alpha) / 255

def blendRect (img : Image) (x y w h : Nat) (color : Rgb) (opacity : Nat) : Image :=
  (List.range h).foldl (init := img) fun imgAcc dy =>
    (List.range w).foldl (init := imgAcc) fun imgAcc2 dx =>
      let px := x + dx
      let py := y + dy
      match getPixelOpt imgAcc2 px py with
      | none => imgAcc2
      | some cur =>
        let blended : Rgb := {
          r := blendChan cur.r color.r opacity,
          g := blendChan cur.g color.g opacity,
          b := blendChan cur.b color.b opacity
        }
        setPixel imgAcc2 px py blended

def drawRectOutline (img : Image) (x y w h : Nat) (color : Rgb) (stroke : Nat)
    : Image :=
  let strokeN := max stroke 1
  (List.range strokeN).foldl (init := img) fun imgAcc s =>
    let imgAcc1 := (List.range w).foldl (init := imgAcc) fun im dx =>
      let im1 := setPixel im (x + dx) (y + s) color
      if h > s then setPixel im1 (x + dx) (y + h - 1 - s) color else im1
    (List.range h).foldl (init := imgAcc1) fun im dy =>
      let im1 := setPixel im (x + s) (y + dy) color
      if w > s then setPixel im1 (x + w - 1 - s) (y + dy) color else im1

partial def drawLine (img : Image) (x1 y1 x2 y2 : Nat) (color : Rgb) : Image :=
  let mut imgM := img
  let mut x : Int := x1.toInt
  let mut y : Int := y1.toInt
  let x2i : Int := x2.toInt
  let y2i : Int := y2.toInt
  let dx := (x2i - x).natAbs.toInt
  let dy := -(y2i - y).natAbs.toInt
  let sx : Int := if x < x2i then 1 else -1
  let sy : Int := if y < y2i then 1 else -1
  let mut err := dx + dy
  let mut done := false
  while !done do
    if x ≥ 0 ∧ y ≥ 0 then
      imgM := setPixel imgM x.toNat y.toNat color
    if x == x2i ∧ y == y2i then
      done := true
    else
      let e2 := 2 * err
      if e2 ≥ dy then
        err := err + dy
        x := x + sx
      if e2 ≤ dx then
        err := err + dx
        y := y + sy
  imgM

def applyOne (img : Image) (a : Annotation) : Image :=
  match a with
  | .rectangle x y w h color stroke => drawRectOutline img x y w h color stroke
  | .highlight x y w h color opacity => blendRect img x y w h color opacity
  | .text x y _ color => setPixel img x y color
  | .arrow x1 y1 x2 y2 color => drawLine img x1 y1 x2 y2 color

def applyAnnotations (base : Image) (anns : List Annotation) : Image :=
  anns.foldl (init := base) applyOne

def Recorder.capture (r : Recorder) (screen : Image)
    (windowLookup : Nat → Option Image) : Recorder × Option Capture :=
  if r.state != .capturing then (r, none)
  else match r.pendingSpec with
  | none => (r, none)
  | some spec =>
    let id := r.pendingId
    let baseOpt : Option Image :=
      match spec.region with
      | .fullScreen => some { width := screen.width, height := screen.height,
                                pixels := screen.pixels }
      | .rect x y w h =>
        if x + w > screen.width ∨ y + h > screen.height then none
        else some (cropImg screen x y w h)
      | .window id2 => windowLookup id2
    match baseOpt with
    | none =>
      let r' := { r with
        state := .idle,
        pendingSpec := none,
        events := r.events ++ [{ kind := .captureFailed, id, detail := "region invalid" }]
      }
      (r', none)
    | some base =>
      let annotated := applyAnnotations base spec.annotations
      let cap : Capture := { id, spec, image := annotated, capturedAtMs := r.elapsedMs }
      let history' := r.history ++ [cap]
      let (history'', evictEvents) :=
        if history'.length > r.historyMax then
          match history' with
          | [] => (history', [])
          | evicted :: rest => (rest, [({ kind := .evicted, id := evicted.id } : RecorderEvent)])
        else (history', [])
      let r' := { r with
        state := .captured,
        pendingSpec := none,
        history := history'',
        events := r.events ++ evictEvents ++ [{
          kind := .captured, id, detail := s!"at={r.elapsedMs}ms"
        }]
      }
      (r', some cap)

def Recorder.reset (r : Recorder) : Recorder :=
  { r with state := .idle, pendingSpec := none, pendingId := 0,
           countdownRemainingMs := 0 }

end Graphics.Capture
