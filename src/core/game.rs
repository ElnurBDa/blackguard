//! The Scoundrel rules engine: pure game state plus a single `apply` reducer.
//!
//! This module has **no I/O and no TUI dependency**. Every state transition
//! goes through [`GameState::apply`], which validates the move, mutates state,
//! and returns a list of [`GameEvent`]s describing what happened (for the log
//! and animations) — or a [`MoveError`] if the move was illegal.

use serde::{Deserialize, Serialize};

use super::card::{Card, Kind};
use super::deck::shuffled_deck;
use super::rules::WeaponRule;

/// Starting and maximum health.
pub const MAX_HEALTH: i32 = 20;
/// Cards face-up in a full room.
pub const ROOM_SIZE: usize = 4;

/// An equipped weapon and the monsters it has slain (its "binding" stack).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub card: Card,
    /// Monsters slain with this weapon, in order. The most recent one sets the
    /// binding ceiling.
    pub slain: Vec<Card>,
}

impl Weapon {
    fn new(card: Card) -> Self {
        Self {
            card,
            slain: Vec::new(),
        }
    }

    /// The binding ceiling: the value of the last monster slain, or `None` if
    /// the weapon is fresh (can hit anything).
    pub fn bound(&self) -> Option<u8> {
        self.slain.last().map(|c| c.value())
    }

    /// Whether this weapon may legally be used against `monster` under `rule`.
    pub fn can_reach(&self, monster: u8, rule: WeaponRule) -> bool {
        match self.bound() {
            None => true,
            Some(b) => rule.reaches(monster, b),
        }
    }
}

/// Overall game status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Playing,
    Won,
    Lost,
}

/// A move the player can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Fight the monster at `index`. `use_weapon` chooses the equipped weapon
    /// (if any and if it reaches) over bare hands.
    Fight { index: usize, use_weapon: bool },
    /// Equip the weapon (diamond) at `index`.
    Equip(usize),
    /// Drink the potion (heart) at `index`.
    Drink(usize),
    /// Avoid the whole room: put all four cards on the bottom of the deck and
    /// deal a fresh four.
    Avoid,
}

/// Why a move was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    GameOver,
    OutOfBounds,
    /// The card at that index is the wrong kind for the action.
    WrongKind,
    /// Tried to fight with a weapon but none is equipped.
    NoWeapon,
    /// The equipped weapon is bound and cannot reach that monster.
    WeaponCantReach,
    /// Avoiding is not allowed right now (already avoided the previous room, or
    /// the room has been partially played, or it is not a full room).
    CantAvoid,
}

/// Something that happened as a result of a move — consumed by the UI for the
/// message log and animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    Fought {
        monster: Card,
        with_weapon: bool,
        damage: u8,
    },
    Equipped {
        weapon: Card,
        discarded: Option<Card>,
    },
    Drank {
        potion: Card,
        healed: u8,
        wasted: bool,
    },
    Avoided,
    /// A new room was dealt after the previous one was played down to one card.
    RoomRefilled,
    Won,
    Lost,
}

/// The full game state. Serializable for save/resume and stats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub seed: u64,
    pub rule: WeaponRule,
    pub health: i32,
    /// Draw pile. The **top** of the deck is the last element (`pop`); the
    /// **bottom** is the front.
    pub deck: Vec<Card>,
    /// Face-up room, up to `ROOM_SIZE` cards.
    pub room: Vec<Card>,
    pub discard: Vec<Card>,
    pub weapon: Option<Weapon>,
    pub status: Status,

    /// Whether a potion has already been drunk in the current room cycle.
    potion_used: bool,
    /// Whether the immediately preceding room was avoided (blocks a second
    /// avoid in a row).
    avoided_previous: bool,
    /// Cards resolved since this room cycle began (avoid is only legal at 0).
    resolved_this_cycle: u8,
    /// The last card resolved — used for the "won at full health on a potion"
    /// scoring bonus.
    last_resolved: Option<Card>,

    // ---- run statistics ----
    pub rooms_cleared: u32,
    pub monsters_slain: u32,
    pub turns: u32,
}

