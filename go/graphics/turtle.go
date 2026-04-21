// Turtle Graphics — the Rosetta Stone finale.
package graphics

import "fmt"

var turtleSin0to90 = [91]int32{
	0, 175, 349, 523, 698, 872, 1045, 1219, 1392, 1564,
	1736, 1908, 2079, 2250, 2419, 2588, 2756, 2924, 3090, 3256,
	3420, 3584, 3746, 3907, 4067, 4226, 4384, 4540, 4695, 4848,
	5000, 5150, 5299, 5446, 5592, 5736, 5878, 6018, 6157, 6293,
	6428, 6561, 6691, 6820, 6947, 7071, 7193, 7314, 7431, 7547,
	7660, 7771, 7880, 7986, 8090, 8192, 8290, 8387, 8480, 8572,
	8660, 8746, 8829, 8910, 8988, 9063, 9135, 9205, 9272, 9336,
	9397, 9455, 9511, 9563, 9613, 9659, 9703, 9744, 9781, 9816,
	9848, 9877, 9903, 9925, 9945, 9962, 9976, 9986, 9994, 9998,
	10000,
}

func SinTable(angle int32) int32 {
	a := ((angle % 360) + 360) % 360
	switch {
	case a <= 90: return turtleSin0to90[a]
	case a <= 180: return turtleSin0to90[180-a]
	case a <= 270: return -turtleSin0to90[a-180]
	default: return -turtleSin0to90[360-a]
	}
}

func CosTable(angle int32) int32 {
	return SinTable((angle + 90) % 360)
}

type Turtle struct {
	X, Y     int32
	Heading  int32 // 0..=359
	PenDown  bool
	PenColor GSRgb
	PenWidth uint32
}

type TLine struct {
	X1, Y1, X2, Y2 int32
	Color          GSRgb
	Width          uint32
}

type TCommandKind int

const (
	TCmdForward TCommandKind = iota
	TCmdBackward
	TCmdTurn
	TCmdSetHeading
	TCmdPenUp
	TCmdPenDown
	TCmdSetColor
	TCmdSetPenWidth
	TCmdPushState
	TCmdPopState
	TCmdGoto
)

type TCommand struct {
	Kind    TCommandKind
	N       int32
	Deg     int32
	Heading int32
	Color   GSRgb
	Width   uint32
	X, Y    int32
}

func TCmdForward(n int32) TCommand { return TCommand{Kind: TCmdForward, N: n} }
func TCmdBackward(n int32) TCommand { return TCommand{Kind: TCmdBackward, N: n} }
func TCmdTurn(deg int32) TCommand { return TCommand{Kind: TCmdTurn, Deg: deg} }
func TCmdSetHeading(h int32) TCommand { return TCommand{Kind: TCmdSetHeading, Heading: h} }
func TCmdPenUp() TCommand { return TCommand{Kind: TCmdPenUp} }
func TCmdPenDown() TCommand { return TCommand{Kind: TCmdPenDown} }
func TCmdSetColor(c GSRgb) TCommand { return TCommand{Kind: TCmdSetColor, Color: c} }
func TCmdSetPenWidth(w uint32) TCommand { return TCommand{Kind: TCmdSetPenWidth, Width: w} }
func TCmdPushState() TCommand { return TCommand{Kind: TCmdPushState} }
func TCmdPopState() TCommand { return TCommand{Kind: TCmdPopState} }
func TCmdGoto(x, y int32) TCommand { return TCommand{Kind: TCmdGoto, X: x, Y: y} }

type TCanvas struct {
	Lines      []TLine
	Turtle     Turtle
	StateStack []Turtle
}

func NewTCanvas() *TCanvas {
	return &TCanvas{
		Turtle: Turtle{PenDown: true, PenColor: GSRgb{0, 0, 0}, PenWidth: 1},
	}
}

func (c *TCanvas) moveBy(distance int32) {
	cos := CosTable(c.Turtle.Heading)
	sin := SinTable(c.Turtle.Heading)
	newX := c.Turtle.X + int32(int64(distance)*int64(cos)/10000)
	newY := c.Turtle.Y + int32(int64(distance)*int64(sin)/10000)
	if c.Turtle.PenDown {
		c.Lines = append(c.Lines, TLine{
			X1: c.Turtle.X, Y1: c.Turtle.Y, X2: newX, Y2: newY,
			Color: c.Turtle.PenColor, Width: c.Turtle.PenWidth,
		})
	}
	c.Turtle.X = newX
	c.Turtle.Y = newY
}

