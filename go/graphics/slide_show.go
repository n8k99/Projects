// Slide Show — presentation navigation with dual-track content.
package graphics

import (
	"fmt"
	"strings"
)

// SSElementKind tags a slide element.
type SSElementKind int

const (
	SSEHeading SSElementKind = 0
	SSEBullet  SSElementKind = 1
	SSEImage   SSElementKind = 2
	SSESpacer  SSElementKind = 3
	SSECode    SSElementKind = 4
)

// SSElement is one piece of slide content.
type SSElement struct {
	Kind     SSElementKind
	Text     string
	Level    int
	Indent   int
	Src      string
	Alt      string
	Language string
	Lines    int
}

// SSTransition selects a between-slide animation.
type SSTransition int

const (
	SSTNone     SSTransition = 0
	SSTFade     SSTransition = 1
	SSTSlide    SSTransition = 2
	SSTDissolve SSTransition = 3
)

// SSSlide is one slide in the deck.
type SSSlide struct {
	Title      string
	Content    []SSElement
	Notes      string
	Transition SSTransition
}

// Heading appends a heading element (fluent).
func (s *SSSlide) Heading(text string, level int) *SSSlide {
	s.Content = append(s.Content, SSElement{Kind: SSEHeading, Text: text, Level: level})
	return s
}

// Bullet appends a bullet element.
func (s *SSSlide) Bullet(text string, indent int) *SSSlide {
	s.Content = append(s.Content, SSElement{Kind: SSEBullet, Text: text, Indent: indent})
	return s
}

// Image appends an image element.
func (s *SSSlide) Image(src, alt string) *SSSlide {
	s.Content = append(s.Content, SSElement{Kind: SSEImage, Src: src, Alt: alt})
	return s
}

// Code appends a code element.
func (s *SSSlide) Code(text, language string) *SSSlide {
	s.Content = append(s.Content, SSElement{Kind: SSECode, Text: text, Language: language})
	return s
}

// WithNotes sets speaker notes.
func (s *SSSlide) WithNotes(notes string) *SSSlide {
	s.Notes = notes
	return s
}

// SSMode selects rendering mode.
type SSMode int

const (
	SSMSpeaker  SSMode = 0
	SSMAudience SSMode = 1
	SSMHandout  SSMode = 2
)

// SSPresentation holds the slide deck.
type SSPresentation struct {
	Title        string
	Slides       []*SSSlide
	CurrentIndex int
}

// NewSSPresentation returns a presentation with the given title.
func NewSSPresentation(title string) *SSPresentation {
	return &SSPresentation{Title: title}
}

// AddSlide appends a slide.
func (p *SSPresentation) AddSlide(s *SSSlide) { p.Slides = append(p.Slides, s) }

// SlideCount reports the number of slides.
func (p *SSPresentation) SlideCount() int { return len(p.Slides) }

// Current returns the current slide, or nil.
func (p *SSPresentation) Current() *SSSlide {
	if p.CurrentIndex < 0 || p.CurrentIndex >= len(p.Slides) {
		return nil
	}
	return p.Slides[p.CurrentIndex]
}

// Advance moves to next slide, returns true on success.
func (p *SSPresentation) Advance() bool {
	if p.CurrentIndex+1 < len(p.Slides) {
		p.CurrentIndex++
		return true
	}
	return false
}

// Back moves to previous slide, returns true on success.
func (p *SSPresentation) Back() bool {
	if p.CurrentIndex > 0 {
		p.CurrentIndex--
		return true
	}
	return false
}

// Goto jumps to a specific index if valid.
func (p *SSPresentation) Goto(i int) bool {
	if i < 0 || i >= len(p.Slides) {
		return false
	}
	p.CurrentIndex = i
	return true
}

// Progress returns (current_1_indexed, total).
func (p *SSPresentation) Progress() (int, int) {
	if len(p.Slides) == 0 {
		return 0, 0
	}
	return p.CurrentIndex + 1, len(p.Slides)
}

// RenderCurrent renders the current slide for the given mode.
func (p *SSPresentation) RenderCurrent(mode SSMode) string {
	s := p.Current()
	if s == nil {
		return ""
	}
	return ssRenderSlide(s, mode)
}

// RenderHandout renders the entire deck.
func (p *SSPresentation) RenderHandout() string {
	var b strings.Builder
	fmt.Fprintf(&b, "# %s\n\n", p.Title)
	for i, s := range p.Slides {
		fmt.Fprintf(&b, "--- Slide %d ---\n", i+1)
		b.WriteString(ssRenderSlide(s, SSMHandout))
		b.WriteByte('\n')
	}
	return b.String()
}

// Search returns indices of slides whose title or content contains the needle.
func (p *SSPresentation) Search(needle string) []int {
	lc := strings.ToLower(needle)
	var out []int
	for i, s := range p.Slides {
		if strings.Contains(strings.ToLower(s.Title), lc) {
			out = append(out, i)
			continue
		}
		for _, e := range s.Content {
			if strings.Contains(strings.ToLower(ssElementText(e)), lc) {
				out = append(out, i)
				break
			}
		}
	}
	return out
}

// TransitionStats counts transitions per kind.
func (p *SSPresentation) TransitionStats() map[string]int {
	out := map[string]int{}
	for _, s := range p.Slides {
		key := ssTransitionName(s.Transition)
		out[key]++
	}
	return out
}

func ssTransitionName(t SSTransition) string {
	switch t {
	case SSTNone:     return "none"
	case SSTFade:     return "fade"
	case SSTSlide:    return "slide"
	case SSTDissolve: return "dissolve"
	}
	return "unknown"
}

func ssElementText(e SSElement) string {
	switch e.Kind {
	case SSEHeading: return e.Text
	case SSEBullet:  return e.Text
	case SSEImage:   return e.Alt
	case SSECode:    return e.Text
	}
	return ""
}

func ssRenderElement(e SSElement) string {
	switch e.Kind {
	case SSEHeading:
		level := e.Level
		if level < 1 { level = 1 }
		if level > 6 { level = 6 }
		return strings.Repeat("#", level) + " " + e.Text
	case SSEBullet:
		return strings.Repeat("  ", e.Indent) + "- " + e.Text
	case SSEImage:
		return fmt.Sprintf("![%s](%s)", e.Alt, e.Src)
	case SSECode:
		return fmt.Sprintf("```%s\n%s\n```", e.Language, e.Text)
	case SSESpacer:
		return strings.Repeat("\n", e.Lines)
	}
	return ""
}

func ssRenderSlide(s *SSSlide, mode SSMode) string {
	var b strings.Builder
	b.WriteString(s.Title)
	b.WriteByte('\n')
	width := len(s.Title)
	if width > 80 { width = 80 }
	b.WriteString(strings.Repeat("=", width))
	b.WriteByte('\n')
	for _, e := range s.Content {
		b.WriteString(ssRenderElement(e))
		b.WriteByte('\n')
	}
	if (mode == SSMSpeaker || mode == SSMHandout) && s.Notes != "" {
		b.WriteString("\n[Speaker notes]\n")
		b.WriteString(s.Notes)
		b.WriteByte('\n')
	}
	return b.String()
}
