package text

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

type Category string

const (
	News    Category = "news"
	Sports  Category = "sports"
	Finance Category = "finance"
	Alert   Category = "alert"
)

type TickerItem struct {
	Text      string
	Category  Category
	Priority  int // 1-5, 5 = highest
	Timestamp time.Time
	ExpiresAt *time.Time
}

func (item *TickerItem) IsExpired() bool {
	if item.ExpiresAt == nil {
		return false
	}
	return time.Now().After(*item.ExpiresAt)
}

type NewsTicker struct {
	Items []*TickerItem
}

func NewNewsTicker() *NewsTicker {
	return &NewsTicker{
		Items: make([]*TickerItem, 0),
	}
}

func (t *NewsTicker) Post(text string, category Category, priority int, ttlSeconds *int) (*TickerItem, error) {
	if priority < 1 || priority > 5 {
		return nil, fmt.Errorf("priority must be 1-5, got %d", priority)
	}

	validCategories := map[Category]bool{News: true, Sports: true, Finance: true, Alert: true}
	if !validCategories[category] {
		return nil, fmt.Errorf("invalid category: %s", category)
	}

	item := &TickerItem{
		Text:      text,
		Category:  category,
		Priority:  priority,
		Timestamp: time.Now(),
	}

	if ttlSeconds != nil {
		expires := time.Now().Add(time.Duration(*ttlSeconds) * time.Second)
		item.ExpiresAt = &expires
	}

	t.Items = append(t.Items, item)
	return item, nil
}

func (t *NewsTicker) Breaking(text string) (*TickerItem, error) {
	return t.Post(text, Alert, 5, nil)
}

func (t *NewsTicker) GetCurrent() []*TickerItem {
	var active []*TickerItem
	for _, item := range t.Items {
		if !item.IsExpired() {
			active = append(active, item)
		}
	}

	sort.Slice(active, func(i, j int) bool {
		if active[i].Priority != active[j].Priority {
			return active[i].Priority > active[j].Priority
		}
		return active[i].Timestamp.Before(active[j].Timestamp)
	})

	return active
}

func (t *NewsTicker) GetByCategory(cat Category) []*TickerItem {
	var result []*TickerItem
	for _, item := range t.GetCurrent() {
		if item.Category == cat {
			result = append(result, item)
		}
	}
	return result
}

func (t *NewsTicker) ExpireOld() int {
	before := len(t.Items)
	var kept []*TickerItem
	for _, item := range t.Items {
		if !item.IsExpired() {
			kept = append(kept, item)
		}
	}
	t.Items = kept
	return before - len(t.Items)
}

func (t *NewsTicker) Display() string {
	current := t.GetCurrent()
	if len(current) == 0 {
		return ">>> No items >>>"
	}

	texts := make([]string, len(current))
	for i, item := range current {
		texts[i] = item.Text
	}
	return ">>> " + strings.Join(texts, " | ") + " >>>"
}
