// Template Maker — meta-level tool for building and validating templates.
// Reuses ECSlot / ECSlotKind from ecard.go.
package web

import (
	"fmt"
	"regexp"
)

// TMBuilder holds a template work-in-progress.
type TMBuilder struct {
	ID         int64
	Name       string
	Category   string
	Slots      []ECSlot
	HTMLDraft  string
	SampleData map[string]string
}

// TMIssueKind classifies a validation issue.
type TMIssueKind int

const (
	TMUnusedSlot TMIssueKind = iota
	TMUndeclaredPlaceholder
	TMMissingSample
	TMDuplicateSlot
	TMInvalidSlotName
)

// TMIssue is one validation finding.
type TMIssue struct {
	Kind TMIssueKind
	Name string
}

// TemplateMaker owns the set of work-in-progress templates.
type TemplateMaker struct {
	builders []*TMBuilder
	nextID   int64
}

// NewTemplateMaker returns an empty template maker.
func NewTemplateMaker() *TemplateMaker { return &TemplateMaker{nextID: 1} }

// NewTemplate starts a new template builder.
func (m *TemplateMaker) NewTemplate(name, category string) int64 {
	id := m.nextID
	m.nextID++
	m.builders = append(m.builders, &TMBuilder{
		ID: id, Name: name, Category: category,
		SampleData: map[string]string{},
	})
	return id
}

// Builder returns a builder by id, or nil.
func (m *TemplateMaker) Builder(id int64) *TMBuilder {
	for _, b := range m.builders {
		if b.ID == id {
			return b
		}
	}
	return nil
}

func (m *TemplateMaker) require(id int64) (*TMBuilder, error) {
	b := m.Builder(id)
	if b == nil {
		return nil, fmt.Errorf("unknown builder: %d", id)
	}
	return b, nil
}

// AddSlot appends a slot, rejecting duplicates.
func (m *TemplateMaker) AddSlot(id int64, slot ECSlot) error {
	b, err := m.require(id)
	if err != nil {
		return err
	}
	for _, s := range b.Slots {
		if s.Name == slot.Name {
			return fmt.Errorf("slot already exists: %s", slot.Name)
		}
	}
	b.Slots = append(b.Slots, slot)
	return nil
}

// RemoveSlot removes a slot and any associated sample value.
func (m *TemplateMaker) RemoveSlot(id int64, slotName string) error {
	b, err := m.require(id)
	if err != nil {
		return err
	}
	for i, s := range b.Slots {
		if s.Name == slotName {
			b.Slots = append(b.Slots[:i], b.Slots[i+1:]...)
			delete(b.SampleData, slotName)
			return nil
		}
	}
	return fmt.Errorf("slot not found: %s", slotName)
}

// UpdateHTML sets the HTML draft.
func (m *TemplateMaker) UpdateHTML(id int64, html string) error {
	b, err := m.require(id)
	if err != nil {
		return err
	}
	b.HTMLDraft = html
	return nil
}

// SetSample sets the preview sample value for a slot.
func (m *TemplateMaker) SetSample(id int64, slotName, value string) error {
	b, err := m.require(id)
	if err != nil {
		return err
	}
	for _, s := range b.Slots {
		if s.Name == slotName {
			b.SampleData[slotName] = value
			return nil
		}
	}
	return fmt.Errorf("slot not found: %s", slotName)
}

var tmPlaceholderRe = regexp.MustCompile(`\{\{([^{}]+)\}\}`)
var tmSlotNameRe = regexp.MustCompile(`^[A-Za-z0-9_]+$`)

// Validate returns a list of validation issues.
func (m *TemplateMaker) Validate(id int64) []TMIssue {
	b := m.Builder(id)
	if b == nil {
		return nil
	}
	var issues []TMIssue
	seen := map[string]bool{}
	for _, slot := range b.Slots {
		if slot.Name == "" || !tmSlotNameRe.MatchString(slot.Name) {
			issues = append(issues, TMIssue{TMInvalidSlotName, slot.Name})
		}
		if seen[slot.Name] {
			issues = append(issues, TMIssue{TMDuplicateSlot, slot.Name})
		}
		seen[slot.Name] = true
	}
	declared := map[string]bool{}
	for _, s := range b.Slots {
		declared[s.Name] = true
	}
	referenced := map[string]bool{}
	for _, match := range tmPlaceholderRe.FindAllStringSubmatch(b.HTMLDraft, -1) {
		referenced[match[1]] = true
	}
	for _, slot := range b.Slots {
		if !referenced[slot.Name] {
			issues = append(issues, TMIssue{TMUnusedSlot, slot.Name})
		}
	}
	for ref := range referenced {
		if len(ref) > 0 && ref[0] == '_' {
			continue
		}
		if !declared[ref] {
			issues = append(issues, TMIssue{TMUndeclaredPlaceholder, ref})
		}
	}
	for _, slot := range b.Slots {
		if slot.Required {
			if _, ok := b.SampleData[slot.Name]; !ok {
				issues = append(issues, TMIssue{TMMissingSample, slot.Name})
			}
		}
	}
	return issues
}

// Preview renders the template with sample data / defaults.
func (m *TemplateMaker) Preview(id int64) string {
	b := m.Builder(id)
	if b == nil {
		return ""
	}
	out := b.HTMLDraft
	for _, slot := range b.Slots {
		var value string
		if s, ok := b.SampleData[slot.Name]; ok {
			value = s
		} else if slot.Default != "" {
			value = slot.Default
		} else {
			continue
		}
		out = replaceAll(out, "{{"+slot.Name+"}}", value)
	}
	out = replaceAll(out, "{{_recipient}}", "[preview recipient]")
	out = replaceAll(out, "{{_sender}}", "[preview sender]")
	return out
}

// Builders returns all active builders.
func (m *TemplateMaker) Builders() []*TMBuilder { return m.builders }

// RemoveBuilder removes a builder by id.
func (m *TemplateMaker) RemoveBuilder(id int64) bool {
	for i, b := range m.builders {
		if b.ID == id {
			m.builders = append(m.builders[:i], m.builders[i+1:]...)
			return true
		}
	}
	return false
}

func replaceAll(s, old, new string) string {
	result := ""
	for {
		i := indexOf(s, old)
		if i < 0 {
			return result + s
		}
		result += s[:i] + new
		s = s[i+len(old):]
	}
}

func indexOf(s, sub string) int {
	if len(sub) == 0 {
		return 0
	}
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return i
		}
	}
	return -1
}
