//! Text Based Game — room graph + player state + command dispatch.
//!
//! Combines primitives from earlier projects: G064's graph (rooms connected
//! by exits), G073's command dispatch, G077/G078's state FSMs (alive, dead,
//! won). Each command is a turn; the world responds (enemies attack, etc.)
//! and produces a log of messages.
//!
//! This is the **text-adventure** shape: describe → prompt → parse →
//! dispatch → mutate → describe. Every ancient Infocom game, every MUD,
//! every modern interactive fiction tool is built on this loop.

use std::collections::HashMap;

pub type RoomId = u32;
pub type ItemId = u32;
pub type EnemyId = u32;

#[derive(Debug, Clone)]
pub enum ItemKind {
    Weapon { damage: i32 },
    Potion { heal: i32 },
    Treasure { worth_gold: u32 },
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: EnemyId,
    pub name: String,
    pub health: i32,
    pub damage: i32,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, RoomId>,
    pub items: Vec<Item>,
    pub enemies: Vec<Enemy>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub location: RoomId,
    pub health: i32,
    pub max_health: i32,
    pub gold: u32,
    pub inventory: Vec<Item>,
    pub equipped_weapon_id: Option<ItemId>,
    pub equipped_damage: i32,        // includes bare-fists base damage
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Dead,
    Won,
}

pub struct World {
    pub rooms: HashMap<RoomId, Room>,
    pub player: Player,
    pub state: GameState,
    pub turn: u32,
    pub win_condition_gold: u32,
    pub log: Vec<String>,
}

impl World {
    /// Build the demo world: 3 rooms (entrance, cavern, treasury), goblin
    /// in the cavern, sword and healing potion scattered about.
    pub fn demo() -> Self {
        let mut rooms = HashMap::new();

        let sword = Item {
            id: 100, name: "rusty sword".into(),
            kind: ItemKind::Weapon { damage: 5 },
        };
        let potion = Item {
            id: 101, name: "healing potion".into(),
            kind: ItemKind::Potion { heal: 15 },
        };
        let treasure = Item {
            id: 102, name: "gold chalice".into(),
            kind: ItemKind::Treasure { worth_gold: 50 },
        };
        let goblin = Enemy {
            id: 200, name: "goblin".into(), health: 10, damage: 2,
        };

        rooms.insert(1, Room {
            id: 1, name: "Entrance Hall".into(),
            description: "A dusty hall. Torchlight flickers on the walls. \
                          Doorways lead north and east.".into(),
            exits: HashMap::from([("north".into(), 2), ("east".into(), 3)]),
            items: vec![sword, potion.clone()],
            enemies: Vec::new(),
        });
        rooms.insert(2, Room {
            id: 2, name: "Damp Cavern".into(),
            description: "A cold cavern smelling of mildew. Something shifts \
                          in the shadows. A passage leads south.".into(),
            exits: HashMap::from([("south".into(), 1)]),
            items: Vec::new(),
            enemies: vec![goblin],
        });
        rooms.insert(3, Room {
            id: 3, name: "Treasure Room".into(),
            description: "A small room gleaming with gold. A lone chalice \
                          rests on a pedestal. Exit west.".into(),
            exits: HashMap::from([("west".into(), 1)]),
            items: vec![treasure],
            enemies: Vec::new(),
        });

        Self {
            rooms,
            player: Player {
                location: 1, health: 20, max_health: 20, gold: 0,
                inventory: Vec::new(),
                equipped_weapon_id: None, equipped_damage: 1, // bare hands
            },
            state: GameState::Playing,
            turn: 0,
            win_condition_gold: 50,
            log: vec!["You stand at the entrance of an old dungeon.".into()],
        }
    }

    pub fn current_room(&self) -> &Room {
        self.rooms.get(&self.player.location).unwrap()
    }

