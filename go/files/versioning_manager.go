// Versioning Manager — content-addressed version store, minimal Git-like.
package files

import (
	"fmt"
	"sort"
	"strings"
)

// VMFnv1aHash computes FNV-1a 64-bit. Cross-language deterministic.
func VMFnv1aHash(data []byte) uint64 {
	h := uint64(0xcbf29ce484222325)
	for _, b := range data {
		h ^= uint64(b)
		h *= 0x100000001b3
	}
	return h
}

// VMBlob is a content-addressed byte sequence.
type VMBlob struct {
	Data []byte
}

// VMTree is a snapshot map filename → blob hash.
type VMTree struct {
	Entries map[string]uint64
}

// VMCommit is a tree + parent + message + timestamp.
type VMCommit struct {
	TreeHash    uint64
	Parent      *uint64
	Message     string
	TimestampMs int64
}

// VMDiffKind tags an operation in a diff.
type VMDiffKind int

const (
	VMAdded   VMDiffKind = 0
	VMRemoved VMDiffKind = 1
	VMChanged VMDiffKind = 2
)

// VMDiffOp describes one change between commits.
type VMDiffOp struct {
	Kind     VMDiffKind
	Name     string
	FromHash uint64
	ToHash   uint64
}

// VMRepo is a content-addressed version store.
type VMRepo struct {
	Blobs   map[uint64]VMBlob
	Trees   map[uint64]VMTree
	Commits map[uint64]VMCommit
	Head    *uint64
}

// NewVMRepo returns an empty repo.
func NewVMRepo() *VMRepo {
	return &VMRepo{
		Blobs:   map[uint64]VMBlob{},
		Trees:   map[uint64]VMTree{},
		Commits: map[uint64]VMCommit{},
	}
}

// PutBlob stores bytes by hash.
func (r *VMRepo) PutBlob(data []byte) uint64 {
	h := VMFnv1aHash(data)
	if _, exists := r.Blobs[h]; !exists {
		cp := make([]byte, len(data))
		copy(cp, data)
		r.Blobs[h] = VMBlob{Data: cp}
	}
	return h
}

// PutTree stores a tree by canonical content hash.
func (r *VMRepo) PutTree(entries map[string]uint64) uint64 {
	tree := VMTree{Entries: entries}
	canonical := vmCanonicalTreeBytes(tree)
	h := VMFnv1aHash(canonical)
	if _, exists := r.Trees[h]; !exists {
		r.Trees[h] = tree
	}
	return h
}

// Commit creates a commit from (filename, content) pairs.
func (r *VMRepo) Commit(files [][2]interface{}, message string, timestamp int64) uint64 {
	entries := map[string]uint64{}
	for _, pair := range files {
		name, _ := pair[0].(string)
		data, _ := pair[1].([]byte)
		entries[name] = r.PutBlob(data)
	}
	treeHash := r.PutTree(entries)
	c := VMCommit{TreeHash: treeHash, Parent: r.Head, Message: message, TimestampMs: timestamp}
	canonical := vmCanonicalCommitBytes(c)
	h := VMFnv1aHash(canonical)
	r.Commits[h] = c
	r.Head = &h
	return h
}

// Checkout returns files at a commit.
func (r *VMRepo) Checkout(commitHash uint64) map[string][]byte {
	c, ok := r.Commits[commitHash]
	if !ok {
		return nil
	}
	tree, ok := r.Trees[c.TreeHash]
	if !ok {
		return nil
	}
	out := map[string][]byte{}
	for name, bh := range tree.Entries {
		blob, ok := r.Blobs[bh]
		if !ok {
			return nil
		}
		cp := make([]byte, len(blob.Data))
		copy(cp, blob.Data)
		out[name] = cp
	}
	return out
}

// Diff between two commits.
func (r *VMRepo) Diff(from, to uint64) []VMDiffOp {
	fromC, ok := r.Commits[from]
	if !ok {
		return nil
	}
	toC, ok := r.Commits[to]
	if !ok {
		return nil
	}
	fromTree := r.Trees[fromC.TreeHash]
	toTree := r.Trees[toC.TreeHash]

	var ops []VMDiffOp
	for name, th := range toTree.Entries {
		if fh, exists := fromTree.Entries[name]; !exists {
			ops = append(ops, VMDiffOp{Kind: VMAdded, Name: name, ToHash: th})
		} else if fh != th {
			ops = append(ops, VMDiffOp{Kind: VMChanged, Name: name, FromHash: fh, ToHash: th})
		}
	}
	for name, fh := range fromTree.Entries {
		if _, exists := toTree.Entries[name]; !exists {
			ops = append(ops, VMDiffOp{Kind: VMRemoved, Name: name, FromHash: fh})
		}
	}

	sort.SliceStable(ops, func(i, j int) bool {
		return vmOpKey(ops[i]) < vmOpKey(ops[j])
	})
	return ops
}

// Log returns commits from HEAD back through parents.
func (r *VMRepo) Log() [][2]interface{} {
	var out [][2]interface{}
	cursor := r.Head
	for cursor != nil {
		c, ok := r.Commits[*cursor]
		if !ok {
			break
		}
		out = append(out, [2]interface{}{*cursor, c})
		cursor = c.Parent
	}
	return out
}

func (r *VMRepo) BlobCount() int   { return len(r.Blobs) }
func (r *VMRepo) CommitCount() int { return len(r.Commits) }

func vmOpKey(op VMDiffOp) string {
	prefix := "?"
	switch op.Kind {
	case VMAdded:
		prefix = "A"
	case VMRemoved:
		prefix = "R"
	case VMChanged:
		prefix = "C"
	}
	return prefix + op.Name
}

func vmCanonicalTreeBytes(t VMTree) []byte {
	names := make([]string, 0, len(t.Entries))
	for n := range t.Entries {
		names = append(names, n)
	}
	sort.Strings(names)
	var b strings.Builder
	b.WriteString("tree\n")
	for _, n := range names {
		fmt.Fprintf(&b, "%s\t%016x\n", n, t.Entries[n])
	}
	return []byte(b.String())
}

func vmCanonicalCommitBytes(c VMCommit) []byte {
	var b strings.Builder
	b.WriteString("commit\n")
	fmt.Fprintf(&b, "tree %016x\n", c.TreeHash)
	if c.Parent != nil {
		fmt.Fprintf(&b, "parent %016x\n", *c.Parent)
	}
	fmt.Fprintf(&b, "ts %d\n", c.TimestampMs)
	fmt.Fprintf(&b, "msg %s\n", c.Message)
	return []byte(b.String())
}
