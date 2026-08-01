//! Application state and input handling — the "update" half of the loop.
//!
//! `App` wraps the pure [`GameState`] with UI concerns (which card is selected,
//! the message log, transient error flashes, help overlay) and translates key
//! presses into engine [`Action`]s.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::core::{Action, Card, GameEvent, GameState, Kind, MoveError, Status, WeaponRule};
use crate::effects::Fx;
use crate::stats::Stats;
use crate::ui::{help, theme};

const LOG_CAP: usize = 200;

pub struct App {
    pub game: GameState,
    pub selected: usize,
    pub log: Vec<Line<'static>>,
    pub flash: Option<String>,
    pub show_help: bool,
    pub help_page: usize,
    pub should_quit: bool,
    pub stats: Stats,
    pub fx: Fx,
    /// Guards against recording the same finished game more than once.
    recorded: bool,
}

impl App {
    pub fn new(seed: u64, rule: WeaponRule) -> Self {
        let mut app = Self {
            game: GameState::new(seed, rule),
            selected: 0,
            log: Vec::new(),
            flash: None,
            show_help: false,
            help_page: 0,
            should_quit: false,
            stats: Stats::load(),
            fx: Fx::default(),
            recorded: false,
        };
        app.announce();
        app
    }

    fn announce(&mut self) {
        self.push(dim(format!(
            "Dungeon opened — seed {}, {} weapons.",
            self.game.seed,
            self.game.rule.as_str()
        )));
    }

    // ---- input ----

