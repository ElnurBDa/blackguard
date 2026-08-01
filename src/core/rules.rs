//! Configurable rule variants.
//!
//! The original 2011 rules are self-contradictory about weapon binding: the
//! text says a weapon may afterwards only slay monsters of "a lower value
//! (less than equal)" than the last one it slew. "Lower value" implies
//! *strictly* less; the parenthetical implies *less-than-or-equal*. Different
//! implementations pick different readings, so we make it a toggle.

use serde::{Deserialize, Serialize};

/// How a bound weapon's reach is compared against a monster's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeaponRule {
    /// A bound weapon may only be used on monsters *strictly weaker* than the
    /// last monster it slew (kill an 8 → next must be ≤ 7). The community
    /// standard; our default.
    #[default]
    Strict,
    /// A bound weapon may be used on monsters of *equal or lower* value than
    /// the last monster it slew (kill an 8 → next may be an 8).
    Equal,
}

impl WeaponRule {
    /// Can a weapon whose binding ceiling is `bound` (the value of the last
    /// monster it slew) be used against a monster of value `monster`?
    pub fn reaches(self, monster: u8, bound: u8) -> bool {
        match self {
            WeaponRule::Strict => monster < bound,
            WeaponRule::Equal => monster <= bound,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            WeaponRule::Strict => "strict",
            WeaponRule::Equal => "equal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_excludes_equal_value() {
        assert!(WeaponRule::Strict.reaches(7, 8));
        assert!(!WeaponRule::Strict.reaches(8, 8));
        assert!(!WeaponRule::Strict.reaches(9, 8));
    }

    #[test]
    fn equal_allows_equal_value() {
        assert!(WeaponRule::Equal.reaches(7, 8));
        assert!(WeaponRule::Equal.reaches(8, 8));
        assert!(!WeaponRule::Equal.reaches(9, 8));
    }

    #[test]
    fn default_is_strict() {
        assert_eq!(WeaponRule::default(), WeaponRule::Strict);
    }
}
