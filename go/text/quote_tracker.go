package text

import (
	"fmt"
	"math/rand"
	"sort"
	"strings"
)

// Quote holds a single quote with attribution and categorization.
type Quote struct {
	Text   string
	Author string
	Source  string
	Tags   []string
}

// String returns a formatted representation of the quote.
func (q Quote) String() string {
	s := fmt.Sprintf("%q — %s", q.Text, q.Author)
	if q.Source != "" {
		s += fmt.Sprintf(" (%s)", q.Source)
	}
	if len(q.Tags) > 0 {
		s += fmt.Sprintf(" [%s]", strings.Join(q.Tags, ", "))
	}
	return s
}

// QuoteTracker is a curated collection of quotes with search and random access.
type QuoteTracker struct {
	quotes []Quote
}

// NewQuoteTracker creates an empty QuoteTracker.
func NewQuoteTracker() *QuoteTracker {
	return &QuoteTracker{}
}

// AddQuote adds a quote to the collection.
func (qt *QuoteTracker) AddQuote(text, author, source string, tags []string) Quote {
	q := Quote{
		Text:   text,
		Author: author,
		Source:  source,
		Tags:   tags,
	}
	qt.quotes = append(qt.quotes, q)
	return q
}

// RandomQuote returns a random quote and true, or the zero Quote and false if empty.
func (qt *QuoteTracker) RandomQuote() (Quote, bool) {
	if len(qt.quotes) == 0 {
		return Quote{}, false
	}
	return qt.quotes[rand.Intn(len(qt.quotes))], true
}

// SearchByAuthor returns all quotes whose author contains the query (case-insensitive).
func (qt *QuoteTracker) SearchByAuthor(author string) []Quote {
	needle := strings.ToLower(author)
	var results []Quote
	for _, q := range qt.quotes {
		if strings.Contains(strings.ToLower(q.Author), needle) {
			results = append(results, q)
		}
	}
	return results
}

// SearchByTag returns all quotes that have the given tag (case-insensitive).
func (qt *QuoteTracker) SearchByTag(tag string) []Quote {
	needle := strings.ToLower(tag)
	var results []Quote
	for _, q := range qt.quotes {
		for _, t := range q.Tags {
			if strings.ToLower(t) == needle {
				results = append(results, q)
				break
			}
		}
	}
	return results
}

// ListAuthors returns a sorted slice of unique author names.
func (qt *QuoteTracker) ListAuthors() []string {
	seen := make(map[string]bool)
	for _, q := range qt.quotes {
		seen[q.Author] = true
	}
	authors := make([]string, 0, len(seen))
	for a := range seen {
		authors = append(authors, a)
	}
	sort.Strings(authors)
	return authors
}

// ListTags returns a sorted slice of unique tags.
func (qt *QuoteTracker) ListTags() []string {
	seen := make(map[string]bool)
	for _, q := range qt.quotes {
		for _, t := range q.Tags {
			seen[t] = true
		}
	}
	tags := make([]string, 0, len(seen))
	for t := range seen {
		tags = append(tags, t)
	}
	sort.Strings(tags)
	return tags
}

// Count returns the total number of quotes.
func (qt *QuoteTracker) Count() int {
	return len(qt.quotes)
}
