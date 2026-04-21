-- Turtle Graphics — the Rosetta Stone finale.

import Graphics.Grayscale

namespace Graphics.TurtleG

open Graphics.Gray

def sin0To90 : Array Int := #[
  0, 175, 349, 523, 698, 872, 1045, 1219, 1392, 1564,
  1736, 1908, 2079, 2250, 2419, 2588, 2756, 2924, 3090, 3256,
  3420, 3584, 3746, 3907, 4067, 4226, 4384, 4540, 4695, 4848,
  5000, 5150, 5299, 5446, 5592, 5736, 5878, 6018, 6157, 6293,
  6428, 6561, 6691, 6820, 6947, 7071, 7193, 7314, 7431, 7547,
  7660, 7771, 7880, 7986, 8090, 8192, 8290, 8387, 8480, 8572,
  8660, 8746, 8829, 8910, 8988, 9063, 9135, 9205, 9272, 9336,
  9397, 9455, 9511, 9563, 9613, 9659, 9703, 9744, 9781, 9816,
  9848, 9877, 9903, 9925, 9945, 9962, 9976, 9986, 9994, 9998,
  10000
]

def sinTable (angle : Int) : Int :=
  let a := ((angle % 360) + 360) % 360
  if a ≤ 90 then sin0To90.get! a.toNat
  else if a ≤ 180 then sin0To90.get! (180 - a).toNat
  else if a ≤ 270 then -sin0To90.get! (a - 180).toNat
  else -sin0To90.get! (360 - a).toNat

def cosTable (angle : Int) : Int :=
  sinTable ((angle + 90) % 360)

structure Turtle where
  x : Int := 0
  y : Int := 0
  heading : Int := 0
  penDown : Bool := true
  penColor : Rgb := { r := 0, g := 0, b := 0 }
  penWidth : Nat := 1
  deriving Repr

structure TLine where
  x1 : Int
  y1 : Int
  x2 : Int
  y2 : Int
  color : Rgb
  width : Nat
  deriving Repr

inductive Command where
  | forward (n : Int)
  | backward (n : Int)
  | turn (deg : Int)
  | setHeading (h : Int)
  | penUp
  | penDown
  | setColor (c : Rgb)
  | setPenWidth (w : Nat)
  | pushState
  | popState
  | goto (x y : Int)
  deriving Repr

structure Canvas where
  lines : List TLine := []
  turtle : Turtle := {}
  stateStack : List Turtle := []
  deriving Repr

def Canvas.new : Canvas := {}

def moveBy (c : Canvas) (distance : Int) : Canvas :=
  let cos := cosTable c.turtle.heading
  let sin := sinTable c.turtle.heading
  let newX := c.turtle.x + distance * cos / 10000
  let newY := c.turtle.y + distance * sin / 10000
  let c1 := if c.turtle.penDown then
    { c with lines := c.lines ++ [{
        x1 := c.turtle.x, y1 := c.turtle.y,
        x2 := newX, y2 := newY,
        color := c.turtle.penColor, width := c.turtle.penWidth
      }]}
  else c
  { c1 with turtle := { c1.turtle with x := newX, y := newY } }

def Canvas.step (c : Canvas) (cmd : Command) : Canvas :=
  match cmd with
  | .forward n => moveBy c n
  | .backward n => moveBy c (-n)
  | .turn deg =>
    { c with turtle := { c.turtle with
               heading := ((c.turtle.heading + deg) % 360 + 360) % 360 } }
  | .setHeading h =>
    { c with turtle := { c.turtle with heading := ((h % 360) + 360) % 360 } }
  | .penUp => { c with turtle := { c.turtle with penDown := false } }
  | .penDown => { c with turtle := { c.turtle with penDown := true } }
  | .setColor col => { c with turtle := { c.turtle with penColor := col } }
  | .setPenWidth w => { c with turtle := { c.turtle with penWidth := w } }
  | .pushState => { c with stateStack := c.turtle :: c.stateStack }
  | .popState =>
    match c.stateStack with
    | [] => c
    | s :: rest => { c with turtle := s, stateStack := rest }
  | .goto x y =>
    let oldX := c.turtle.x
    let oldY := c.turtle.y
    let c1 := if c.turtle.penDown then
      { c with lines := c.lines ++ [{
          x1 := oldX, y1 := oldY, x2 := x, y2 := y,
          color := c.turtle.penColor, width := c.turtle.penWidth
        }]}
    else c
    { c1 with turtle := { c1.turtle with x := x, y := y } }

def Canvas.execute (c : Canvas) (program : List Command) : Canvas :=
  program.foldl (init := c) (·.step ·)

end Graphics.TurtleG
