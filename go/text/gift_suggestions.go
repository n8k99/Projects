package text

import (
	"math/rand"
	"sort"
	"strings"
)

// Gift holds a gift with category, price range, and compatible traits.
type Gift struct {
	Name        string
	Category    string
	PriceRange  string // "budget", "mid", "premium"
	SuitableFor []string
}

// GiftSuggester is an attribute-based gift recommendation engine.
type GiftSuggester struct {
	gifts []Gift
}

// NewGiftSuggester creates a suggester pre-loaded with default gifts.
func NewGiftSuggester() *GiftSuggester {
	gs := &GiftSuggester{}
	gs.loadDefaults()
	return gs
}

func (gs *GiftSuggester) loadDefaults() {
	gs.gifts = []Gift{
		// Tech
		{"Mechanical Keyboard", "tech", "mid", []string{"techie", "creative"}},
		{"Raspberry Pi Kit", "tech", "budget", []string{"techie", "creative"}},
		{"Noise-Cancelling Headphones", "tech", "premium", []string{"techie", "music-lover"}},
		{"Smart Home Starter Kit", "tech", "mid", []string{"techie"}},
		// Books
		{"Leather-Bound Journal", "books", "mid", []string{"bookworm", "creative"}},
		{"Complete Tolkien Collection", "books", "premium", []string{"bookworm", "adventurer"}},
		{"Pocket Poetry Anthology", "books", "budget", []string{"bookworm", "creative"}},
		{"Cookbook: World Cuisines", "books", "mid", []string{"bookworm", "foodie"}},
		// Outdoor
		{"Hammock", "outdoor", "budget", []string{"adventurer"}},
		{"Hiking Backpack", "outdoor", "mid", []string{"adventurer"}},
		{"Camping Cookset", "outdoor", "mid", []string{"adventurer", "foodie"}},
		{"Trail Running Shoes", "outdoor", "premium", []string{"adventurer"}},
		// Cooking
		{"Cast Iron Skillet", "cooking", "budget", []string{"foodie"}},
		{"Spice Collection Box", "cooking", "mid", []string{"foodie", "adventurer"}},
		{"Chef's Knife Set", "cooking", "premium", []string{"foodie"}},
		{"Pasta Maker", "cooking", "mid", []string{"foodie", "creative"}},
		// Music
		{"Vinyl Record Starter Pack", "music", "budget", []string{"music-lover"}},
		{"Concert Tickets", "music", "mid", []string{"music-lover", "adventurer"}},
		{"Turntable", "music", "premium", []string{"music-lover"}},
		{"MIDI Controller", "music", "mid", []string{"music-lover", "techie", "creative"}},
		// Art
		{"Watercolor Set", "art", "budget", []string{"creative"}},
		{"Drawing Tablet", "art", "mid", []string{"creative", "techie"}},
		{"Museum Membership", "art", "mid", []string{"creative", "bookworm"}},
		{"Oil Paint Master Set", "art", "premium", []string{"creative"}},
	}
}

// AddGift adds a gift to the catalog.
func (gs *GiftSuggester) AddGift(name, category, priceRange string, suitableFor []string) {
	gs.gifts = append(gs.gifts, Gift{
		Name:        name,
		Category:    category,
		PriceRange:  priceRange,
		SuitableFor: suitableFor,
	})
}

// scoredGift pairs a relevance score with a gift for sorting.
type scoredGift struct {
	score int
	gift  Gift
}

// Suggest returns gifts matching the given traits, sorted by relevance (descending).
// If budget is non-empty, only gifts in that price range are included.
func (gs *GiftSuggester) Suggest(traits []string, budget string) []Gift {
	traitSet := make(map[string]bool)
	for _, t := range traits {
		traitSet[strings.ToLower(t)] = true
	}

	var scored []scoredGift
	for _, g := range gs.gifts {
		if budget != "" && g.PriceRange != budget {
			continue
		}
		matches := 0
		for _, s := range g.SuitableFor {
			if traitSet[strings.ToLower(s)] {
				matches++
			}
		}
		if matches > 0 {
			scored = append(scored, scoredGift{score: matches, gift: g})
		}
	}

	sort.Slice(scored, func(i, j int) bool {
		return scored[i].score > scored[j].score
	})

	result := make([]Gift, len(scored))
	for i, sg := range scored {
		result[i] = sg.gift
	}
	return result
}

// RandomSuggestion returns a random gift from the catalog.
func (gs *GiftSuggester) RandomSuggestion() (Gift, bool) {
	if len(gs.gifts) == 0 {
		return Gift{}, false
	}
	return gs.gifts[rand.Intn(len(gs.gifts))], true
}

// SuggestByCategory returns all gifts in a given category.
func (gs *GiftSuggester) SuggestByCategory(category string) []Gift {
	cat := strings.ToLower(category)
	var result []Gift
	for _, g := range gs.gifts {
		if strings.ToLower(g.Category) == cat {
			result = append(result, g)
		}
	}
	return result
}
