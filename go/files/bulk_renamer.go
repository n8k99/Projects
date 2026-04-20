// Bulk Renamer and Organizer — pattern-driven rename + preview + undo log.
package files

import (
	"fmt"
	"sort"
	"strings"
)

// RuleKind tags a rename rule.
type RuleKind int

const (
	RKReplace         RuleKind = 0
	RKPrefix          RuleKind = 1
	RKSuffixBeforeExt RuleKind = 2
	RKSequence        RuleKind = 3
)

// RenameRule specifies how to rewrite filenames.
type RenameRule struct {
	Kind     RuleKind
	Find     string
	Replace  string
	Prefix   string
	Suffix   string
	Template string
}

// RenameOp is one from→to transformation.
type RenameOp struct {
	From string
	To   string
}

// BrnDirectory is a set of filenames.
type BrnDirectory struct {
	Entries map[string]struct{}
}

// NewBrnDirectory creates a directory with the given names.
func NewBrnDirectory(names []string) *BrnDirectory {
	d := &BrnDirectory{Entries: map[string]struct{}{}}
	for _, n := range names {
		d.Entries[n] = struct{}{}
	}
	return d
}

// Names returns sorted names.
func (d *BrnDirectory) Names() []string {
	out := make([]string, 0, len(d.Entries))
	for n := range d.Entries {
		out = append(out, n)
	}
	sort.Strings(out)
	return out
}

// Preview computes rename ops without applying. Errors on collision/target-exists.
func (d *BrnDirectory) Preview(rule RenameRule) ([]RenameOp, error) {
	names := d.Names()
	var ops []RenameOp
	seenTargets := map[string]string{}
	for i, name := range names {
		newName := applyBrnRule(name, rule, i)
		if newName == name {
			continue
		}
		if prev, exists := seenTargets[newName]; exists {
			return nil, fmt.Errorf("collision: %s and %s both → %s", prev, name, newName)
		}
		seenTargets[newName] = name
		ops = append(ops, RenameOp{From: name, To: newName})
	}
	renameSources := map[string]bool{}
	for _, op := range ops {
		renameSources[op.From] = true
	}
	for _, op := range ops {
		if _, exists := d.Entries[op.To]; exists && !renameSources[op.To] {
			return nil, fmt.Errorf("target exists: %s", op.To)
		}
	}
	return ops, nil
}

// Apply runs ops atomically. Returns the undo log.
func (d *BrnDirectory) Apply(ops []RenameOp) ([]RenameOp, error) {
	for _, op := range ops {
		if _, ok := d.Entries[op.From]; !ok {
			return nil, fmt.Errorf("from missing: %s", op.From)
		}
	}
	for _, op := range ops {
		delete(d.Entries, op.From)
	}
	for _, op := range ops {
		d.Entries[op.To] = struct{}{}
	}
	undo := make([]RenameOp, len(ops))
	for i, op := range ops {
		undo[i] = RenameOp{From: op.To, To: op.From}
	}
	return undo, nil
}

// Undo applies a previous undo log.
func (d *BrnDirectory) Undo(log []RenameOp) error {
	_, err := d.Apply(log)
	return err
}

func applyBrnRule(name string, rule RenameRule, index int) string {
	switch rule.Kind {
	case RKReplace:
		pos := strings.Index(name, rule.Find)
		if pos < 0 {
			return name
		}
		return name[:pos] + rule.Replace + name[pos+len(rule.Find):]
	case RKPrefix:
		return rule.Prefix + name
	case RKSuffixBeforeExt:
		dot := strings.LastIndex(name, ".")
		if dot < 0 {
			return name + rule.Suffix
		}
		return name[:dot] + rule.Suffix + name[dot:]
	case RKSequence:
		return strings.ReplaceAll(rule.Template, "{n}", fmt.Sprintf("%d", index+1))
	}
	return name
}
