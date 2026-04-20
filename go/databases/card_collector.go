// Card Collector — multiset inventory with valuation and trade balancing.
package databases

import (
	"fmt"
	"math"
	"sort"
)

// Rarity classifies card scarcity.
type Rarity int

const (
	RarityCommon   Rarity = 0
	RarityUncommon Rarity = 1
	RarityRare     Rarity = 2
	RarityMythic   Rarity = 3
)

// Multiplier for a rarity.
func (r Rarity) Multiplier() float64 {
	switch r {
	case RarityCommon: return 1.0
	case RarityUncommon: return 2.0
	case RarityRare: return 8.0
	case RarityMythic: return 25.0
	}
	return 1.0
}

// CCCondition classifies a card's physical condition.
type CCCondition int

const (
	CondMint     CCCondition = 0
	CondNearMint CCCondition = 1
	CondPlayed   CCCondition = 2
	CondDamaged  CCCondition = 3
)

// Multiplier for a condition.
func (c CCCondition) Multiplier() float64 {
	switch c {
	case CondMint: return 1.0
	case CondNearMint: return 0.8
	case CondPlayed: return 0.4
	case CondDamaged: return 0.15
	}
	return 1.0
}

// CCCard is a catalogue entry.
type CCCard struct {
	SetCode        string
	Number         int
	Name           string
	Rarity         Rarity
	BasePriceCents int
}

// ID is "SET-###".
func (c *CCCard) ID() string {
	return fmt.Sprintf("%s-%03d", c.SetCode, c.Number)
}

// ValueCents returns the price in cents for a given condition.
func (c *CCCard) ValueCents(cond CCCondition) int {
	v := float64(c.BasePriceCents) * c.Rarity.Multiplier() * cond.Multiplier()
	return int(math.Round(v))
}

// CCCatalogue is a map of cards by ID.
type CCCatalogue struct {
	Cards map[string]*CCCard
}

// NewCCCatalogue returns an empty catalogue.
func NewCCCatalogue() *CCCatalogue { return &CCCatalogue{Cards: map[string]*CCCard{}} }

// Add stores a card.
func (cat *CCCatalogue) Add(c *CCCard) { cat.Cards[c.ID()] = c }

// Get looks up a card by ID.
func (cat *CCCatalogue) Get(id string) *CCCard { return cat.Cards[id] }

// AllIDs returns sorted card IDs.
func (cat *CCCatalogue) AllIDs() []string {
	ids := make([]string, 0, len(cat.Cards))
	for id := range cat.Cards { ids = append(ids, id) }
	sort.Strings(ids)
	return ids
}

// InventoryKey is (card_id, condition).
type InventoryKey struct {
	CardID    string
	Condition CCCondition
}

// CCCollection is a multiset of cards.
type CCCollection struct {
	Counts map[InventoryKey]int
}

// NewCCCollection returns an empty collection.
func NewCCCollection() *CCCollection { return &CCCollection{Counts: map[InventoryKey]int{}} }

// AddCard increments the count for a (card, condition) key.
func (c *CCCollection) AddCard(cardID string, cond CCCondition, qty int) {
	k := InventoryKey{cardID, cond}
	c.Counts[k] += qty
}

// RemoveCard decrements, deleting the key at zero. Errors if short.
func (c *CCCollection) RemoveCard(cardID string, cond CCCondition, qty int) error {
	k := InventoryKey{cardID, cond}
	cur := c.Counts[k]
	if cur < qty {
		return fmt.Errorf("not enough %s — have %d, need %d", cardID, cur, qty)
	}
	if cur == qty {
		delete(c.Counts, k)
	} else {
		c.Counts[k] = cur - qty
	}
	return nil
}

// TotalCards is the sum of all counts.
func (c *CCCollection) TotalCards() int {
	total := 0
	for _, n := range c.Counts { total += n }
	return total
}

// UniqueCards is distinct card IDs.
func (c *CCCollection) UniqueCards() int {
	ids := map[string]bool{}
	for k := range c.Counts { ids[k.CardID] = true }
	return len(ids)
}

// TotalValueCents sums each (card, condition, qty) valuation.
func (c *CCCollection) TotalValueCents(cat *CCCatalogue) int {
	total := 0
	for k, n := range c.Counts {
		if card := cat.Get(k.CardID); card != nil {
			total += card.ValueCents(k.Condition) * n
		}
	}
	return total
}

// Missing returns catalogue IDs not in the collection.
func (c *CCCollection) Missing(cat *CCCatalogue) []string {
	owned := map[string]bool{}
	for k := range c.Counts { owned[k.CardID] = true }
	var out []string
	for _, id := range cat.AllIDs() {
		if !owned[id] { out = append(out, id) }
	}
	return out
}

// TradeBundleItem is (card_id, condition, qty).
type TradeBundleItem struct {
	CardID    string
	Condition CCCondition
	Qty       int
}

// CCTradeBundle is a list of items.
type CCTradeBundle struct {
	Cards []TradeBundleItem
}

// NewCCTradeBundle returns an empty bundle.
func NewCCTradeBundle() *CCTradeBundle { return &CCTradeBundle{} }

// Add appends an item.
func (b *CCTradeBundle) Add(cardID string, cond CCCondition, qty int) {
	b.Cards = append(b.Cards, TradeBundleItem{cardID, cond, qty})
}

// ValueCents sums each item's valuation.
func (b *CCTradeBundle) ValueCents(cat *CCCatalogue) int {
	total := 0
	for _, it := range b.Cards {
		if c := cat.Get(it.CardID); c != nil {
			total += c.ValueCents(it.Condition) * it.Qty
		}
	}
	return total
}

// TradeVerdict is the outcome of evaluating a trade.
type TradeVerdict int

const (
	TVFair         TradeVerdict = 0
	TVOfferedMore  TradeVerdict = 1
	TVAskedMore    TradeVerdict = 2
)

// TradeEvaluation has verdict + diff.
type TradeEvaluation struct {
	Verdict   TradeVerdict
	DiffCents int
}

// EvaluateTrade compares two bundles.
func EvaluateTrade(offered, asked *CCTradeBundle, cat *CCCatalogue, tolerance int) TradeEvaluation {
	o := offered.ValueCents(cat)
	a := asked.ValueCents(cat)
	diff := o - a
	if diff < 0 { diff = -diff }
	if diff <= tolerance {
		return TradeEvaluation{Verdict: TVFair}
	}
	if o > a {
		return TradeEvaluation{Verdict: TVOfferedMore, DiffCents: o - a}
	}
	return TradeEvaluation{Verdict: TVAskedMore, DiffCents: a - o}
}
