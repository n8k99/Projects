// Family Tree Creator — the first self-referential graph with cycle prevention.
//
// A Person refers to other persons (as parents). The structure is a DAG, not a
// tree — "family tree" is a misnomer. A person has at most two parents, but
// joining two unrelated branches of a large family yields multiple paths to
// shared ancestors, so the code treats it as a graph throughout.
//
// SetParents runs a transitive closure check: a candidate parent must not
// already be a descendant of the child. First Classes project where an
// invariant is enforced across the entire reachable structure.
package classes

import (
	"errors"
	"fmt"
)

// PersonID identifies a person in a FamilyTree.
type PersonID int64

// Person represents a person in the tree.
type Person struct {
	ID      PersonID
	Name    string
	Birth   *int
	Death   *int
	Parents []PersonID
}

// FamilyTree is a collection of persons and their parent/child relationships.
type FamilyTree struct {
	persons  map[PersonID]*Person
	children map[PersonID]map[PersonID]struct{}
	nextID   PersonID
}

// Family errors.
var (
	ErrFamilyUnknownPerson  = errors.New("unknown person")
	ErrFamilyTooManyParents = errors.New("too many parents")
	ErrFamilyDuplicateParent = errors.New("duplicate parent")
	ErrFamilySelfParent      = errors.New("self-parent not allowed")
	ErrFamilyWouldCreateCycle = errors.New("would create cycle")
)

// NewFamilyTree returns an empty tree.
func NewFamilyTree() *FamilyTree {
	return &FamilyTree{
		persons:  make(map[PersonID]*Person),
		children: make(map[PersonID]map[PersonID]struct{}),
		nextID:   1,
	}
}

// AddPerson inserts a person and returns its generated ID.
func (t *FamilyTree) AddPerson(name string, birth, death *int) PersonID {
	id := t.nextID
	t.nextID++
	t.persons[id] = &Person{ID: id, Name: name, Birth: birth, Death: death}
	t.children[id] = make(map[PersonID]struct{})
	return id
}

// Get returns the Person by id (nil if unknown).
func (t *FamilyTree) Get(id PersonID) *Person { return t.persons[id] }

// SetParents replaces the parents of child with the given list (0, 1, or 2
// distinct known persons, none of which is the child or a descendant of it).
func (t *FamilyTree) SetParents(child PersonID, parents []PersonID) error {
	if _, ok := t.persons[child]; !ok {
		return fmt.Errorf("%w: %d", ErrFamilyUnknownPerson, child)
	}
	if len(parents) > 2 {
		return fmt.Errorf("%w: %d", ErrFamilyTooManyParents, len(parents))
	}
	seen := make(map[PersonID]struct{})
	for _, p := range parents {
		if _, dup := seen[p]; dup {
			return fmt.Errorf("%w: %d", ErrFamilyDuplicateParent, p)
		}
		seen[p] = struct{}{}
		if p == child {
			return ErrFamilySelfParent
		}
		if _, ok := t.persons[p]; !ok {
			return fmt.Errorf("%w: %d", ErrFamilyUnknownPerson, p)
		}
	}
	desc := t.Descendants(child, -1)
	for _, p := range parents {
		if _, isDesc := desc[p]; isDesc {
			return fmt.Errorf("%w: %d is a descendant of %d", ErrFamilyWouldCreateCycle, p, child)
		}
	}

	// Detach from old parents' children sets.
	for _, op := range t.persons[child].Parents {
		delete(t.children[op], child)
	}
	// Install new parent links.
	newParents := make([]PersonID, len(parents))
	copy(newParents, parents)
	t.persons[child].Parents = newParents
	for _, p := range parents {
		t.children[p][child] = struct{}{}
	}
	return nil
}

// Ancestors returns the set of ancestors reachable within maxDepth generations.
// Pass maxDepth < 0 for unbounded. Excludes `id`.
func (t *FamilyTree) Ancestors(id PersonID, maxDepth int) map[PersonID]struct{} {
	return t.bfs(id, maxDepth, func(x PersonID) []PersonID {
		p, ok := t.persons[x]
		if !ok {
			return nil
		}
		return p.Parents
	})
}

// Descendants returns the set of descendants reachable within maxDepth generations.
// Pass maxDepth < 0 for unbounded. Excludes `id`.
func (t *FamilyTree) Descendants(id PersonID, maxDepth int) map[PersonID]struct{} {
	return t.bfs(id, maxDepth, func(x PersonID) []PersonID {
		kids := t.children[x]
		out := make([]PersonID, 0, len(kids))
		for k := range kids {
			out = append(out, k)
		}
		return out
	})
}

// CommonAncestors returns the set of persons who are ancestors of BOTH a and b.
func (t *FamilyTree) CommonAncestors(a, b PersonID) map[PersonID]struct{} {
	aa := t.Ancestors(a, -1)
	bb := t.Ancestors(b, -1)
	out := make(map[PersonID]struct{})
	for x := range aa {
		if _, ok := bb[x]; ok {
			out[x] = struct{}{}
		}
	}
	return out
}

// Siblings returns the set of persons sharing at least one parent with id.
func (t *FamilyTree) Siblings(id PersonID) map[PersonID]struct{} {
	out := make(map[PersonID]struct{})
	p, ok := t.persons[id]
	if !ok {
		return out
	}
	for _, par := range p.Parents {
		for k := range t.children[par] {
			if k != id {
				out[k] = struct{}{}
			}
		}
	}
	return out
}

func (t *FamilyTree) bfs(start PersonID, maxDepth int,
	step func(PersonID) []PersonID) map[PersonID]struct{} {
	seen := make(map[PersonID]struct{})
	type node struct {
		id    PersonID
		depth int
	}
	queue := []node{{start, 0}}
	for len(queue) > 0 {
		cur := queue[0]
		queue = queue[1:]
		if maxDepth >= 0 && cur.depth >= maxDepth {
			continue
		}
		for _, next := range step(cur.id) {
			if _, ok := seen[next]; ok {
				continue
			}
			seen[next] = struct{}{}
			queue = append(queue, node{next, cur.depth + 1})
		}
	}
	return seen
}
