// Page Scraper — parse HTML-like markup into a tree, query with selectors.
//
// Forgiving parser: unknown constructs become text, stray close tags drop,
// recovery at the next valid boundary. Selectors: CSS subset (tag, .class,
// #id, combinations, descendant combinator via whitespace).
package web

import (
	"strings"
	"unicode"
)

// ScrapedNode is either an Element or Text.
type ScrapedNode interface{ isScraped() }

// ScrapedElement is a parsed tag with attributes and children.
type ScrapedElement struct {
	Tag      string
	Attrs    map[string]string
	Children []ScrapedNode
}

func (*ScrapedElement) isScraped() {}

// ScrapedText is a text node.
type ScrapedText struct{ Text string }

func (*ScrapedText) isScraped() {}

var voidTags = map[string]bool{
	"area": true, "base": true, "br": true, "col": true, "embed": true,
	"hr": true, "img": true, "input": true, "link": true, "meta": true,
	"source": true, "track": true, "wbr": true,
}

// ParseHTML parses the given HTML-like string into a slice of nodes.
func ParseHTML(html string) []ScrapedNode {
	p := &scrapeParser{s: html}
	return p.parseNodes("")
}

type scrapeParser struct {
	s string
	i int
}

func (p *scrapeParser) eof() bool { return p.i >= len(p.s) }
func (p *scrapeParser) peek() byte {
	if p.eof() {
		return 0
	}
	return p.s[p.i]
}
func (p *scrapeParser) startsWith(prefix string) bool {
	return strings.HasPrefix(p.s[p.i:], prefix)
}
func (p *scrapeParser) consumeWhile(pred func(byte) bool) string {
	start := p.i
	for !p.eof() && pred(p.s[p.i]) {
		p.i++
	}
	return p.s[start:p.i]
}
func (p *scrapeParser) consumeWS() {
	p.consumeWhile(func(c byte) bool { return unicode.IsSpace(rune(c)) })
}

func (p *scrapeParser) parseNodes(parentTag string) []ScrapedNode {
	out := []ScrapedNode{}
	for !p.eof() {
		if parentTag != "" && p.startsWith("</"+parentTag) {
			break
		}
		if p.startsWith("</") {
			p.consumeWhile(func(c byte) bool { return c != '>' })
			if !p.eof() {
				p.i++
			}
			continue
		}
		if p.peek() == '<' && p.i+1 < len(p.s) && isAlpha(p.s[p.i+1]) {
			if el := p.parseElement(); el != nil {
				out = append(out, el)
			}
			continue
		}
		t := p.consumeWhile(func(c byte) bool { return c != '<' })
		if t != "" {
			out = append(out, &ScrapedText{Text: t})
		}
	}
	return out
}

func (p *scrapeParser) parseElement() ScrapedNode {
	p.i++ // consume <
	tag := strings.ToLower(p.consumeWhile(func(c byte) bool {
		return isAlnum(c) || c == '-'
	}))
	if tag == "" {
		return nil
	}
	attrs := p.parseAttrs()
	selfClose := false
	switch {
	case p.startsWith("/>"):
		p.i += 2
		selfClose = true
	case p.startsWith(">"):
		p.i++
	default:
		return nil
	}
	if selfClose || voidTags[tag] {
		return &ScrapedElement{Tag: tag, Attrs: attrs, Children: nil}
	}
	children := p.parseNodes(tag)
	if p.startsWith("</" + tag) {
		p.consumeWhile(func(c byte) bool { return c != '>' })
		if !p.eof() {
			p.i++
		}
	}
	return &ScrapedElement{Tag: tag, Attrs: attrs, Children: children}
}

func (p *scrapeParser) parseAttrs() map[string]string {
	attrs := map[string]string{}
	for {
		p.consumeWS()
		if p.eof() || p.peek() == '>' || p.peek() == '/' {
			return attrs
		}
		name := strings.ToLower(p.consumeWhile(func(c byte) bool {
			return isAlnum(c) || c == '-' || c == '_'
		}))
		if name == "" {
			p.i++
			continue
		}
		p.consumeWS()
		value := ""
		if p.startsWith("=") {
			p.i++
			p.consumeWS()
			if !p.eof() && (p.peek() == '"' || p.peek() == '\'') {
				q := p.peek()
				p.i++
				value = p.consumeWhile(func(c byte) bool { return c != q })
				if !p.eof() {
					p.i++
				}
			} else {
				value = p.consumeWhile(func(c byte) bool {
					return !unicode.IsSpace(rune(c)) && c != '>' && c != '/'
				})
			}
		}
		attrs[name] = value
	}
}

func isAlpha(c byte) bool { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') }
func isAlnum(c byte) bool { return isAlpha(c) || (c >= '0' && c <= '9') }

type simpleSelector struct {
	tag, class, id string
}

func parseSimple(s string) simpleSelector {
	var simp simpleSelector
	buf := ""
	kind := ' '
	flush := func() {
		if buf == "" {
			return
		}
		switch kind {
		case '.':
			simp.class = buf
		case '#':
			simp.id = buf
		default:
			simp.tag = strings.ToLower(buf)
		}
		buf = ""
	}
	for _, c := range s {
		if c == '.' || c == '#' {
			flush()
			kind = c
		} else {
			buf += string(c)
		}
	}
	flush()
	return simp
}

func simpleMatches(simp simpleSelector, el *ScrapedElement) bool {
	if simp.tag != "" && simp.tag != el.Tag {
		return false
	}
	if simp.id != "" && el.Attrs["id"] != simp.id {
		return false
	}
	if simp.class != "" {
		classes := strings.Fields(el.Attrs["class"])
		found := false
		for _, c := range classes {
			if c == simp.class {
				found = true
				break
			}
		}
		if !found {
			return false
		}
	}
	return true
}

// Select returns all elements matching the (space-separated descendant) selector.
func Select(nodes []ScrapedNode, selector string) []*ScrapedElement {
	parts := strings.Fields(selector)
	chain := make([]simpleSelector, len(parts))
	for i, p := range parts {
		chain[i] = parseSimple(p)
	}
	var out []*ScrapedElement
	var rec func(ns []ScrapedNode, idx int)
	rec = func(ns []ScrapedNode, idx int) {
		for _, n := range ns {
			el, ok := n.(*ScrapedElement)
			if !ok {
				continue
			}
			match := simpleMatches(chain[idx], el)
			switch {
			case match && idx == len(chain)-1:
				out = append(out, el)
				rec(el.Children, idx)
			case match:
				rec(el.Children, idx+1)
			default:
				rec(el.Children, idx)
			}
		}
	}
	rec(nodes, 0)
	return out
}

// TextOf returns the concatenation of all text descendants of node.
func TextOf(n ScrapedNode) string {
	switch v := n.(type) {
	case *ScrapedText:
		return v.Text
	case *ScrapedElement:
		var sb strings.Builder
		for _, c := range v.Children {
			sb.WriteString(TextOf(c))
		}
		return sb.String()
	}
	return ""
}
