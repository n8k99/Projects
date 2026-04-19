// Text Based Game — room graph + player state + command dispatch.
package web

import (
	"fmt"
	"sort"
	"strings"
)

// TGItemKind classifies an item.
type TGItemKind int

const (
	TGWeapon TGItemKind = iota
	TGPotion
	TGTreasure
)

// TGItem is one item in the world.
type TGItem struct {
	ID        int
	Name      string
	Kind      TGItemKind
	Damage    int
	Heal      int
	WorthGold int
}

// TGEnemy is one enemy.
type TGEnemy struct {
	ID     int
	Name   string
	Health int
	Damage int
}

// TGRoom is one location.
type TGRoom struct {
	ID          int
	Name        string
	Description string
	Exits       map[string]int
	Items       []TGItem
	Enemies     []TGEnemy
}

// TGPlayer is the player's state.
type TGPlayer struct {
	Location          int
	Health, MaxHealth int
	Gold              int
	Inventory         []TGItem
	EquippedWeaponID  int // 0 = none
	EquippedDamage    int
}

// TGGameState is the game's FSM state.
type TGGameState int

const (
	TGPlaying TGGameState = iota
	TGDead
	TGWon
)

// TGWorld is the game world.
type TGWorld struct {
	Rooms           map[int]*TGRoom
	Player          TGPlayer
	State           TGGameState
	Turn            int
	WinGold         int
	Log             []string
}

// NewTGDemoWorld builds the 3-room demo dungeon.
func NewTGDemoWorld() *TGWorld {
	w := &TGWorld{
		Rooms:   map[int]*TGRoom{},
		Player:  TGPlayer{Location: 1, Health: 20, MaxHealth: 20, EquippedDamage: 1},
		State:   TGPlaying,
		WinGold: 50,
	}
	w.Rooms[1] = &TGRoom{
		ID: 1, Name: "Entrance Hall",
		Description: "A dusty hall with exits north and east.",
		Exits:       map[string]int{"north": 2, "east": 3},
		Items: []TGItem{
			{ID: 100, Name: "rusty sword", Kind: TGWeapon, Damage: 5},
			{ID: 101, Name: "healing potion", Kind: TGPotion, Heal: 15},
		},
	}
	w.Rooms[2] = &TGRoom{
		ID: 2, Name: "Damp Cavern",
		Description: "A cold cavern. Something shifts in the shadows.",
		Exits:       map[string]int{"south": 1},
		Enemies:     []TGEnemy{{ID: 200, Name: "goblin", Health: 10, Damage: 2}},
	}
	w.Rooms[3] = &TGRoom{
		ID: 3, Name: "Treasure Room",
		Description: "A chalice rests on a pedestal. Exit west.",
		Exits:       map[string]int{"west": 1},
		Items:       []TGItem{{ID: 102, Name: "gold chalice", Kind: TGTreasure, WorthGold: 50}},
	}
	w.Log = append(w.Log, "You stand at the entrance of an old dungeon.")
	return w
}

// CurrentRoom returns the player's current room.
func (w *TGWorld) CurrentRoom() *TGRoom { return w.Rooms[w.Player.Location] }

var tgDirAliases = map[string]string{"n": "north", "s": "south", "e": "east", "w": "west"}
var tgProvokes = map[string]bool{
	"go": true, "move": true,
	"north": true, "south": true, "east": true, "west": true,
	"n": true, "s": true, "e": true, "w": true,
	"take": true, "get": true, "use": true, "fight": true, "attack": true,
}

// Execute processes one command and returns the new log lines.
func (w *TGWorld) Execute(cmd string) []string {
	before := len(w.Log)
	if w.State != TGPlaying {
		w.log("The game is over. Reset to play again.")
		return w.Log[before:]
	}
	trimmed := strings.ToLower(strings.TrimSpace(cmd))
	if trimmed == "" {
		return nil
	}
	parts := strings.Fields(trimmed)
	verb := parts[0]
	arg := strings.Join(parts[1:], " ")
	raw := verb
	if alias, ok := tgDirAliases[verb]; ok {
		verb, arg = "go", alias
	}
	switch verb {
	case "north", "south", "east", "west":
		arg = verb
		verb = "go"
	}
	switch verb {
	case "look", "l":
		w.doLook()
	case "go", "move":
		w.doGo(arg)
	case "take", "get":
		w.doTake(arg)
	case "use":
		w.doUse(arg)
	case "equip", "wield":
		w.doEquip(arg)
	case "fight", "attack":
		w.doFight(arg)
	case "inventory", "inv", "i":
		w.doInventory()
	case "stats":
		w.doStats()
	default:
		w.log(fmt.Sprintf("I don't understand '%s'.", verb))
	}
	if tgProvokes[raw] {
		w.Turn++
		w.enemiesStrike()
		w.checkEndConditions()
	}
	return w.Log[before:]
}

func (w *TGWorld) log(s string) { w.Log = append(w.Log, s) }

func (w *TGWorld) doLook() {
	r := w.CurrentRoom()
	w.log(fmt.Sprintf("%s: %s", r.Name, r.Description))
	if len(r.Items) > 0 {
		names := make([]string, len(r.Items))
		for i, it := range r.Items {
			names[i] = it.Name
		}
		w.log("You see here: " + strings.Join(names, ", "))
	}
	if len(r.Enemies) > 0 {
		names := make([]string, len(r.Enemies))
		for i, e := range r.Enemies {
			names[i] = fmt.Sprintf("%s (%dhp)", e.Name, e.Health)
		}
		w.log("Enemies: " + strings.Join(names, ", "))
	}
	dirs := make([]string, 0, len(r.Exits))
	for d := range r.Exits {
		dirs = append(dirs, d)
	}
	sort.Strings(dirs)
	w.log("Exits: " + strings.Join(dirs, ", "))
}