    /// Parse and execute one command. Returns the lines added to the log.
    /// Every command counts as one turn (even invalid ones — the world
    /// still reacts to your failed attempt).
    pub fn execute(&mut self, cmd: &str) -> Vec<String> {
        let before = self.log.len();
        if !matches!(self.state, GameState::Playing) {
            self.log.push("The game is over. Reset to play again.".into());
            return self.log[before..].to_vec();
        }

        let trimmed = cmd.trim().to_lowercase();
        let mut parts = trimmed.split_whitespace();
        let verb = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();
        let arg = rest.join(" ");

        match verb {
            "" => {} // no turn consumed on empty input
            "look" | "l" => self.do_look(),
            "go" | "move" => self.do_go(&arg),
            "north" | "south" | "east" | "west" | "n" | "s" | "e" | "w" => {
                self.do_go(verb_to_direction(verb))
            }
            "take" | "get" => self.do_take(&arg),
            "use" => self.do_use(&arg),
            "equip" | "wield" => self.do_equip(&arg),
            "fight" | "attack" => self.do_fight(&arg),
            "inventory" | "inv" | "i" => self.do_inventory(),
            "stats" => self.do_stats(),
            _ => self.log.push(format!("I don't understand '{verb}'.")),
        }

        // World reacts: enemies in the current room attack (if any and if
        // player took a real action; empty input and stats/inventory don't
        // provoke).
        let provokes = matches!(verb,
            "go" | "move" | "north" | "south" | "east" | "west" |
            "n" | "s" | "e" | "w" | "take" | "get" | "use" | "fight" | "attack");
        if provokes {
            self.turn += 1;
            self.enemies_strike();
            self.check_end_conditions();
        }

        self.log[before..].to_vec()
    }

    fn do_look(&mut self) {
        let room = self.current_room().clone();
        self.log.push(format!("{}: {}", room.name, room.description));
        if !room.items.is_empty() {
            let names: Vec<&str> = room.items.iter().map(|i| i.name.as_str()).collect();
            self.log.push(format!("You see here: {}", names.join(", ")));
        }
        if !room.enemies.is_empty() {
            let names: Vec<String> = room.enemies.iter()
                .map(|e| format!("{} ({}hp)", e.name, e.health)).collect();
            self.log.push(format!("Enemies: {}", names.join(", ")));
        }
        let exits: Vec<&String> = room.exits.keys().collect();
        let mut sorted_exits: Vec<&String> = exits.into_iter().collect();
        sorted_exits.sort();
        let exit_strs: Vec<&str> = sorted_exits.iter().map(|s| s.as_str()).collect();
        self.log.push(format!("Exits: {}", exit_strs.join(", ")));
    }

    fn do_go(&mut self, direction: &str) {
        if direction.is_empty() {
            self.log.push("Go where?".into());
            return;
        }
        let room = self.current_room();
        match room.exits.get(direction).copied() {
            Some(dest) => {
                self.player.location = dest;
                let name = self.current_room().name.clone();
                self.log.push(format!("You travel {direction}. You are now in {name}."));
            }
            None => self.log.push(format!("You can't go {direction} from here.")),
        }
    }

    fn do_take(&mut self, item_name: &str) {
        if item_name.is_empty() {
            self.log.push("Take what?".into());
            return;
        }
        let loc = self.player.location;
        let room = self.rooms.get_mut(&loc).unwrap();
        let pos = room.items.iter().position(|i| i.name.to_lowercase().contains(item_name));
        match pos {
            Some(p) => {
                let item = room.items.remove(p);
                self.log.push(format!("You take the {}.", item.name));
                self.player.inventory.push(item);
            }
            None => self.log.push(format!("There is no '{item_name}' here.")),
        }
    }

