// RPG Character Stat Creator — class templates + derived stats + level progression.
package files

import (
	"fmt"
	"strings"
)

// Stat is a base attribute.
type Stat int

const (
	SStrength  Stat = 0
	SDexterity Stat = 1
	SIntellect Stat = 2
	SStamina   Stat = 3
	SLuck      Stat = 4
)

// AllStats returns every stat in canonical order.
func AllStats() []Stat {
	return []Stat{SStrength, SDexterity, SIntellect, SStamina, SLuck}
}

// StatName returns the lowercase name.
func StatName(s Stat) string {
	return [...]string{"strength", "dexterity", "intellect", "stamina", "luck"}[s]
}

// CharClass is a character class.
type CharClass int

const (
	CCWarrior CharClass = 0
	CCMage    CharClass = 1
	CCRogue   CharClass = 2
)

// ClassName returns the lowercase name.
func ClassName(c CharClass) string {
	return [...]string{"warrior", "mage", "rogue"}[c]
}

// Multipliers per class.
func Multipliers(c CharClass) map[Stat]float64 {
	m := map[Stat]float64{}
	for _, s := range AllStats() {
		m[s] = 1.0
	}
	switch c {
	case CCWarrior:
		m[SStrength] = 1.5
		m[SStamina] = 1.3
	case CCMage:
		m[SIntellect] = 1.5
		m[SStamina] = 0.8
	case CCRogue:
		m[SDexterity] = 1.4
		m[SLuck] = 1.3
	}
	return m
}

// MaxPerStat is the per-class stat cap.
func MaxPerStat(c CharClass) int { return 99 }

// Derived stats computed from base + level.
type Derived struct {
	MaxHP          int
	MaxMP          int
	Armour         int
	CarryCapacity  int
	CritChance     float64
}

// Character is a complete character sheet.
type Character struct {
	Name          string
	Class         CharClass
	Level         int
	Base          map[Stat]int
	UnspentPoints int
}

// LcgRng is a deterministic LCG.
type LcgRng struct{ state uint64 }

// NewLcg returns a seeded RNG.
func NewLcg(seed uint64) *LcgRng { return &LcgRng{state: seed + 1} }

// NextU32 returns the next pseudorandom word.
func (r *LcgRng) NextU32() uint32 {
	r.state = r.state*6364136223846793005 + 1442695040888963407
	return uint32(r.state >> 33)
}

// NextIn returns a value in [lo, lo+range).
func (r *LcgRng) NextIn(lo, rng uint32) uint32 { return lo + r.NextU32()%rng }

// RollCharacter creates a deterministic character from a seed.
func RollCharacter(name string, class CharClass, seed uint64) *Character {
	rng := NewLcg(seed)
	base := map[Stat]int{}
	for _, s := range AllStats() {
		base[s] = int(3 + rng.NextIn(0, 16))
	}
	return &Character{
		Name: name, Class: class, Level: 1, Base: base, UnspentPoints: 0,
	}
}

// Adjusted returns base * class multiplier, floored.
func (c *Character) Adjusted(stat Stat) int {
	b := c.Base[stat]
	m := Multipliers(c.Class)[stat]
	return int(float64(b) * m)
}

// Derived computes all derived stats.
func (c *Character) Derived() Derived {
	str := c.Adjusted(SStrength)
	int_ := c.Adjusted(SIntellect)
	sta := c.Adjusted(SStamina)
	dex := c.Adjusted(SDexterity)
	luk := c.Adjusted(SLuck)
	return Derived{
		MaxHP:         10 + sta*5 + c.Level*10,
		MaxMP:         int_*3 + c.Level*5,
		Armour:        str/3 + sta/5,
		CarryCapacity: 10 + str*2,
		CritChance:    (float64(luk)*0.5 + float64(dex)*0.2) / 100.0,
	}
}

// LevelUp increments level and grants 5 unspent points.
func (c *Character) LevelUp() {
	c.Level++
	c.UnspentPoints += 5
}

// SpendPoint raises a stat by N, enforcing caps and budget.
func (c *Character) SpendPoint(stat Stat, points int) error {
	if points > c.UnspentPoints {
		return fmt.Errorf("not enough points: have %d, need %d", c.UnspentPoints, points)
	}
	newVal := c.Base[stat] + points
	cap := MaxPerStat(c.Class)
	if newVal > cap {
		return fmt.Errorf("stat cap exceeded: %d > %d", newVal, cap)
	}
	c.Base[stat] = newVal
	c.UnspentPoints -= points
	return nil
}

// ToText serialises the character to a line-based format.
func (c *Character) ToText() string {
	var b strings.Builder
	fmt.Fprintf(&b, "name: %s\n", c.Name)
	fmt.Fprintf(&b, "class: %s\n", ClassName(c.Class))
	fmt.Fprintf(&b, "level: %d\n", c.Level)
	fmt.Fprintf(&b, "unspent: %d\n", c.UnspentPoints)
	for _, s := range AllStats() {
		if v, ok := c.Base[s]; ok {
			fmt.Fprintf(&b, "stat:%s=%d\n", StatName(s), v)
		}
	}
	return b.String()
}