    pub fn on_key(&mut self, code: ratatui::crossterm::event::KeyCode) {
        use ratatui::crossterm::event::KeyCode::*;

        // Help overlay captures navigation while open.
        if self.show_help {
            match code {
                Char('?') | Esc | Char('q') => self.show_help = false,
                Left | Char('h') => self.help_page = self.help_page.saturating_sub(1),
                Right | Char('l') | Char(' ') | Enter => {
                    self.help_page = (self.help_page + 1).min(help::PAGE_COUNT - 1)
                }
                _ => {}
            }
            return;
        }

        // Global keys.
        match code {
            Char('q') | Esc => {
                self.should_quit = true;
                return;
            }
            Char('?') => {
                self.show_help = true;
                self.help_page = 0;
                return;
            }
            Char('n') => {
                self.restart(rand::random::<u64>(), self.game.rule);
                return;
            }
            Char('r') => {
                self.restart(self.game.seed, self.game.rule);
                return;
            }
            Char('t') => {
                // Toggle the weapon rule and replay the same deck to compare.
                let rule = match self.game.rule {
                    WeaponRule::Strict => WeaponRule::Equal,
                    WeaponRule::Equal => WeaponRule::Strict,
                };
                self.restart(self.game.seed, rule);
                return;
            }
            _ => {}
        }

        if self.game.is_over() {
            return; // only n / r / q / ? matter on the game-over screen
        }

        match code {
            Left | Char('h') => self.move_selection(-1),
            Right | Char('l') => self.move_selection(1),
            Enter | Char(' ') => self.use_selected(),
            Char('f') => self.fight(true),
            Char('b') => self.fight(false),
            Char('a') => self.act(Action::Avoid),
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let n = self.game.room.len();
        if n == 0 {
            return;
        }
        let cur = self.selected as i32;
        self.selected = (cur + delta).rem_euclid(n as i32) as usize;
    }

    fn use_selected(&mut self) {
        let Some(card) = self.selected_card() else {
            return;
        };
        let i = self.selected;
        let action = match card.kind() {
            Kind::Potion => Action::Drink(i),
            Kind::Weapon => Action::Equip(i),
            Kind::Monster => Action::Fight {
                index: i,
                use_weapon: self.game.weapon_reaches(i),
            },
        };
        self.act(action);
    }

    fn fight(&mut self, use_weapon: bool) {
        match self.selected_card() {
            Some(c) if c.kind() == Kind::Monster => self.act(Action::Fight {
                index: self.selected,
                use_weapon,
            }),
            _ => self.flash = Some("Select a monster to fight.".into()),
        }
    }

    pub fn selected_card(&self) -> Option<Card> {
        self.game.room.get(self.selected).copied()
    }

    // ---- applying moves ----

    fn act(&mut self, action: Action) {
        match self.game.apply(action) {
            Ok(events) => {
                self.flash = None;
                for ev in events {
                    self.log_event(ev);
                    match ev {
                        GameEvent::Fought { damage, .. } if damage > 0 => self.fx.flash_damage(),
                        GameEvent::RoomRefilled => self.fx.reveal_room(),
                        _ => {}
                    }
                }
                self.clamp_selection();
                self.maybe_record();
            }
            Err(err) => self.flash = Some(explain(err)),
        }
    }

    /// Persist the result exactly once when a game finishes.
    fn maybe_record(&mut self) {
        if self.game.is_over() && !self.recorded {
            self.stats.record(&self.game);
            self.stats.save();
            self.recorded = true;
        }
    }

    fn restart(&mut self, seed: u64, rule: WeaponRule) {
        self.game = GameState::new(seed, rule);
        self.selected = 0;
        self.flash = None;
        self.recorded = false;
        self.fx.reveal_room();
        self.announce();
    }

    fn clamp_selection(&mut self) {
        let n = self.game.room.len();
        if n == 0 {
            self.selected = 0;
        } else if self.selected >= n {
            self.selected = n - 1;
        }
    }

    // ---- logging ----

    fn push(&mut self, line: Line<'static>) {
        self.log.push(line);
        if self.log.len() > LOG_CAP {
            self.log.drain(..self.log.len() - LOG_CAP);
        }
    }

    fn log_event(&mut self, ev: GameEvent) {
        let line = match ev {
            GameEvent::Fought {
                monster,
                with_weapon,
                damage,
            } => {
                let dmg = if with_weapon && damage == 0 {
                    "no damage".to_string()
                } else {
                    format!("-{damage} HP")
                };
                let how = if with_weapon {
                    "with your blade"
                } else {
                    "bare-handed"
                };
                colored(
                    format!("Slew {} {how} ({dmg})", label(monster)),
                    theme::threat_color(monster.value()),
                )
            }
            GameEvent::Equipped { weapon, discarded } => {
                let extra = match discarded {
                    Some(d) => format!(" (dropped {})", label(d)),
                    None => String::new(),
                };
                colored(format!("Equipped {}{extra}", label(weapon)), theme::GOLD)
            }
            GameEvent::Drank {
                potion,
                healed,
                wasted,
            } => {
                if wasted {
                    dim(format!(
                        "Wasted {} — already drank this room",
                        label(potion)
                    ))
                } else {
                    colored(
                        format!("Drank {} (+{healed} HP)", label(potion)),
                        theme::GOOD,
                    )
                }
            }
            GameEvent::Avoided => dim("Avoided the room".to_string()),
            GameEvent::RoomRefilled => dim("— entered a new room —".to_string()),
            GameEvent::Won => colored("You cleared the dungeon!".to_string(), theme::GOOD),
            GameEvent::Lost => colored("You fell in the dungeon.".to_string(), theme::BAD),
        };
        self.push(line);
    }
}

fn label(card: Card) -> String {
    format!("{}{}", card.rank_label(), card.suit.glyph())
}

fn colored(text: String, color: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(color)))
}

fn dim(text: String) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default()
            .fg(theme::DIM)
            .add_modifier(Modifier::ITALIC),
    ))
}

fn explain(err: MoveError) -> String {
    match err {
        MoveError::WeaponCantReach => {
            "Your weapon is bound — it can't reach a monster that strong.".into()
        }
        MoveError::CantAvoid => "You can't avoid now (already played or just avoided).".into(),
        MoveError::NoWeapon => "You have no weapon equipped.".into(),
        MoveError::WrongKind => "You can't do that with this card.".into(),
        MoveError::OutOfBounds => "No card there.".into(),
        MoveError::GameOver => "The game is over — press n for a new dungeon.".into(),
    }
}

/// Convenience for the renderer: is the game finished, and how?
pub fn outcome(status: Status) -> Option<&'static str> {
    match status {
        Status::Won => Some("VICTORY"),
        Status::Lost => Some("YOU DIED"),
        Status::Playing => None,
    }
}