impl GameState {
    /// Start a new game from an explicit seed (reproducible).
    pub fn new(seed: u64, rule: WeaponRule) -> Self {
        let mut state = Self {
            seed,
            rule,
            health: MAX_HEALTH,
            deck: shuffled_deck(seed),
            room: Vec::with_capacity(ROOM_SIZE),
            discard: Vec::new(),
            weapon: None,
            status: Status::Playing,
            potion_used: false,
            avoided_previous: false,
            resolved_this_cycle: 0,
            last_resolved: None,
            rooms_cleared: 0,
            monsters_slain: 0,
            turns: 0,
        };
        state.draw_to_fill();
        state
    }

    // ---- queries used by the UI ----

    pub fn is_over(&self) -> bool {
        self.status != Status::Playing
    }

    pub fn potion_used(&self) -> bool {
        self.potion_used
    }

    /// Whether the player may avoid the current room right now.
    pub fn can_avoid(&self) -> bool {
        self.status == Status::Playing
            && !self.avoided_previous
            && self.resolved_this_cycle == 0
            && self.room.len() == ROOM_SIZE
    }

    /// Whether the equipped weapon can legally fight the card at `index`.
    pub fn weapon_reaches(&self, index: usize) -> bool {
        match (self.room.get(index), &self.weapon) {
            (Some(card), Some(w)) if card.is_monster() => w.can_reach(card.value(), self.rule),
            _ => false,
        }
    }

    /// Final score. Positive on a win (remaining health, with a potion bonus at
    /// full health); negative on a loss (health minus the value of every
    /// monster still lurking in the dungeon).
    pub fn score(&self) -> i32 {
        match self.status {
            Status::Won => {
                let base = self.health;
                if base == MAX_HEALTH {
                    if let Some(c) = self.last_resolved {
                        if c.kind() == Kind::Potion {
                            return base + c.value() as i32;
                        }
                    }
                }
                base
            }
            Status::Lost => {
                let remaining: i32 = self
                    .deck
                    .iter()
                    .chain(self.room.iter())
                    .filter(|c| c.is_monster())
                    .map(|c| c.value() as i32)
                    .sum();
                self.health - remaining
            }
            Status::Playing => self.health,
        }
    }

    // ---- the reducer ----

    /// Apply a move. On success, returns the events it produced; on failure,
    /// the state is left untouched and a [`MoveError`] is returned.
    pub fn apply(&mut self, action: Action) -> Result<Vec<GameEvent>, MoveError> {
        if self.status != Status::Playing {
            return Err(MoveError::GameOver);
        }
        match action {
            Action::Fight { index, use_weapon } => self.fight(index, use_weapon),
            Action::Equip(index) => self.equip(index),
            Action::Drink(index) => self.drink(index),
            Action::Avoid => self.avoid(),
        }
    }

    fn fight(&mut self, index: usize, use_weapon: bool) -> Result<Vec<GameEvent>, MoveError> {
        let card = *self.room.get(index).ok_or(MoveError::OutOfBounds)?;
        if !card.is_monster() {
            return Err(MoveError::WrongKind);
        }
        let mv = card.value();
        let mut events = Vec::new();

        let damage = if use_weapon {
            let w = self.weapon.as_ref().ok_or(MoveError::NoWeapon)?;
            if !w.can_reach(mv, self.rule) {
                return Err(MoveError::WeaponCantReach);
            }
            let dmg = mv.saturating_sub(w.card.value());
            // The slain monster is tucked under the weapon, setting its ceiling.
            self.weapon.as_mut().unwrap().slain.push(card);
            dmg
        } else {
            self.discard.push(card);
            mv
        };

        self.room.remove(index);
        self.health -= damage as i32;
        self.monsters_slain += 1;
        self.last_resolved = Some(card);
        events.push(GameEvent::Fought {
            monster: card,
            with_weapon: use_weapon,
            damage,
        });
        self.after_resolve(&mut events);
        Ok(events)
    }

