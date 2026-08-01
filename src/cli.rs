//! Command-line arguments.

use clap::{Parser, ValueEnum};

use crate::core::WeaponRule;

#[derive(Parser, Debug)]
#[command(
    name = "blackguard",
    version,
    about = "Blackguard — a beautiful terminal implementation of the card game Scoundrel"
)]
pub struct Cli {
    /// Seed for a reproducible dungeon (default: random each run).
    #[arg(short, long)]
    pub seed: Option<u64>,

    /// Play today's daily challenge (same deck for everyone, all day).
    #[arg(short, long)]
    pub daily: bool,

    /// Weapon-binding rule variant.
    #[arg(long, value_enum, default_value_t = Rule::Strict)]
    pub rule: Rule,
}

impl Cli {
    /// Resolve the starting seed: daily challenge > explicit seed > random.
    pub fn resolve_seed(&self) -> u64 {
        if self.daily {
            daily_seed()
        } else {
            self.seed.unwrap_or_else(rand::random::<u64>)
        }
    }
}

/// A seed that is stable for the whole current UTC day.
fn daily_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Days since the epoch, offset into a distinctive part of the seed space.
    0xDA11_0000_0000_0000 ^ (secs / 86_400)
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Rule {
    /// A bound weapon may only slay monsters strictly weaker than its last kill.
    Strict,
    /// A bound weapon may slay monsters of equal-or-lower value.
    Equal,
}

impl From<Rule> for WeaponRule {
    fn from(r: Rule) -> Self {
        match r {
            Rule::Strict => WeaponRule::Strict,
            Rule::Equal => WeaponRule::Equal,
        }
    }
}
