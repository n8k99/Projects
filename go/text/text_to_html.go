package text

import (
	"regexp"
	"strings"
)

var (
	reCode   = regexp.MustCompile("`([^`]+)`")
	reBold   = regexp.MustCompile(`\*\*(.+?)\*\*`)
	reItalic = regexp.MustCompile(`\*(.+?)\*`)
	reLink   = regexp.MustCompile(`\[([^\]]+)\]\(([^)]+)\)`)
	reHeader = regexp.MustCompile(`^(#{1,3})\s+(.+)$`)
)

// escapeHTML replaces &, <, > with their HTML entities.
func escapeHTML(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	return s
}

// applyInline converts inline formatting: code, bold, italic, links.
func applyInline(s string) string {
	s = reCode.ReplaceAllString(s, "<code>$1</code>")
	s = reBold.ReplaceAllString(s, "<strong>$1</strong>")
	s = reItalic.ReplaceAllString(s, "<em>$1</em>")
	s = reLink.ReplaceAllString(s, `<a href="$2">$1</a>`)
	return s
}

// TextToHTML converts formatted plain text to HTML.
//
// Supports paragraphs, bold, italic, headers (H1-H3), links,
// unordered lists, inline code, line breaks, and HTML entity escaping.
func TextToHTML(text string) string {
	lines := strings.Split(text, "\n")
	var blocks []string
	var paragraph []string
	var listItems []string

	flushParagraph := func() {
		if len(paragraph) == 0 {
			return
		}
		joined := strings.Join(paragraph, "<br>\n")
		blocks = append(blocks, "<p>"+applyInline(joined)+"</p>")
		paragraph = nil
	}

	flushList := func() {
		if len(listItems) == 0 {
			return
		}
		var items []string
		for _, item := range listItems {
			items = append(items, "<li>"+applyInline(item)+"</li>")
		}
		blocks = append(blocks, "<ul>\n"+strings.Join(items, "\n")+"\n</ul>")
		listItems = nil
	}

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Blank line
		if trimmed == "" {
			flushList()
			flushParagraph()
			continue
		}

		// Header
		if m := reHeader.FindStringSubmatch(trimmed); m != nil {
			flushList()
			flushParagraph()
			level := len(m[1])
			content := escapeHTML(m[2])
			blocks = append(blocks, "<h"+string(rune('0'+level))+">"+applyInline(content)+"</h"+string(rune('0'+level))+">")
			continue
		}

		// List item
		if strings.HasPrefix(trimmed, "- ") {
			flushParagraph()
			listItems = append(listItems, escapeHTML(trimmed[2:]))
			continue
		}

		// Flush list if transitioning out
		if len(listItems) > 0 {
			flushList()
		}

		// Regular paragraph text
		paragraph = append(paragraph, escapeHTML(trimmed))
	}

	flushList()
	flushParagraph()

	return strings.Join(blocks, "\n")
}
