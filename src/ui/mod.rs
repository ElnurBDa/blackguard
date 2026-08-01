//! Rendering — the "view" half of the loop.

pub mod card;
pub mod help;
pub mod theme;

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use tui_big_text::{BigText, PixelSize};

use crate::app::{App, outcome};
use crate::core::{Card, Kind, MAX_HEALTH, ROOM_SIZE, Status, Weapon, WeaponRule};

pub fn draw(f: &mut Frame, app: &mut App, elapsed: Duration) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(theme::BG).fg(theme::FG)),
        area,
    );

    let rows = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Length(1), // health
        Constraint::Min(9),    // room + sidebar
        Constraint::Length(8), // chronicle
        Constraint::Length(1), // flash
        Constraint::Length(1), // footer
    ])
    .split(area);

    draw_title(f, rows[0], app);
    draw_health(f, rows[1], app);

    let mid = Layout::horizontal([Constraint::Min(20), Constraint::Length(28)]).split(rows[2]);
    draw_room(f, mid[0], app);
    draw_sidebar(f, mid[1], app);

    draw_log(f, rows[3], app);
    draw_flash(f, rows[4], app);
    draw_footer(f, rows[5]);

    // Animations post-process the base layer (over the HP line and room),
    // before any modal overlay is drawn on top.
    app.fx.process(f.buffer_mut(), rows[1], mid[0], elapsed);

    if let Some(text) = outcome(app.game.status) {
        game_over(f, area, app, text);
    }
    if app.show_help {
        help::render(f, area, app.help_page);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::horizontal([Constraint::Min(10), Constraint::Length(18)]).split(area);
    let title = Line::from(vec![
        Span::styled("♠♥ ", Style::default().fg(theme::POTION)),
        Span::styled(
            "SCOUNDREL",
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ♦♣", Style::default().fg(theme::GOLD)),
    ]);
    f.render_widget(Paragraph::new(title), cols[0]);
    let mode = Line::from(Span::styled(
        format!("{} rules ", app.game.rule.as_str()),
        Style::default().fg(theme::DIM),
    ))
    .alignment(Alignment::Right);
    f.render_widget(Paragraph::new(mode), cols[1]);
}

fn draw_health(f: &mut Frame, area: Rect, app: &App) {
    let hp = app.game.health.clamp(0, MAX_HEALTH);
    let color = theme::health_color(hp);
    let filled = hp as usize;
    let empty = (MAX_HEALTH as usize).saturating_sub(filled);

    let mut spans = vec![Span::styled(" HP ", Style::default().fg(theme::DIM))];
    spans.push(Span::styled("♥".repeat(filled), Style::default().fg(color)));
    spans.push(Span::styled(
        "·".repeat(empty),
        Style::default().fg(theme::DIM),
    ));
    spans.push(Span::styled(
        format!("  {hp}/{MAX_HEALTH}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_room(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM))
        .title(Span::styled(" Room ", Style::default().fg(theme::FG)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.game.room.is_empty() {
        return;
    }

    let cells = Layout::horizontal(
        (0..ROOM_SIZE)
            .map(|_| Constraint::Ratio(1, ROOM_SIZE as u32))
            .collect::<Vec<_>>(),
    )
    .split(inner);

    for (i, card) in app.game.room.iter().enumerate() {
        let selected = i == app.selected;
        let caption = card_caption(
            *card,
            &app.game.weapon,
            app.game.rule,
            app.game.potion_used(),
        );
        card::render_card(f, cells[i], *card, selected, caption);
    }
}

/// The little annotation printed beneath a room card.
fn card_caption(
    card: Card,
    weapon: &Option<Weapon>,
    rule: WeaponRule,
    potion_used: bool,
) -> Line<'static> {
    match card.kind() {
        Kind::Monster => {
            let val = card.value();
            let reaches = weapon
                .as_ref()
                .map(|w| w.can_reach(val, rule))
                .unwrap_or(false);
            if reaches {
                let wdmg = val.saturating_sub(weapon.as_ref().unwrap().card.value());
                // Compact: gold ♦ = blade damage, threat-coloured = bare-handed.
                Line::from(vec![
                    Span::styled(
                        format!("♦-{wdmg}"),
                        Style::default()
                            .fg(theme::GOLD)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  -", Style::default().fg(theme::DIM)),
                    Span::styled(
                        val.to_string(),
                        Style::default().fg(theme::threat_color(val)),
                    ),
                ])
            } else {
                Line::from(Span::styled(
                    format!("take -{val}"),
                    Style::default().fg(theme::threat_color(val)),
                ))
            }
        }
        Kind::Weapon => Line::from(Span::styled(
            format!("weapon {}", card.value()),
            Style::default().fg(theme::GOLD),
        )),
        Kind::Potion => {
            if potion_used {
                Line::from(Span::styled("no effect", Style::default().fg(theme::DIM)))
            } else {
                Line::from(Span::styled(
                    format!("heal +{}", card.value()),
                    Style::default().fg(theme::GOOD),
                ))
            }
        }
    }
}

fn draw_sidebar(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM))
        .title(Span::styled(" Equipment ", Style::default().fg(theme::FG)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let g = &app.game;
    let mut lines: Vec<Line> = Vec::new();

    match &g.weapon {
        None => lines.push(Line::from(Span::styled(
            "Bare-handed",
            Style::default().fg(theme::DIM),
        ))),
        Some(w) => {
            lines.push(Line::from(vec![
                Span::styled("Weapon  ", Style::default().fg(theme::DIM)),
                Span::styled(
                    format!("{}{}", w.card.rank_label(), w.card.suit.glyph()),
                    Style::default()
                        .fg(theme::GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            let reach = match w.bound() {
                None => "hits anything".to_string(),
                Some(b) => match g.rule {
                    WeaponRule::Strict => format!("hits monsters < {b}"),
                    WeaponRule::Equal => format!("hits monsters ≤ {b}"),
                },
            };
            lines.push(Line::from(Span::styled(
                reach,
                Style::default().fg(theme::FG),
            )));
            if !w.slain.is_empty() {
                let slain: String = w
                    .slain
                    .iter()
                    .map(|c| c.rank_label().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                lines.push(Line::from(vec![
                    Span::styled("slain   ", Style::default().fg(theme::DIM)),
                    Span::styled(slain, Style::default().fg(theme::MONSTER)),
                ]));
            }
        }
    }

    lines.push(Line::default());
    lines.push(stat("Rooms cleared", g.rooms_cleared as i64));
    lines.push(stat("Monsters slain", g.monsters_slain as i64));
    lines.push(stat("Deck", g.deck.len() as i64));
    lines.push(stat("Discard", g.discard.len() as i64));

    f.render_widget(Paragraph::new(lines), inner);
}

fn stat(label: &str, value: i64) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<15}"), Style::default().fg(theme::DIM)),
        Span::styled(value.to_string(), Style::default().fg(theme::FG)),
    ])
}

fn draw_log(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::DIM))
        .title(Span::styled(" Chronicle ", Style::default().fg(theme::FG)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    let start = app.log.len().saturating_sub(height);
    let lines: Vec<Line> = app.log[start..].to_vec();
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_flash(f: &mut Frame, area: Rect, app: &App) {
    if let Some(msg) = &app.flash {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" {msg}"),
                Style::default().fg(theme::BAD).add_modifier(Modifier::BOLD),
            ))),
            area,
        );
    }
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let hint = |k: &'static str, d: &'static str| {
        vec![
            Span::styled(
                k,
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {d}   "), Style::default().fg(theme::DIM)),
        ]
    };
    let mut spans = vec![Span::raw(" ")];
    spans.extend(hint("←/→", "select"));
    spans.extend(hint("↵", "use"));
    spans.extend(hint("f", "blade"));
    spans.extend(hint("b", "fists"));
    spans.extend(hint("a", "avoid"));
    spans.extend(hint("n", "new"));
    spans.extend(hint("t", "rule"));
    spans.extend(hint("?", "help"));
    spans.extend(hint("q", "quit"));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    Rect::new(x, y, w, h)
}

fn game_over(f: &mut Frame, area: Rect, app: &App, verdict: &str) {
    let win = app.game.status == Status::Won;
    let color = if win { theme::GOOD } else { theme::BAD };
    let rect = centered(area, 52, 15);
    f.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(theme::PANEL));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let rows = Layout::vertical([
        Constraint::Length(1), // pad
        Constraint::Length(4), // big verdict
        Constraint::Length(1), // pad
        Constraint::Length(1), // score
        Constraint::Length(1), // seed
        Constraint::Length(1), // best
        Constraint::Min(0),
        Constraint::Length(1), // hint
    ])
    .split(inner);

    // Oversized verdict via tui-big-text (each glyph ≈ 4×4 cells at Quadrant).
    let big_w = 4 * verdict.chars().count() as u16;
    let big_area = card::top_centered(rows[1], big_w, 4);
    let big = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .lines(vec![verdict.to_string().into()])
        .build();
    f.render_widget(big, big_area);

    center(
        f,
        rows[3],
        vec![
            Span::styled("Score  ", Style::default().fg(theme::DIM)),
            Span::styled(
                app.game.score().to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ],
    );
    center(
        f,
        rows[4],
        vec![Span::styled(
            format!("seed {}", app.game.seed),
            Style::default().fg(theme::DIM),
        )],
    );
    if app.stats.has_record {
        center(
            f,
            rows[5],
            vec![Span::styled(
                format!(
                    "best {}   ·   {} wins / {} played",
                    app.stats.best_score, app.stats.wins, app.stats.games_played
                ),
                Style::default().fg(theme::DIM),
            )],
        );
    }

    let hint = |k: &'static str, d: &'static str| {
        [
            Span::styled(
                k,
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {d}   "), Style::default().fg(theme::DIM)),
        ]
    };
    let mut spans = Vec::new();
    spans.extend(hint("n", "new"));
    spans.extend(hint("r", "retry"));
    spans.extend(hint("t", "flip rule"));
    spans.extend(hint("q", "quit"));
    center(f, rows[7], spans);
}

fn center(f: &mut Frame, area: Rect, spans: Vec<Span<'static>>) {
    f.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::core::{Card, Suit, Weapon, WeaponRule};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::time::Duration;

    fn render(app: &mut App, w: u16, h: u16) -> String {
        render_dt(app, w, h, Duration::ZERO)
    }

    fn render_dt(app: &mut App, w: u16, h: u16, dt: Duration) -> String {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw(f, app, dt)).unwrap();
        format!("{}", terminal.backend())
    }

    #[test]
    fn full_scene_renders() {
        let mut app = App::new(7, WeaponRule::Strict);
        app.game.room = vec![
            Card::new(Suit::Spades, 13),
            Card::new(Suit::Hearts, 7),
            Card::new(Suit::Diamonds, 9),
            Card::new(Suit::Clubs, 4),
        ];
        app.game.weapon = Some(Weapon {
            card: Card::new(Suit::Diamonds, 10),
            slain: vec![Card::new(Suit::Spades, 8), Card::new(Suit::Clubs, 6)],
        });
        app.game.health = 14;

        let out = render(&mut app, 100, 30);
        // Structural assertions: the key chrome is present.
        assert!(out.contains("SCOUNDREL"));
        assert!(out.contains("Room"));
        assert!(out.contains("Equipment"));
        assert!(out.contains("Chronicle"));
        // Print for `cargo test full_scene_renders -- --nocapture`.
        println!("\n{out}");
    }

    #[test]
    fn game_over_renders_big_text() {
        let mut app = App::new(3, WeaponRule::Strict);
        app.game.status = Status::Lost;
        app.game.health = -3;
        let out = render(&mut app, 100, 30);
        println!("\n{out}");
        assert!(out.contains("Score"));
    }

    #[test]
    fn every_help_page_renders() {
        let mut app = App::new(1, WeaponRule::Strict);
        app.show_help = true;
        for page in 0..help::PAGE_COUNT {
            app.help_page = page;
            let out = render(&mut app, 90, 28);
            assert!(out.contains("How to play"));
        }
    }

    #[test]
    fn effects_process_across_frames() {
        // Actually drive tachyonfx's process() path (active effects), advancing
        // the animation frame by frame — must not panic and must keep drawing.
        let mut app = App::new(9, WeaponRule::Strict);
        app.fx.reveal_room();
        assert!(app.fx.active());
        for _ in 0..8 {
            let out = render_dt(&mut app, 100, 30, Duration::from_millis(80));
            assert!(out.contains("SCOUNDREL"));
            app.fx.gc();
        }
    }

    #[test]
    fn does_not_panic_on_tiny_terminal() {
        let mut app = App::new(1, WeaponRule::Strict);
        let _ = render(&mut app, 20, 8);
        // Overlays on a tiny terminal must not panic either.
        app.game.status = Status::Won;
        let _ = render(&mut app, 18, 6);
        app.show_help = true;
        let _ = render(&mut app, 10, 5);
        let _ = render(&mut app, 8, 4);
    }
}
