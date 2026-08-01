//! The pure Scoundrel rules engine — no I/O, no TUI. Everything here is
//! deterministic given a seed and fully unit-tested, and could be reused by any
//! frontend (TUI, web/WASM, AI solver).

pub mod card;
pub mod deck;
pub mod game;
pub mod rules;

pub use card::{Card, Kind, Suit};
pub use game::{Action, GameEvent, GameState, MAX_HEALTH, MoveError, ROOM_SIZE, Status, Weapon};
pub use rules::WeaponRule;
