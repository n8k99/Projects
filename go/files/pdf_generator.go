// PDF Generator — document pagination via block flow layout.
package files

import (
	"fmt"
	"strings"
)

// BlockKind tags a document block.
type BlockKind int

const (
	BKHeading   BlockKind = 0
	BKParagraph BlockKind = 1
	BKSpacer    BlockKind = 2
	BKPageBreak BlockKind = 3
)

// PdfBlock is one document block.
type PdfBlock struct {
	Kind   BlockKind
	Text   string
	Level  int
	Height int
}

// PdfPageSize is the render geometry.
type PdfPageSize struct {
	WidthChars  int
	HeightLines int
}

// PdfA4Like is the default 80×66 page.
var PdfA4Like = PdfPageSize{WidthChars: 80, HeightLines: 66}

// RenderedLine is one placed line on a page.
type RenderedLine struct {
	Text      string
	Y         int
	IsHeading bool
}

// PdfPage is a rendered page.
type PdfPage struct {
	Lines  []RenderedLine
	Number int
}

// PdfDocument is a sequence of blocks + a page size.
type PdfDocument struct {
	PageSize PdfPageSize
	Blocks   []PdfBlock
}

// NewPdfDocument returns an empty document.
func NewPdfDocument(size PdfPageSize) *PdfDocument {
	return &PdfDocument{PageSize: size}
}

// Heading appends a heading block.
func (d *PdfDocument) Heading(text string, level int) *PdfDocument {
	d.Blocks = append(d.Blocks, PdfBlock{Kind: BKHeading, Text: text, Level: level})
	return d
}

// Paragraph appends a paragraph block.
func (d *PdfDocument) Paragraph(text string) *PdfDocument {
	d.Blocks = append(d.Blocks, PdfBlock{Kind: BKParagraph, Text: text})
	return d
}

// Spacer appends vertical space.
func (d *PdfDocument) Spacer(height int) *PdfDocument {
	d.Blocks = append(d.Blocks, PdfBlock{Kind: BKSpacer, Height: height})
	return d
}

// PageBreak appends a page break.
func (d *PdfDocument) PageBreak() *PdfDocument {
	d.Blocks = append(d.Blocks, PdfBlock{Kind: BKPageBreak})
	return d
}

// Render lays out the document into pages.
func (d *PdfDocument) Render() []PdfPage {
	pages := []PdfPage{}
	current := PdfPage{Number: 1}
	y := 0

	pushPage := func() {
		if len(current.Lines) > 0 {
			pages = append(pages, current)
			current = PdfPage{Number: current.Number + 1}
			y = 0
		}
	}

	for _, block := range d.Blocks {
		switch block.Kind {
		case BKPageBreak:
			pushPage()
		case BKSpacer:
			if y+block.Height > d.PageSize.HeightLines {
				pushPage()
			} else {
				y += block.Height
			}
		case BKHeading:
			if y+2 > d.PageSize.HeightLines {
				pushPage()
			}
			rendered := renderHeading(block.Text, block.Level)
			current.Lines = append(current.Lines, RenderedLine{Text: rendered, Y: y, IsHeading: true})
			y += 2
		case BKParagraph:
			for _, line := range WrapText(block.Text, d.PageSize.WidthChars) {
				if y >= d.PageSize.HeightLines {
					pushPage()
				}
				current.Lines = append(current.Lines, RenderedLine{Text: line, Y: y, IsHeading: false})
				y++
			}
			if y < d.PageSize.HeightLines {
				y++
			}
		}
	}

	if len(current.Lines) > 0 {
		pages = append(pages, current)
	}
	return pages
}

// PageCount returns the number of rendered pages.
func (d *PdfDocument) PageCount() int { return len(d.Render()) }

func renderHeading(text string, level int) string {
	switch level {
	case 1:
		return "# " + text
	case 2:
		return "## " + text
	default:
		return "### " + text
	}
}

// WrapText breaks text into lines fitting width at word boundaries.
func WrapText(text string, width int) []string {
	lines := []string{}
	current := ""
	for _, word := range strings.Fields(text) {
		switch {
		case current == "":
			current = word
		case len(current)+1+len(word) <= width:
			current += " " + word
		default:
			lines = append(lines, current)
			current = word
		}
	}
	if current != "" {
		lines = append(lines, current)
	}
	if len(lines) == 0 {
		lines = append(lines, "")
	}
	return lines
}

// String renders a compact summary for debugging.
func (p PdfPage) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Page %d:\n", p.Number)
	for _, l := range p.Lines {
		mark := " "
		if l.IsHeading {
			mark = "H"
		}
		fmt.Fprintf(&b, "  [%s] y=%2d  %s\n", mark, l.Y, l.Text)
	}
	return b.String()
}
