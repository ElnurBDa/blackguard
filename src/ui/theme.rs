//! Truecolor palette and per-card colouring.

use ratatui::style::Color;

use crate::core::Suit;

// Base palette (24-bit; degrades gracefully on 256-colour terminals).
pub const BG: Color = Color::Rgb(18, 18, 24);
pub const PANEL: Color = Color::Rgb(30, 31, 42);
pub const FG: Color = Color::Rgb(222, 225, 232);
pub const DIM: Color = Color::Rgb(110, 114, 128);
pub const ACCENT: Color = Color::Rgb(122, 162, 247); // selection / focus

pub const GOLD: Color = Color::Rgb(214, 164, 76); // diamonds → weapons
pub const POTION: Color = Color::Rgb(224, 108, 117); // hearts → potions
pub const MONSTER: Color = Color::Rgb(205, 209, 218); // spades / clubs face

pub const GOOD: Color = Color::Rgb(152, 195, 121); // low threat / healthy
pub const WARN: Color = Color::Rgb(229, 192, 123); // medium
pub const BAD: Color = Color::Rgb(224, 108, 117); // high threat / hurt

/// The pip colour for a card, by suit.
pub fn suit_color(suit: Suit) -> Color {
    match suit {
        Suit::Hearts => POTION,
        Suit::Diamonds => GOLD,
        Suit::Spades | Suit::Clubs => MONSTER,
    }
}

/// Colour a monster (or any value 2–14) by how dangerous it is.
pub fn threat_color(value: u8) -> Color {
    match value {
        0..=6 => GOOD,
        7..=10 => WARN,
        _ => BAD,
    }
}

/// Colour the health readout by remaining fraction of 20.
pub fn health_color(health: i32) -> Color {
    match health {
        h if h >= 14 => GOOD,
        h if h >= 7 => WARN,
        _ => BAD,
    }
}