    fn do_use(&mut self, item_name: &str) {
        if item_name.is_empty() { self.log.push("Use what?".into()); return; }
        let pos = self.player.inventory.iter()
            .position(|i| i.name.to_lowercase().contains(item_name));
        let Some(p) = pos else {
            self.log.push(format!("You don't have a '{item_name}'."));
            return;
        };
        let item = self.player.inventory.remove(p);
        match item.kind {
            ItemKind::Potion { heal } => {
                let before = self.player.health;
                self.player.health = (self.player.health + heal).min(self.player.max_health);
                self.log.push(format!(
                    "You drink the {}. Health {} → {}.",
                    item.name, before, self.player.health
                ));
            }
            ItemKind::Treasure { worth_gold } => {
                self.player.gold += worth_gold;
                self.log.push(format!(
                    "You pocket the {} (+{} gold, total {}).",
                    item.name, worth_gold, self.player.gold
                ));
            }
            ItemKind::Weapon { damage } => {
                // Using a weapon = equipping it.
                self.player.equipped_weapon_id = Some(item.id);
                self.player.equipped_damage = damage;
                self.log.push(format!("You wield the {} ({} damage).", item.name, damage));
                self.player.inventory.push(item);  // keep the weapon
            }
        }
    }

    fn do_equip(&mut self, item_name: &str) {
        if item_name.is_empty() { self.log.push("Equip what?".into()); return; }
        // Find in inventory.
        let pos = self.player.inventory.iter()
            .position(|i| i.name.to_lowercase().contains(item_name));
        let Some(p) = pos else {
            self.log.push(format!("You don't have a '{item_name}'."));
            return;
        };
        let item = &self.player.inventory[p];
        if let ItemKind::Weapon { damage } = item.kind {
            self.player.equipped_weapon_id = Some(item.id);
            self.player.equipped_damage = damage;
            self.log.push(format!("You wield the {} ({} damage).", item.name, damage));
        } else {
            self.log.push(format!("The {} is not a weapon.", item.name));
        }
    }

    fn do_fight(&mut self, enemy_name: &str) {
        let loc = self.player.location;
        let room = self.rooms.get_mut(&loc).unwrap();
        let pos = if enemy_name.is_empty() {
            (!room.enemies.is_empty()).then_some(0)
        } else {
            room.enemies.iter()
                .position(|e| e.name.to_lowercase().contains(enemy_name))
        };
        let Some(p) = pos else {
            self.log.push(format!("There's no '{enemy_name}' to fight here."));
            return;
        };
        let damage = self.player.equipped_damage;
        let enemy = &mut room.enemies[p];
        enemy.health -= damage;
        self.log.push(format!(
            "You hit the {} for {} damage ({}hp remaining).",
            enemy.name, damage, enemy.health.max(0)
        ));
        if enemy.health <= 0 {
            let defeated = room.enemies.remove(p);
            self.log.push(format!("The {} falls defeated.", defeated.name));
        }
    }

    fn do_inventory(&mut self) {
        if self.player.inventory.is_empty() {
            self.log.push("Your pack is empty.".into());
            return;
        }
        let names: Vec<&str> = self.player.inventory.iter().map(|i| i.name.as_str()).collect();
        self.log.push(format!("You carry: {}.", names.join(", ")));
    }

    fn do_stats(&mut self) {
        self.log.push(format!(
            "Turn {}. Health {}/{}. Gold {}. Damage {} (equipped: {}).",
            self.turn, self.player.health, self.player.max_health,
            self.player.gold, self.player.equipped_damage,
            self.player.equipped_weapon_id
                .and_then(|id| self.player.inventory.iter().find(|i| i.id == id))
                .map(|i| i.name.as_str()).unwrap_or("bare hands"),
        ));
    }

    fn enemies_strike(&mut self) {
        let loc = self.player.location;
        let damage_total: i32 = {
            let room = self.rooms.get(&loc).unwrap();
            room.enemies.iter().map(|e| e.damage).sum()
        };
        if damage_total > 0 {
            self.player.health -= damage_total;
            self.log.push(format!(
                "Enemies strike you for {damage_total} damage ({} hp)!",
                self.player.health.max(0)
            ));
        }
    }

