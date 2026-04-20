// Mind Mapper — radial layout + SVG emission.
package graphics

import (
	"fmt"
	"math"
	"strings"
)

// MMNode is one mind-map node with nested children.
type MMNode struct {
	ID        int64
	Label     string
	Color     string
	Collapsed bool
	Children  []*MMNode
}

// NewMMNode returns a default-coloured node.
func NewMMNode(id int64, label string) *MMNode {
	return &MMNode{ID: id, Label: label, Color: "#3b82f6"}
}

// WithChild appends a child (fluent).
func (n *MMNode) WithChild(c *MMNode) *MMNode {
	n.Children = append(n.Children, c)
	return n
}

// WithColor sets the fill colour.
func (n *MMNode) WithColor(color string) *MMNode {
	n.Color = color
	return n
}

// Find returns the node with the given ID in the subtree, or nil.
func (n *MMNode) Find(id int64) *MMNode {
	if n.ID == id { return n }
	for _, c := range n.Children {
		if found := c.Find(id); found != nil { return found }
	}
	return nil
}

// ToggleCollapse flips collapsed state; returns new state, or false/false if missing.
func (n *MMNode) ToggleCollapse(id int64) (bool, bool) {
	target := n.Find(id)
	if target == nil { return false, false }
	target.Collapsed = !target.Collapsed
	return target.Collapsed, true
}

// VisibleCount counts visible nodes only.
func (n *MMNode) VisibleCount() int {
	if n.Collapsed { return 1 }
	total := 1
	for _, c := range n.Children { total += c.VisibleCount() }
	return total
}

// TotalCount counts all descendants regardless of collapse.
func (n *MMNode) TotalCount() int {
	total := 1
	for _, c := range n.Children { total += c.TotalCount() }
	return total
}

// MaxDepth returns depth of the deepest visible leaf.
func (n *MMNode) MaxDepth() int {
	if len(n.Children) == 0 || n.Collapsed { return 0 }
	best := 0
	for _, c := range n.Children {
		d := c.MaxDepth()
		if d > best { best = d }
	}
	return 1 + best
}

// MMLaidOutNode is a node with computed coordinates.
type MMLaidOutNode struct {
	ID        int64
	Label     string
	Color     string
	Depth     int
	X, Y      float64
	ParentID  int64
	HasParent bool
	Collapsed bool
}

// RadialLayout assigns (x, y) coords via polar layout around (cx, cy).
func RadialLayout(root *MMNode, cx, cy, radiusStep float64) []MMLaidOutNode {
	var out []MMLaidOutNode
	out = append(out, MMLaidOutNode{
		ID: root.ID, Label: root.Label, Color: root.Color, Depth: 0,
		X: cx, Y: cy, HasParent: false, Collapsed: root.Collapsed,
	})
	if !root.Collapsed {
		layoutChildren(root, 0.0, 2*math.Pi, 1, radiusStep, cx, cy, &out)
	}
	return out
}

func layoutChildren(parent *MMNode, arcStart, arcEnd float64,
	depth int, radiusStep, cx, cy float64, out *[]MMLaidOutNode) {
	n := len(parent.Children)
	if n == 0 { return }
	slice := (arcEnd - arcStart) / float64(n)
	for i, child := range parent.Children {
		a0 := arcStart + slice*float64(i)
		a1 := arcStart + slice*float64(i+1)
		angle := (a0 + a1) / 2
		r := float64(depth) * radiusStep
		x := cx + r*math.Cos(angle)
		y := cy + r*math.Sin(angle)
		*out = append(*out, MMLaidOutNode{
			ID: child.ID, Label: child.Label, Color: child.Color,
			Depth: depth, X: x, Y: y,
			ParentID: parent.ID, HasParent: true,
			Collapsed: child.Collapsed,
		})
		if !child.Collapsed {
			layoutChildren(child, a0, a1, depth+1, radiusStep, cx, cy, out)
		}
	}
}

// MMTraverseDFS returns pre-order DFS of visible node IDs.
func MMTraverseDFS(root *MMNode) []int64 {
	var out []int64
	mmDfs(root, &out)
	return out
}

func mmDfs(node *MMNode, out *[]int64) {
	*out = append(*out, node.ID)
	if node.Collapsed { return }
	for _, c := range node.Children { mmDfs(c, out) }
}

// MMTraverseBFS returns level-order BFS of visible node IDs.
func MMTraverseBFS(root *MMNode) []int64 {
	var out []int64
	queue := []*MMNode{root}
	for len(queue) > 0 {
		n := queue[0]
		queue = queue[1:]
		out = append(out, n.ID)
		if !n.Collapsed {
			queue = append(queue, n.Children...)
		}
	}
	return out
}

// MMToSVG emits an SVG string for the laid-out nodes.
func MMToSVG(nodes []MMLaidOutNode, width, height, radius float64) string {
	var b strings.Builder
	fmt.Fprintf(&b, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"%v\" height=\"%v\" viewBox=\"0 0 %v %v\">\n",
		width, height, width, height)
	byID := map[int64]MMLaidOutNode{}
	for _, n := range nodes { byID[n.ID] = n }
	for _, n := range nodes {
		if n.HasParent {
			p, ok := byID[n.ParentID]
			if ok {
				fmt.Fprintf(&b,
					"  <line x1=\"%.2f\" y1=\"%.2f\" x2=\"%.2f\" y2=\"%.2f\" stroke=\"#cbd5e1\" stroke-width=\"2\"/>\n",
					p.X, p.Y, n.X, n.Y)
			}
		}
	}
	for _, n := range nodes {
		fmt.Fprintf(&b, "  <circle cx=\"%.2f\" cy=\"%.2f\" r=\"%v\" fill=\"%s\"/>\n",
			n.X, n.Y, radius, n.Color)
		fmt.Fprintf(&b,
			"  <text x=\"%.2f\" y=\"%.2f\" text-anchor=\"middle\" dy=\"0.35em\" fill=\"white\" font-size=\"12\">%s</text>\n",
			n.X, n.Y, mmEscapeXML(n.Label))
	}
	b.WriteString("</svg>\n")
	return b.String()
}

func mmEscapeXML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, "\"", "&quot;")
	return s
}
