//! Cards, suits, and their in-game roles.
//!
//! Scoundrel uses a 44-card deck (a standard deck with the red face cards and
//! red aces removed). A card's *suit* determines its role:
//!
//! * ♠ Spades / ♣ Clubs  → monsters (values 2–14)
//! * ♦ Diamonds          → weapons (values 2–10)
//! * ♥ Hearts            → health potions (values 2–10)

use serde::{Deserialize, Serialize};

/// The four suits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

/// What a card *does* in the dungeon, derived purely from its suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Monster,
    Weapon,
    Potion,
}

impl Suit {
    /// The role this suit plays in the game.
    pub fn kind(self) -> Kind {
        match self {
            Suit::Spades | Suit::Clubs => Kind::Monster,
            Suit::Diamonds => Kind::Weapon,
            Suit::Hearts => Kind::Potion,
        }
    }

    /// The Unicode pip glyph for this suit.
    pub fn glyph(self) -> char {
        match self {
            Suit::Spades => '♠',
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
        }
    }
}

/// A single playing card. `rank` is the game value: 2–10 face value,
/// J=11, Q=12, K=13, A=14.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: u8,
}

impl Card {
    pub fn new(suit: Suit, rank: u8) -> Self {
        debug_assert!((2..=14).contains(&rank), "rank out of range: {rank}");
        Self { suit, rank }
    }

    /// The card's combat/heal/weapon value.
    pub fn value(self) -> u8 {
        self.rank
    }

    pub fn kind(self) -> Kind {
        self.suit.kind()
    }

    pub fn is_monster(self) -> bool {
        self.kind() == Kind::Monster
    }

    pub fn is_weapon(self) -> bool {
        self.kind() == Kind::Weapon
    }

    pub fn is_potion(self) -> bool {
        self.kind() == Kind::Potion
    }

    /// Short rank label as shown on the card face ("2".."10", "J", "Q", "K", "A").
    pub fn rank_label(self) -> &'static str {
        match self.rank {
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            11 => "J",
            12 => "Q",
            13 => "K",
            14 => "A",
            _ => "?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_follow_suit() {
        assert_eq!(Card::new(Suit::Spades, 5).kind(), Kind::Monster);
        assert_eq!(Card::new(Suit::Clubs, 14).kind(), Kind::Monster);
        assert_eq!(Card::new(Suit::Diamonds, 7).kind(), Kind::Weapon);
        assert_eq!(Card::new(Suit::Hearts, 2).kind(), Kind::Potion);
    }

    #[test]
    fn rank_labels() {
        assert_eq!(Card::new(Suit::Spades, 10).rank_label(), "10");
        assert_eq!(Card::new(Suit::Spades, 11).rank_label(), "J");
        assert_eq!(Card::new(Suit::Spades, 14).rank_label(), "A");
    }

    #[test]
    fn glyphs() {
        assert_eq!(Suit::Hearts.glyph(), '♥');
        assert_eq!(Suit::Spades.glyph(), '♠');
        assert_eq!(Suit::Diamonds.glyph(), '♦');
        assert_eq!(Suit::Clubs.glyph(), '♣');
    }
}