    fn equip(&mut self, index: usize) -> Result<Vec<GameEvent>, MoveError> {
        let card = *self.room.get(index).ok_or(MoveError::OutOfBounds)?;
        if !card.is_weapon() {
            return Err(MoveError::WrongKind);
        }
        let mut events = Vec::new();

        let discarded = self.weapon.take().map(|old| {
            let c = old.card;
            self.discard.push(old.card);
            self.discard.extend(old.slain);
            c
        });
        self.weapon = Some(Weapon::new(card));
        self.room.remove(index);
        self.last_resolved = Some(card);
        events.push(GameEvent::Equipped {
            weapon: card,
            discarded,
        });
        self.after_resolve(&mut events);
        Ok(events)
    }

    fn drink(&mut self, index: usize) -> Result<Vec<GameEvent>, MoveError> {
        let card = *self.room.get(index).ok_or(MoveError::OutOfBounds)?;
        if !card.is_potion() {
            return Err(MoveError::WrongKind);
        }
        let mut events = Vec::new();

        let (healed, wasted) = if self.potion_used {
            (0, true)
        } else {
            let before = self.health;
            self.health = (self.health + card.value() as i32).min(MAX_HEALTH);
            self.potion_used = true;
            ((self.health - before) as u8, false)
        };
        self.discard.push(card);
        self.room.remove(index);
        self.last_resolved = Some(card);
        events.push(GameEvent::Drank {
            potion: card,
            healed,
            wasted,
        });
        self.after_resolve(&mut events);
        Ok(events)
    }

    fn avoid(&mut self) -> Result<Vec<GameEvent>, MoveError> {
        if !self.can_avoid() {
            return Err(MoveError::CantAvoid);
        }
        // Move all four cards to the bottom (front) of the deck, preserving
        // their order, then deal a fresh room.
        let room = std::mem::take(&mut self.room);
        for card in room.into_iter().rev() {
            self.deck.insert(0, card);
        }
        self.avoided_previous = true;
        self.potion_used = false;
        self.resolved_this_cycle = 0;
        self.turns += 1;
        self.draw_to_fill();

        let mut events = vec![GameEvent::Avoided];
        // Avoiding cannot end the game, but keep the invariant centralized.
        self.check_win(&mut events);
        Ok(events)
    }

    /// Shared bookkeeping after a card is resolved (fight/equip/drink).
    fn after_resolve(&mut self, events: &mut Vec<GameEvent>) {
        self.turns += 1;
        self.resolved_this_cycle += 1;

        if self.health <= 0 {
            self.status = Status::Lost;
            events.push(GameEvent::Lost);
            return;
        }

        // Once a room is played down to a single card, carry it over and deal
        // three fresh cards — a new room cycle.
        if self.room.len() == 1 && !self.deck.is_empty() {
            self.draw_to_fill();
            self.rooms_cleared += 1;
            self.potion_used = false;
            self.resolved_this_cycle = 0;
            self.avoided_previous = false;
            events.push(GameEvent::RoomRefilled);
        }

        self.check_win(events);
    }

    fn check_win(&mut self, events: &mut Vec<GameEvent>) {
        if self.status == Status::Playing && self.deck.is_empty() && self.room.is_empty() {
            self.status = Status::Won;
            events.push(GameEvent::Won);
        }
    }

