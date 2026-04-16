package text

import (
	"math/rand"
)

// FortuneTeller generates randomized wisdom from categorized pools.
type FortuneTeller struct {
	pools map[string][]string
}

// NewFortuneTeller creates a fortune teller pre-loaded with default fortunes.
func NewFortuneTeller() *FortuneTeller {
	ft := &FortuneTeller{
		pools: map[string][]string{
			"wisdom":     {},
			"humor":      {},
			"warning":    {},
			"motivation": {},
			"mystery":    {},
		},
	}
	ft.loadDefaults()
	return ft
}

func (ft *FortuneTeller) loadDefaults() {
	ft.pools["wisdom"] = []string{
		"The obstacle is the path.",
		"What you resist persists.",
		"Still water runs deep.",
		"The map is not the territory.",
		"Every expert was once a beginner.",
	}
	ft.pools["humor"] = []string{
		"You will step on a Lego in the dark tonight.",
		"A closed mouth gathers no foot.",
		"Today is a good day to avoid making eye contact.",
		"Your socks will never match again.",
	}
	ft.pools["warning"] = []string{
		"Beware the shortcut that saves no time.",
		"Not every open door is an invitation.",
		"The comfortable path leads to the boring destination.",
		"Trust your instincts — they remember what you forgot.",
	}
	ft.pools["motivation"] = []string{
		"You are closer than you think.",
		"The best time to start was yesterday. The second best is now.",
		"Doubt kills more dreams than failure ever will.",
		"Small steps still move you forward.",
		"Discipline is choosing what you want most over what you want now.",
	}
	ft.pools["mystery"] = []string{
		"Something lost will find you when you stop looking.",
		"The answer you seek is in a room you haven't entered yet.",
		"A stranger already knows your name.",
		"The next full moon brings clarity.",
	}
}

// TellFortune returns a random fortune from all categories.
func (ft *FortuneTeller) TellFortune() string {
	all := ft.allFortunes()
	if len(all) == 0 {
		return "The spirits are silent."
	}
	return all[rand.Intn(len(all))]
}

// TellFortuneByCategory returns a random fortune from the specified category.
// Returns empty string if category doesn't exist or is empty.
func (ft *FortuneTeller) TellFortuneByCategory(category string) string {
	pool, ok := ft.pools[category]
	if !ok || len(pool) == 0 {
		return ""
	}
	return pool[rand.Intn(len(pool))]
}

// AddFortune adds a fortune to a category pool.
func (ft *FortuneTeller) AddFortune(text, category string) {
	ft.pools[category] = append(ft.pools[category], text)
}

// AskQuestion acknowledges the question, then returns a random fortune regardless.
func (ft *FortuneTeller) AskQuestion(question string) string {
	_ = question
	return ft.TellFortune()
}

// FortuneCount returns the total number of fortunes across all categories.
func (ft *FortuneTeller) FortuneCount() int {
	total := 0
	for _, pool := range ft.pools {
		total += len(pool)
	}
	return total
}

func (ft *FortuneTeller) allFortunes() []string {
	var all []string
	for _, pool := range ft.pools {
		all = append(all, pool...)
	}
	return all
}
