// ERD Creator — entity-relationship diagram with cardinality + topological sort.
package databases

import (
	"fmt"
	"sort"
	"strings"
)

// ERCardinality describes a relationship's multiplicity.
type ERCardinality int

const (
	ERCOneToOne    ERCardinality = 0
	ERCOneToMany   ERCardinality = 1
	ERCManyToMany  ERCardinality = 2
)

// DotArrow returns the DOT arrowhead keyword for this cardinality.
func (c ERCardinality) DotArrow() string {
	switch c {
	case ERCOneToOne: return "none"
	case ERCOneToMany: return "crow"
	case ERCManyToMany: return "crowodot"
	}
	return "none"
}

// ERAttribute is one column of an entity.
type ERAttribute struct {
	Name       string
	Ty         string
	PrimaryKey bool
	Nullable   bool
}

// EREntity is a table with attributes.
type EREntity struct {
	Name       string
	Attributes []ERAttribute
}

// PrimaryKeys returns names of PK attributes.
func (e *EREntity) PrimaryKeys() []string {
	var out []string
	for _, a := range e.Attributes {
		if a.PrimaryKey { out = append(out, a.Name) }
	}
	return out
}

// ERRelationship is a foreign-key edge.
type ERRelationship struct {
	FromEntity, FromAttr string
	ToEntity, ToAttr     string
	Cardinality          ERCardinality
}

// ERD is the entity-relationship diagram.
type ERD struct {
	Entities      map[string]*EREntity
	Relationships []ERRelationship
}

// NewERD returns an empty diagram.
func NewERD() *ERD { return &ERD{Entities: map[string]*EREntity{}} }

// AddEREntity inserts an entity.
func (e *ERD) AddEREntity(entity *EREntity) { e.Entities[entity.Name] = entity }

// AddERRelationship appends a relationship.
func (e *ERD) AddERRelationship(r ERRelationship) {
	e.Relationships = append(e.Relationships, r)
}

// DependenciesOf lists entities referenced via FK.
func (e *ERD) DependenciesOf(name string) []string {
	deps := map[string]bool{}
	for _, r := range e.Relationships {
		if r.FromEntity == name && r.ToEntity != name {
			deps[r.ToEntity] = true
		}
	}
	out := make([]string, 0, len(deps))
	for k := range deps { out = append(out, k) }
	sort.Strings(out)
	return out
}

// TopoSort returns entities in dependency order, or nil on cycle.
func (e *ERD) TopoSort() []string {
	inDegree := map[string]int{}
	for name := range e.Entities {
		inDegree[name] = 0
	}
	for _, r := range e.Relationships {
		if r.FromEntity != r.ToEntity &&
			e.Entities[r.FromEntity] != nil && e.Entities[r.ToEntity] != nil {
			inDegree[r.FromEntity]++
		}
	}
	var queue []string
	for n, d := range inDegree {
		if d == 0 { queue = append(queue, n) }
	}
	sort.Strings(queue)
	var out []string
	for len(queue) > 0 {
		n := queue[0]
		queue = queue[1:]
		out = append(out, n)
		for _, r := range e.Relationships {
			if r.ToEntity == n && r.FromEntity != r.ToEntity {
				if _, ok := inDegree[r.FromEntity]; ok {
					if inDegree[r.FromEntity] > 0 {
						inDegree[r.FromEntity]--
					}
					if inDegree[r.FromEntity] == 0 &&
						!containsStr(out, r.FromEntity) &&
						!containsStr(queue, r.FromEntity) {
						queue = append(queue, r.FromEntity)
						sort.Strings(queue)
					}
				}
			}
		}
	}
	if len(out) != len(e.Entities) { return nil }
	return out
}

// FindCycles returns lists of entity names forming cycles.
func (e *ERD) FindCycles() [][]string {
	adj := map[string][]string{}
	for name := range e.Entities {
		adj[name] = nil
	}
	for _, r := range e.Relationships {
		if r.FromEntity != r.ToEntity &&
			e.Entities[r.FromEntity] != nil && e.Entities[r.ToEntity] != nil {
			adj[r.FromEntity] = append(adj[r.FromEntity], r.ToEntity)
		}
	}
	var cycles [][]string
	visited := map[string]bool{}

	var dfs func(node string, path []string, onPath map[string]bool)
	dfs = func(node string, path []string, onPath map[string]bool) {
		for _, child := range adj[node] {
			if onPath[child] {
				start := 0
				for i, n := range path {
					if n == child { start = i; break }
				}
				cycle := append([]string{}, path[start:]...)
				cycle = append(cycle, child)
				cycles = append(cycles, cycle)
			} else if !visited[child] {
				newPath := append([]string{}, path...)
				newPath = append(newPath, child)
				newOnPath := map[string]bool{}
				for k, v := range onPath { newOnPath[k] = v }
				newOnPath[child] = true
				dfs(child, newPath, newOnPath)
			}
		}
		visited[node] = true
	}
	var names []string
	for n := range e.Entities { names = append(names, n) }
	sort.Strings(names)
	for _, start := range names {
		if visited[start] { continue }
		dfs(start, []string{start}, map[string]bool{start: true})
	}
	return cycles
}

// ValidateERD returns a list of issues.
func (e *ERD) ValidateERD() []string {
	var issues []string
	for i, r := range e.Relationships {
		from := e.Entities[r.FromEntity]
		to := e.Entities[r.ToEntity]
		if from == nil {
			issues = append(issues, fmt.Sprintf("relationship %d: unknown from_entity %s", i, r.FromEntity))
		} else {
			found := false
			for _, a := range from.Attributes { if a.Name == r.FromAttr { found = true; break } }
			if !found {
				issues = append(issues, fmt.Sprintf("relationship %d: %s.%s — attribute missing", i, r.FromEntity, r.FromAttr))
			}
		}
		if to == nil {
			issues = append(issues, fmt.Sprintf("relationship %d: unknown to_entity %s", i, r.ToEntity))
		} else {
			found := false
			for _, a := range to.Attributes { if a.Name == r.ToAttr { found = true; break } }
			if !found {
				issues = append(issues, fmt.Sprintf("relationship %d: %s.%s — attribute missing", i, r.ToEntity, r.ToAttr))
			}
		}
	}
	return issues
}

// ToDot emits Graphviz DOT text.
func (e *ERD) ToDot() string {
	var b strings.Builder
	b.WriteString("digraph ERD {\n  node [shape=record];\n")
	var names []string
	for n := range e.Entities { names = append(names, n) }
	sort.Strings(names)
	for _, name := range names {
		ent := e.Entities[name]
		attrs := make([]string, len(ent.Attributes))
		for i, a := range ent.Attributes {
			marker := ""
			if a.PrimaryKey { marker = "🔑 " }
			attrs[i] = fmt.Sprintf("%s%s: %s", marker, a.Name, a.Ty)
		}
		fmt.Fprintf(&b, "  %s [label=\"{%s|%s\\l}\"];\n", name, name, strings.Join(attrs, `\l`))
	}
	for _, r := range e.Relationships {
		fmt.Fprintf(&b, "  %s -> %s [label=\"%s.%s -> %s.%s\", arrowhead=\"%s\"];\n",
			r.FromEntity, r.ToEntity,
			r.FromEntity, r.FromAttr, r.ToEntity, r.ToAttr,
			r.Cardinality.DotArrow())
	}
	b.WriteString("}\n")
	return b.String()
}

func containsStr(list []string, s string) bool {
	for _, v := range list { if v == s { return true } }
	return false
}
