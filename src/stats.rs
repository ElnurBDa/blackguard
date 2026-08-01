//! Persistent run statistics and high scores.
//!
//! Stored as JSON in the platform data directory (via `directories`), so it
//! works the same on Linux, macOS, and Windows.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::{GameState, Status};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub games_played: u32,
    pub wins: u32,
    pub best_score: i32,
    pub worst_score: i32,
    pub total_monsters_slain: u64,
    /// Whether `best_score`/`worst_score` have been seeded yet (so a first loss
    /// of e.g. -30 doesn't lose out to the `0` default).
    pub has_record: bool,
}

impl Stats {
    /// Load stats from disk, or defaults if absent/unreadable.
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Best-effort save (failures are silently ignored — stats are non-critical).
    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, json);
        }
    }

    /// Fold a finished game into the running totals.
    pub fn record(&mut self, game: &GameState) {
        let score = game.score();
        self.games_played += 1;
        if game.status == Status::Won {
            self.wins += 1;
        }
        self.total_monsters_slain += game.monsters_slain as u64;
        if self.has_record {
            self.best_score = self.best_score.max(score);
            self.worst_score = self.worst_score.min(score);
        } else {
            self.best_score = score;
            self.worst_score = score;
            self.has_record = true;
        }
    }
}

fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from("net", "Blackguard", "blackguard")
        .map(|d| d.data_dir().join("stats.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WeaponRule;

    #[test]
    fn record_tracks_best_and_worst() {
        let mut s = Stats::default();
        let mut win = GameState::new(1, WeaponRule::Strict);
        win.status = Status::Won;
        win.health = 12;
        s.record(&win); // score 12
        assert_eq!(s.best_score, 12);
        assert_eq!(s.worst_score, 12);
        assert_eq!(s.wins, 1);

        let mut loss = GameState::new(1, WeaponRule::Strict);
        loss.status = Status::Lost;
        loss.health = -5;
        // no monsters remaining tallied precisely here; just check ordering
        let score = loss.score();
        s.record(&loss);
        assert_eq!(s.best_score, 12);
        assert_eq!(s.worst_score, score.min(12));
        assert_eq!(s.games_played, 2);
    }
}
