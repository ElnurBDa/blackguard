//! Building and shuffling the 44-card Scoundrel dungeon deck.

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use super::card::{Card, Suit};

/// Total number of cards in the dungeon.
pub const DECK_SIZE: usize = 44;

/// Build the ordered 44-card deck:
///
/// * Spades and Clubs: ranks 2–14 (13 each → 26 monsters)
/// * Diamonds and Hearts: ranks 2–10 (9 each → the red faces and red aces are
///   removed)
pub fn build_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(DECK_SIZE);
    for suit in [Suit::Spades, Suit::Clubs] {
        for rank in 2..=14 {
            deck.push(Card::new(suit, rank));
        }
    }
    for suit in [Suit::Diamonds, Suit::Hearts] {
        for rank in 2..=10 {
            deck.push(Card::new(suit, rank));
        }
    }
    debug_assert_eq!(deck.len(), DECK_SIZE);
    deck
}

/// A deck shuffled deterministically from `seed`. The same seed always yields
/// the same order — this powers reproducible runs, the daily challenge, and
/// tests.
pub fn shuffled_deck(seed: u64) -> Vec<Card> {
    let mut deck = build_deck();
    let mut rng = StdRng::seed_from_u64(seed);
    deck.shuffle(&mut rng);
    deck
}

#[cfg(test)]
mod tests {
    use super::super::card::Kind;
    use super::*;

    #[test]
    fn deck_has_44_cards() {
        assert_eq!(build_deck().len(), 44);
    }

    #[test]
    fn composition_is_correct() {
        let deck = build_deck();
        let monsters = deck.iter().filter(|c| c.kind() == Kind::Monster).count();
        let weapons = deck.iter().filter(|c| c.kind() == Kind::Weapon).count();
        let potions = deck.iter().filter(|c| c.kind() == Kind::Potion).count();
        assert_eq!(monsters, 26);
        assert_eq!(weapons, 9);
        assert_eq!(potions, 9);
    }

    #[test]
    fn no_red_face_cards_or_red_aces() {
        let deck = build_deck();
        for c in &deck {
            if matches!(c.suit, Suit::Diamonds | Suit::Hearts) {
                assert!(
                    c.rank <= 10,
                    "red suit {:?} should not have rank {}",
                    c.suit,
                    c.rank
                );
            }
        }
    }

    #[test]
    fn no_duplicate_cards() {
        let deck = build_deck();
        let mut seen = std::collections::HashSet::new();
        for c in &deck {
            assert!(seen.insert((c.suit, c.rank)), "duplicate card {c:?}");
        }
    }

    #[test]
    fn same_seed_same_order() {
        assert_eq!(shuffled_deck(42), shuffled_deck(42));
    }

    #[test]
    fn different_seeds_usually_differ() {
        assert_ne!(shuffled_deck(1), shuffled_deck(2));
    }

    #[test]
    fn shuffle_preserves_the_multiset() {
        let mut a = build_deck();
        let mut b = shuffled_deck(7);
        a.sort_by_key(|c| (c.suit as u8, c.rank));
        b.sort_by_key(|c| (c.suit as u8, c.rank));
        assert_eq!(a, b);
    }
}
