// File Explorer — navigable virtual filesystem with path resolution.
package files

import (
	"fmt"
	"sort"
	"strings"
)

// Kind tags a node.
type Kind int

const (
	KDir  Kind = 0
	KFile Kind = 1
)

// Node is a virtual filesystem node.
type Node struct {
	Kind     Kind
	Size     int64
	MtimeMs  int64
	Children map[string]*Node
}

// NewDir returns an empty directory node.
func NewDir() *Node { return &Node{Kind: KDir, Children: map[string]*Node{}} }

// NewFile returns a file node.
func NewFile(size, mtime int64) *Node {
	return &Node{Kind: KFile, Size: size, MtimeMs: mtime}
}

// Listing is one entry from `ls`.
type Listing struct {
	Name    string
	Kind    Kind
	Size    int64
	MtimeMs int64
}

// Explorer is a mutable virtual filesystem with a current working directory.
type Explorer struct {
	Root *Node
	cwd  []string
}

// NewExplorer returns a fresh explorer rooted at `/`.
func NewExplorer() *Explorer {
	return &Explorer{Root: NewDir(), cwd: []string{}}
}

// CwdString returns `/a/b/c`.
func (e *Explorer) CwdString() string { return "/" + strings.Join(e.cwd, "/") }

// Resolve canonicalises an absolute-or-relative path into segments.
// Empty path returns nil. Root is the empty slice.
func (e *Explorer) Resolve(path string) []string {
	if path == "" {
		return nil
	}
	var segs []string
	if !strings.HasPrefix(path, "/") {
		segs = append(segs, e.cwd...)
	}
	for _, seg := range strings.Split(path, "/") {
		if seg == "" || seg == "." {
			continue
		}
		if seg == ".." {
			if len(segs) > 0 {
				segs = segs[:len(segs)-1]
			}
			continue
		}
		segs = append(segs, seg)
	}
	if segs == nil {
		return []string{}  // explicit empty = root
	}
	return segs
}

func (e *Explorer) nodeAt(segs []string) *Node {
	cur := e.Root
	for _, s := range segs {
		if cur.Kind != KDir {
			return nil
		}
		next, ok := cur.Children[s]
		if !ok {
			return nil
		}
		cur = next
	}
	return cur
}

// Cd changes the current working directory.
func (e *Explorer) Cd(path string) error {
	segs := e.Resolve(path)
	if segs == nil {
		return fmt.Errorf("not found: %s", path)
	}
	node := e.nodeAt(segs)
	if node == nil {
		return fmt.Errorf("not found: %s", path)
	}
	if node.Kind != KDir {
		return fmt.Errorf("not a dir: %s", path)
	}
	e.cwd = segs
	return nil
}

// Ls lists the entries in a directory. Empty path = cwd.
func (e *Explorer) Ls(path string) ([]Listing, error) {
	var segs []string
	if path == "" {
		segs = append([]string{}, e.cwd...)
	} else {
		segs = e.Resolve(path)
		if segs == nil {
			return nil, fmt.Errorf("not found: %s", path)
		}
	}
	node := e.nodeAt(segs)
	if node == nil {
		return nil, fmt.Errorf("not found: %s", path)
	}
	if node.Kind != KDir {
		return nil, fmt.Errorf("not a dir: %s", path)
	}
	out := make([]Listing, 0, len(node.Children))
	for name, child := range node.Children {
		l := Listing{Name: name, Kind: child.Kind}
		if child.Kind == KFile {
			l.Size = child.Size
			l.MtimeMs = child.MtimeMs
		}
		out = append(out, l)
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Kind != out[j].Kind {
			return out[i].Kind == KDir
		}
		return out[i].Name < out[j].Name
	})
	return out, nil
}

// Mkdir creates a directory.
func (e *Explorer) Mkdir(path string) error {
	segs := e.Resolve(path)
	if segs == nil || len(segs) == 0 {
		return fmt.Errorf("already exists: %s", path)
	}
	parent := e.nodeAt(segs[:len(segs)-1])
	if parent == nil || parent.Kind != KDir {
		return fmt.Errorf("not found: %s", path)
	}
	name := segs[len(segs)-1]
	if _, exists := parent.Children[name]; exists {
		return fmt.Errorf("already exists: %s", path)
	}
	parent.Children[name] = NewDir()
	return nil
}

// Touch writes a file node (replacing any existing entry).
func (e *Explorer) Touch(path string, size, mtime int64) error {
	segs := e.Resolve(path)
	if segs == nil || len(segs) == 0 {
		return fmt.Errorf("already exists: %s", path)
	}
	parent := e.nodeAt(segs[:len(segs)-1])
	if parent == nil || parent.Kind != KDir {
		return fmt.Errorf("not found: %s", path)
	}
	parent.Children[segs[len(segs)-1]] = NewFile(size, mtime)
	return nil
}

// Stat returns a listing for one entry.
func (e *Explorer) Stat(path string) (Listing, error) {
	segs := e.Resolve(path)
	if segs == nil {
		return Listing{}, fmt.Errorf("not found: %s", path)
	}
	node := e.nodeAt(segs)
	if node == nil {
		return Listing{}, fmt.Errorf("not found: %s", path)
	}
	name := "/"
	if len(segs) > 0 {
		name = segs[len(segs)-1]
	}
	if node.Kind == KDir {
		return Listing{Name: name, Kind: KDir}, nil
	}
	return Listing{Name: name, Kind: KFile, Size: node.Size, MtimeMs: node.MtimeMs}, nil
}

// TotalSize walks the subtree rooted at path summing file sizes.
func (e *Explorer) TotalSize(path string) (int64, error) {
	segs := e.Resolve(path)
	if segs == nil {
		return 0, fmt.Errorf("not found: %s", path)
	}
	node := e.nodeAt(segs)
	if node == nil {
		return 0, fmt.Errorf("not found: %s", path)
	}
	return sizeOf(node), nil
}

func sizeOf(n *Node) int64 {
	if n.Kind == KFile {
		return n.Size
	}
	var sum int64
	for _, c := range n.Children {
		sum += sizeOf(c)
	}
	return sum
}
