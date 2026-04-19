// E-Card Generator — template + data rendering with slot validation.
package web

import (
	"errors"
	"fmt"
	"strings"
)

// ECSlotKind classifies a template slot.
type ECSlotKind int

const (
	ECText ECSlotKind = iota
	ECLongText
	ECImageURL
	ECColor
	ECDate
)

// ECSlot declares one template slot.
type ECSlot struct {
	Name     string
	Kind     ECSlotKind
	Required bool
	Default  string // "" means no default
}

// ECTemplate is a card template.
type ECTemplate struct {
	ID           int64
	Name         string
	Category     string
	Slots        []ECSlot
	HTMLTemplate string
}

// ECCard is a filled template instance.
type ECCard struct {
	ID          int64
	TemplateID  int64
	Filled      map[string]string
	Recipient   string
	Sender      string
	CreatedAtMs int64
}

// Validation errors.
var (
	ErrECUnknownTemplate = errors.New("unknown template")
	ErrECMissingRequired = errors.New("missing required slot")
	ErrECUnknownSlot     = errors.New("unknown slot")
)

// ECGallery holds templates and cards.
type ECGallery struct {
	templates      []*ECTemplate
	cards          []*ECCard
	nextTemplateID int64
	nextCardID     int64
}

// NewECGallery returns an empty gallery.
func NewECGallery() *ECGallery {
	return &ECGallery{nextTemplateID: 1, nextCardID: 1}
}

// AddTemplate registers a new template and returns its id.
func (g *ECGallery) AddTemplate(name, category string,
	slots []ECSlot, htmlTemplate string) int64 {
	id := g.nextTemplateID
	g.nextTemplateID++
	g.templates = append(g.templates, &ECTemplate{
		ID: id, Name: name, Category: category,
		Slots: slots, HTMLTemplate: htmlTemplate,
	})
	return id
}

// Template returns a template by id (nil if unknown).
func (g *ECGallery) Template(id int64) *ECTemplate {
	for _, t := range g.templates {
		if t.ID == id {
			return t
		}
	}
	return nil
}

// Templates returns all registered templates.
func (g *ECGallery) Templates() []*ECTemplate { return g.templates }

// TemplatesInCategory filters templates by category.
func (g *ECGallery) TemplatesInCategory(category string) []*ECTemplate {
	var out []*ECTemplate
	for _, t := range g.templates {
		if t.Category == category {
			out = append(out, t)
		}
	}
	return out
}

// CreateCard validates slot data and creates a card.
func (g *ECGallery) CreateCard(templateID int64,
	filled map[string]string, recipient, sender string, nowMs int64,
) (int64, error) {
	template := g.Template(templateID)
	if template == nil {
		return 0, fmt.Errorf("%w: %d", ErrECUnknownTemplate, templateID)
	}
	known := map[string]bool{}
	for _, s := range template.Slots {
		known[s.Name] = true
	}
	for k := range filled {
		if !known[k] {
			return 0, fmt.Errorf("%w: %s", ErrECUnknownSlot, k)
		}
	}
	for _, s := range template.Slots {
		if s.Required {
			if _, ok := filled[s.Name]; !ok {
				return 0, fmt.Errorf("%w: %s", ErrECMissingRequired, s.Name)
			}
		}
	}
	final := make(map[string]string, len(filled))
	for k, v := range filled {
		final[k] = v
	}
	for _, s := range template.Slots {
		if _, ok := final[s.Name]; !ok && s.Default != "" {
			final[s.Name] = s.Default
		}
	}
	id := g.nextCardID
	g.nextCardID++
	g.cards = append(g.cards, &ECCard{
		ID: id, TemplateID: templateID, Filled: final,
		Recipient: recipient, Sender: sender, CreatedAtMs: nowMs,
	})
	return id, nil
}

// Card returns a card by id (nil if unknown).
func (g *ECGallery) Card(id int64) *ECCard {
	for _, c := range g.cards {
		if c.ID == id {
			return c
		}
	}
	return nil
}

// CardsByTemplate returns cards instantiated from the given template.
func (g *ECGallery) CardsByTemplate(templateID int64) []*ECCard {
	var out []*ECCard
	for _, c := range g.cards {
		if c.TemplateID == templateID {
			out = append(out, c)
		}
	}
	return out
}

// Render produces the card's HTML. "" if card or template missing.
func (g *ECGallery) Render(cardID int64) string {
	card := g.Card(cardID)
	if card == nil {
		return ""
	}
	template := g.Template(card.TemplateID)
	if template == nil {
		return ""
	}
	out := template.HTMLTemplate
	for k, v := range card.Filled {
		out = strings.ReplaceAll(out, "{{"+k+"}}", v)
	}
	out = strings.ReplaceAll(out, "{{_recipient}}", card.Recipient)
	out = strings.ReplaceAll(out, "{{_sender}}", card.Sender)
	return out
}

// RenderPlain strips tags after rendering.
func (g *ECGallery) RenderPlain(cardID int64) string {
	html := g.Render(cardID)
	if html == "" {
		return ""
	}
	var sb strings.Builder
	inTag := false
	for _, r := range html {
		switch r {
		case '<':
			inTag = true
		case '>':
			inTag = false
		default:
			if !inTag {
				sb.WriteRune(r)
			}
		}
	}
	return sb.String()
}