func (c *TCanvas) Step(cmd TCommand) {
	switch cmd.Kind {
	case TCmdForward:
		c.moveBy(cmd.N)
	case TCmdBackward:
		c.moveBy(-cmd.N)
	case TCmdTurn:
		c.Turtle.Heading = ((c.Turtle.Heading+cmd.Deg)%360 + 360) % 360
	case TCmdSetHeading:
		c.Turtle.Heading = ((cmd.Heading % 360) + 360) % 360
	case TCmdPenUp:
		c.Turtle.PenDown = false
	case TCmdPenDown:
		c.Turtle.PenDown = true
	case TCmdSetColor:
		c.Turtle.PenColor = cmd.Color
	case TCmdSetPenWidth:
		c.Turtle.PenWidth = cmd.Width
	case TCmdPushState:
		c.StateStack = append(c.StateStack, c.Turtle)
	case TCmdPopState:
		if len(c.StateStack) > 0 {
			c.Turtle = c.StateStack[len(c.StateStack)-1]
			c.StateStack = c.StateStack[:len(c.StateStack)-1]
		}
	case TCmdGoto:
		oldX := c.Turtle.X
		oldY := c.Turtle.Y
		c.Turtle.X = cmd.X
		c.Turtle.Y = cmd.Y
		if c.Turtle.PenDown {
			c.Lines = append(c.Lines, TLine{
				X1: oldX, Y1: oldY, X2: cmd.X, Y2: cmd.Y,
				Color: c.Turtle.PenColor, Width: c.Turtle.PenWidth,
			})
		}
	}
}

func (c *TCanvas) Execute(program []TCommand) {
	for _, cmd := range program {
		c.Step(cmd)
	}
}

func TurtleDemo() {
	fmt.Printf("sin(0)=%d cos(0)=%d\n", SinTable(0), CosTable(0))
	fmt.Printf("sin(90)=%d cos(90)=%d\n", SinTable(90), CosTable(90))
	fmt.Printf("sin(180)=%d cos(180)=%d\n", SinTable(180), CosTable(180))
	fmt.Printf("sin(270)=%d cos(270)=%d\n", SinTable(270), CosTable(270))
	fmt.Printf("sin(45)=%d cos(45)=%d\n", SinTable(45), CosTable(45))

	c := NewTCanvas()
	square := []TCommand{
		TCmdForward(100), TCmdTurn(90),
		TCmdForward(100), TCmdTurn(90),
		TCmdForward(100), TCmdTurn(90),
		TCmdForward(100), TCmdTurn(90),
	}
	c.Execute(square)
	fmt.Printf("square: turtle=(%d, %d) heading=%d lines=%d\n",
		c.Turtle.X, c.Turtle.Y, c.Turtle.Heading, len(c.Lines))

	c2 := NewTCanvas()
	c2.Execute([]TCommand{
		TCmdForward(50), TCmdPushState(),
		TCmdTurn(90), TCmdForward(30),
		TCmdPopState(),
	})
	fmt.Printf("after push/pop: turtle=(%d, %d) heading=%d\n",
		c2.Turtle.X, c2.Turtle.Y, c2.Turtle.Heading)
	fmt.Printf("lines drawn: %d\n", len(c2.Lines))

	c3 := NewTCanvas()
	rotatedSquare := []TCommand{TCmdTurn(45)}
	for i := 0; i < 4; i++ {
		rotatedSquare = append(rotatedSquare, TCmdForward(100), TCmdTurn(90))
	}
	c3.Execute(rotatedSquare)
	fmt.Printf("45-degree square: turtle=(%d, %d)\n", c3.Turtle.X, c3.Turtle.Y)

	c4 := NewTCanvas()
	for i := 0; i < 5; i++ {
		c4.Execute([]TCommand{TCmdForward(100), TCmdTurn(144)})
	}
	fmt.Printf("five-pointed star: lines=%d turtle=(%d, %d)\n",
		len(c4.Lines), c4.Turtle.X, c4.Turtle.Y)
}