func (w *TGWorld) doGo(direction string) {
	if direction == "" {
		w.log("Go where?")
		return
	}
	dest, ok := w.CurrentRoom().Exits[direction]
	if !ok {
		w.log(fmt.Sprintf("You can't go %s from here.", direction))
		return
	}
	w.Player.Location = dest
	w.log(fmt.Sprintf("You travel %s. You are now in %s.", direction, w.CurrentRoom().Name))
}

func (w *TGWorld) doTake(name string) {
	if name == "" {
		w.log("Take what?")
		return
	}
	r := w.CurrentRoom()
	for i, it := range r.Items {
		if strings.Contains(strings.ToLower(it.Name), name) {
			w.Player.Inventory = append(w.Player.Inventory, it)
			r.Items = append(r.Items[:i], r.Items[i+1:]...)
			w.log(fmt.Sprintf("You take the %s.", it.Name))
			return
		}
	}
	w.log(fmt.Sprintf("There is no '%s' here.", name))
}

func (w *TGWorld) doUse(name string) {
	if name == "" {
		w.log("Use what?")
		return
	}
	for i, it := range w.Player.Inventory {
		if strings.Contains(strings.ToLower(it.Name), name) {
			switch it.Kind {
			case TGPotion:
				before := w.Player.Health
				w.Player.Health += it.Heal
				if w.Player.Health > w.Player.MaxHealth {
					w.Player.Health = w.Player.MaxHealth
				}
				w.Player.Inventory = append(w.Player.Inventory[:i],
					w.Player.Inventory[i+1:]...)
				w.log(fmt.Sprintf("You drink the %s. Health %d → %d.",
					it.Name, before, w.Player.Health))
			case TGTreasure:
				w.Player.Gold += it.WorthGold
				w.Player.Inventory = append(w.Player.Inventory[:i],
					w.Player.Inventory[i+1:]...)
				w.log(fmt.Sprintf("You pocket the %s (+%d gold, total %d).",
					it.Name, it.WorthGold, w.Player.Gold))
			case TGWeapon:
				w.Player.EquippedWeaponID = it.ID
				w.Player.EquippedDamage = it.Damage
				w.log(fmt.Sprintf("You wield the %s (%d damage).", it.Name, it.Damage))
			}
			return
		}
	}
	w.log(fmt.Sprintf("You don't have a '%s'.", name))
}

func (w *TGWorld) doEquip(name string) {
	if name == "" {
		w.log("Equip what?")
		return
	}
	for _, it := range w.Player.Inventory {
		if strings.Contains(strings.ToLower(it.Name), name) {
			if it.Kind != TGWeapon {
				w.log(fmt.Sprintf("The %s is not a weapon.", it.Name))
				return
			}
			w.Player.EquippedWeaponID = it.ID
			w.Player.EquippedDamage = it.Damage
			w.log(fmt.Sprintf("You wield the %s (%d damage).", it.Name, it.Damage))
			return
		}
	}
	w.log(fmt.Sprintf("You don't have a '%s'.", name))
}

func (w *TGWorld) doFight(name string) {
	r := w.CurrentRoom()
	target := -1
	if name == "" && len(r.Enemies) > 0 {
		target = 0
	} else {
		for i, e := range r.Enemies {
			if strings.Contains(strings.ToLower(e.Name), name) {
				target = i
				break
			}
		}
	}
	if target < 0 {
		w.log(fmt.Sprintf("There's no '%s' to fight here.", name))
		return
	}
	enemy := &r.Enemies[target]
	enemy.Health -= w.Player.EquippedDamage
	hp := enemy.Health
	if hp < 0 {
		hp = 0
	}
	w.log(fmt.Sprintf("You hit the %s for %d damage (%dhp remaining).",
		enemy.Name, w.Player.EquippedDamage, hp))
	if enemy.Health <= 0 {
		defeated := r.Enemies[target]
		r.Enemies = append(r.Enemies[:target], r.Enemies[target+1:]...)
		w.log(fmt.Sprintf("The %s falls defeated.", defeated.Name))
	}
}

func (w *TGWorld) doInventory() {
	if len(w.Player.Inventory) == 0 {
		w.log("Your pack is empty.")
		return
	}
	names := make([]string, len(w.Player.Inventory))
	for i, it := range w.Player.Inventory {
		names[i] = it.Name
	}
	w.log("You carry: " + strings.Join(names, ", ") + ".")
}

func (w *TGWorld) doStats() {
	wpn := "bare hands"
	for _, it := range w.Player.Inventory {
		if it.ID == w.Player.EquippedWeaponID {
			wpn = it.Name
			break
		}
	}
	w.log(fmt.Sprintf("Turn %d. Health %d/%d. Gold %d. Damage %d (equipped: %s).",
		w.Turn, w.Player.Health, w.Player.MaxHealth, w.Player.Gold,
		w.Player.EquippedDamage, wpn))
}

func (w *TGWorld) enemiesStrike() {
	r := w.CurrentRoom()
	total := 0
	for _, e := range r.Enemies {
		total += e.Damage
	}
	if total > 0 {
		w.Player.Health -= total
		hp := w.Player.Health
		if hp < 0 {
			hp = 0
		}
		w.log(fmt.Sprintf("Enemies strike you for %d damage (%d hp)!", total, hp))
	}
}

func (w *TGWorld) checkEndConditions() {
	switch {
	case w.Player.Health <= 0:
		w.State = TGDead
		w.log("You fall, lifeless. The dungeon is silent.")
	case w.Player.Gold >= w.WinGold:
		w.State = TGWon
		w.log(fmt.Sprintf("With %d gold in hand, you've proven your fortune. You win!",
			w.Player.Gold))
	}
}