    fn check_end_conditions(&mut self) {
        if self.player.health <= 0 {
            self.state = GameState::Dead;
            self.log.push("You fall, lifeless. The dungeon is silent.".into());
            return;
        }
        if self.player.gold >= self.win_condition_gold {
            self.state = GameState::Won;
            self.log.push(format!(
                "With {} gold in hand, you've proven your fortune. You win!",
                self.player.gold
            ));
        }
    }
}

fn verb_to_direction(verb: &str) -> &'static str {
    match verb {
        "north" | "n" => "north",
        "south" | "s" => "south",
        "east"  | "e" => "east",
        "west"  | "w" => "west",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_world_starts_in_entrance() {
        let w = World::demo();
        assert_eq!(w.current_room().id, 1);
        assert_eq!(w.state, GameState::Playing);
        assert_eq!(w.player.health, 20);
    }

    #[test]
    fn look_describes_room_with_items_and_exits() {
        let mut w = World::demo();
        let out = w.execute("look");
        let joined = out.join("\n");
        assert!(joined.contains("Entrance Hall"));
        assert!(joined.contains("rusty sword"));
        assert!(joined.contains("north"));
    }

    #[test]
    fn movement_between_rooms() {
        let mut w = World::demo();
        w.execute("go east");
        assert_eq!(w.current_room().id, 3);
        w.execute("go west");
        assert_eq!(w.current_room().id, 1);
    }

    #[test]
    fn short_direction_aliases_work() {
        let mut w = World::demo();
        w.execute("n");
        assert_eq!(w.current_room().id, 2);
    }

    #[test]
    fn take_moves_item_to_inventory() {
        let mut w = World::demo();
        w.execute("take sword");
        assert_eq!(w.player.inventory.len(), 1);
        assert!(w.current_room().items.iter().all(|i| !i.name.contains("sword")));
    }

    #[test]
    fn equipped_weapon_increases_damage() {
        let mut w = World::demo();
        assert_eq!(w.player.equipped_damage, 1);   // bare hands
        w.execute("take sword");
        w.execute("equip sword");
        assert_eq!(w.player.equipped_damage, 5);
    }

    #[test]
    fn fighting_and_enemy_retaliation() {
        let mut w = World::demo();
        w.execute("take sword");
        w.execute("equip sword");
        let health_before = w.player.health;
        w.execute("go north");   // enter cavern with goblin (1 strike)
        assert!(w.player.health < health_before, "goblin should hit on entry turn");
        // Keep attacking the goblin.
        for _ in 0..3 {
            w.execute("fight goblin");
            if w.current_room().enemies.is_empty() { break; }
        }
        assert!(w.current_room().enemies.is_empty(), "goblin should die in 3 hits at 5 damage");
    }

    #[test]
    fn potion_heals_up_to_max() {
        let mut w = World::demo();
        w.player.health = 5;    // pretend we're hurt
        w.execute("take potion");
        w.execute("use potion");
        assert_eq!(w.player.health, 20);  // heal 15, clamped at max 20
    }

    #[test]
    fn win_condition_requires_treasure_gold() {
        let mut w = World::demo();
        w.execute("go east");
        w.execute("take chalice");
        w.execute("use chalice");
        assert_eq!(w.state, GameState::Won);
        assert_eq!(w.player.gold, 50);
    }

    #[test]
    fn commands_on_ended_game_are_ignored() {
        let mut w = World::demo();
        w.state = GameState::Won;
        let out = w.execute("look");
        assert_eq!(out.len(), 1);
        assert!(out[0].contains("game is over"));
    }

    #[test]
    fn unknown_command_still_counts_as_input_but_no_turn_provoked() {
        let mut w = World::demo();
        let turns_before = w.turn;
        w.execute("frobnicate");
        assert_eq!(w.turn, turns_before);   // no turn consumed for unknown verb
    }
}