    /// Draw from the top of the deck until the room holds `ROOM_SIZE` cards or
    /// the deck is empty.
    fn draw_to_fill(&mut self) {
        while self.room.len() < ROOM_SIZE {
            match self.deck.pop() {
                Some(card) => self.room.push(card),
                None => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::card::{Card, Suit};

    /// Build a deterministic state with a hand-picked deck and room, so tests
    /// don't depend on shuffle order. `deck` is bottom→top; `room` is dealt.
    fn rigged(room: Vec<Card>, deck: Vec<Card>) -> GameState {
        GameState {
            seed: 0,
            rule: WeaponRule::Strict,
            health: MAX_HEALTH,
            deck,
            room,
            discard: Vec::new(),
            weapon: None,
            status: Status::Playing,
            potion_used: false,
            avoided_previous: false,
            resolved_this_cycle: 0,
            last_resolved: None,
            rooms_cleared: 0,
            monsters_slain: 0,
            turns: 0,
        }
    }

    fn mon(v: u8) -> Card {
        Card::new(Suit::Spades, v)
    }
    fn wpn(v: u8) -> Card {
        Card::new(Suit::Diamonds, v)
    }
    fn pot(v: u8) -> Card {
        Card::new(Suit::Hearts, v)
    }

    #[test]
    fn new_game_deals_four_and_full_health() {
        let g = GameState::new(1, WeaponRule::Strict);
        assert_eq!(g.room.len(), 4);
        assert_eq!(g.deck.len(), 40);
        assert_eq!(g.health, 20);
        assert_eq!(g.status, Status::Playing);
    }

    #[test]
    fn barehanded_takes_full_damage() {
        let mut g = rigged(vec![mon(9), pot(5), mon(3), wpn(4)], vec![]);
        g.apply(Action::Fight {
            index: 0,
            use_weapon: false,
        })
        .unwrap();
        assert_eq!(g.health, 11);
        assert_eq!(g.discard, vec![mon(9)]);
    }

    #[test]
    fn weapon_reduces_damage_and_binds() {
        let mut g = rigged(vec![mon(9), pot(5), mon(3), wpn(4)], vec![]);
        g.apply(Action::Equip(3)).unwrap();
        g.apply(Action::Fight {
            index: 0,
            use_weapon: true,
        })
        .unwrap();
        assert_eq!(g.health, 20 - (9 - 4));
        let w = g.weapon.as_ref().unwrap();
        assert_eq!(w.bound(), Some(9));
    }

    #[test]
    fn weapon_damage_floors_at_zero() {
        let mut g = rigged(vec![wpn(10), mon(3), pot(2), mon(2)], vec![]);
        g.apply(Action::Equip(0)).unwrap();
        g.apply(Action::Fight {
            index: 0,
            use_weapon: true,
        })
        .unwrap();
        assert_eq!(g.health, 20); // 3 - 10 -> 0 damage
    }

    #[test]
    fn strict_rule_blocks_equal_value_monster() {
        let mut g = rigged(vec![wpn(6), mon(8), mon(8), pot(2)], vec![]);
        g.rule = WeaponRule::Strict;
        g.apply(Action::Equip(0)).unwrap();
        g.apply(Action::Fight {
            index: 0,
            use_weapon: true,
        })
        .unwrap(); // slay first 8, bound = 8
        let err = g.apply(Action::Fight {
            index: 0,
            use_weapon: true,
        });
        assert_eq!(err, Err(MoveError::WeaponCantReach));
    }

    #[test]
    fn equal_rule_allows_equal_value_monster() {
        let mut g = rigged(vec![wpn(6), mon(8), mon(8), pot(2)], vec![]);
        g.rule = WeaponRule::Equal;
        g.apply(Action::Equip(0)).unwrap();
        g.apply(Action::Fight {
            index: 0,
            use_weapon: true,
        })
        .unwrap();
        assert!(
            g.apply(Action::Fight {
                index: 0,
                use_weapon: true
            })
            .is_ok()
        );
    }

    #[test]
    fn equipping_replaces_and_discards_old_weapon_with_its_stack() {
        let mut g = rigged(vec![wpn(5), mon(4), wpn(8), pot(2)], vec![]);
        g.apply(Action::Equip(0)).unwrap(); // equip 5
        g.apply(Action::Fight {
            index: 0,
            use_weapon: true,
        })
        .unwrap(); // slay 4 with it; room is now [wpn8, pot2]
        // Now equip the 8; old weapon (5) and its slain (4) go to discard.
        g.apply(Action::Equip(0)).unwrap();
        let w = g.weapon.as_ref().unwrap();
        assert_eq!(w.card, wpn(8));
        assert!(w.slain.is_empty());
        assert!(g.discard.contains(&wpn(5)));
        assert!(g.discard.contains(&mon(4)));
    }

    #[test]
    fn potion_heals_but_only_first_per_room() {
        let mut g = rigged(vec![pot(5), pot(7), mon(3), mon(2)], vec![]);
        g.health = 10;
        g.apply(Action::Drink(0)).unwrap();
        assert_eq!(g.health, 15);
        let ev = g.apply(Action::Drink(0)).unwrap(); // second potion this room
        assert_eq!(g.health, 15); // no effect
        assert!(matches!(ev[0], GameEvent::Drank { wasted: true, .. }));
    }

    #[test]
    fn potion_cannot_exceed_max_health() {
        let mut g = rigged(vec![pot(10), mon(3), mon(2), mon(2)], vec![]);
        g.health = 18;
        g.apply(Action::Drink(0)).unwrap();
        assert_eq!(g.health, 20);
    }

    #[test]
    fn cannot_avoid_twice_in_a_row() {
        // Enough deck so avoid can deal fresh rooms.
        let deck: Vec<Card> = (2..=10).map(mon).chain((2..=6).map(mon)).collect();
        let mut g = rigged(vec![mon(14), mon(13), mon(12), mon(11)], deck);
        assert!(g.can_avoid());
        g.apply(Action::Avoid).unwrap();
        assert!(!g.can_avoid());
        assert_eq!(g.apply(Action::Avoid), Err(MoveError::CantAvoid));
    }

    #[test]
    fn cannot_avoid_after_playing_a_card() {
        let mut g = rigged(vec![pot(5), mon(3), mon(2), mon(2)], vec![mon(4), mon(5)]);
        g.apply(Action::Drink(0)).unwrap();
        assert!(!g.can_avoid());
    }

    #[test]
    fn avoid_moves_room_to_bottom_and_deals_fresh() {
        let deck = vec![mon(2), mon(3), mon(4), mon(5)]; // bottom..top
        let mut g = rigged(vec![pot(6), pot(7), pot(8), pot(9)], deck);
        g.apply(Action::Avoid).unwrap();
        // Fresh room is the top four of the deck (5,4,3,2).
        assert_eq!(g.room, vec![mon(5), mon(4), mon(3), mon(2)]);
        // The avoided potions now sit at the bottom.
        assert_eq!(&g.deck, &vec![pot(6), pot(7), pot(8), pot(9)]);
    }

    #[test]
    fn room_refills_after_three_resolved() {
        let mut g = rigged(
            vec![mon(2), mon(2), mon(2), mon(2)],
            vec![pot(9), pot(8), pot(7)],
        );
        for _ in 0..3 {
            g.apply(Action::Fight {
                index: 0,
                use_weapon: false,
            })
            .unwrap();
        }
        // One carried over + three drawn = full room again.
        assert_eq!(g.room.len(), 4);
        assert_eq!(g.rooms_cleared, 1);
    }

    #[test]
    fn death_sets_lost_and_negative_score() {
        let mut g = rigged(
            vec![mon(14), mon(13), mon(12), mon(11)],
            vec![mon(10), mon(9)],
        );
        g.health = 5;
        let ev = g
            .apply(Action::Fight {
                index: 0,
                use_weapon: false,
            })
            .unwrap();
        assert!(ev.contains(&GameEvent::Lost));
        assert_eq!(g.status, Status::Lost);
        // health 5 - 14 = -9; remaining monsters 13+12+11+10+9 = 55; score = -9 - 55
        assert_eq!(g.score(), -9 - 55);
    }

    #[test]
    fn clearing_the_dungeon_wins_with_health_as_score() {
        // Small dungeon: 4 harmless potions, empty deck.
        let mut g = rigged(vec![pot(2), pot(2), pot(2), pot(2)], vec![]);
        g.health = 12;
        for _ in 0..4 {
            g.apply(Action::Drink(0)).unwrap();
        }
        assert_eq!(g.status, Status::Won);
        assert_eq!(g.score(), g.health);
    }

    #[test]
    fn full_health_potion_finish_grants_bonus() {
        // Clear the dungeon while at max health with the last card a potion:
        // score is 20 + that potion's value (the only way to exceed 20).
        let mut g = rigged(vec![pot(2), pot(2), pot(2), pot(5)], vec![]);
        g.health = 20;
        for _ in 0..4 {
            g.apply(Action::Drink(0)).unwrap();
        }
        assert_eq!(g.status, Status::Won);
        assert_eq!(g.score(), 25);
    }
}
