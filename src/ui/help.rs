//! The paginated in-game rulebook. Content was drafted and fact-checked
//! against the engine; each line is kept <= 58 chars to fit the box.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use super::theme;

pub struct Page {
    pub title: &'static str,
    pub lines: &'static [&'static str],
}

pub const PAGES: &[Page] = &[
    Page {
        title: "The Goal",
        lines: &[
            "Scoundrel is a solo dungeon-crawl card game.",
            "The 44-card deck IS the dungeon. You face it",
            "one room at a time, card by card.",
            "",
            "You start at 20 HP — also your maximum.",
            "Clear the entire deck to WIN.",
            "If HP falls to 0, you LOSE.",
            "",
            "The more HP you keep, the higher your score.",
        ],
    },
    Page {
        title: "The Cards",
        lines: &[
            "44 cards: a standard deck minus the red",
            "face cards (J/Q/K of ♥ and ♦) and both",
            "red aces.",
            "",
            "  ♠ Spades   monsters   values 2-14",
            "  ♣ Clubs    monsters   values 2-14",
            "  ♦ Diamonds weapons    values 2-10",
            "  ♥ Hearts   potions    values 2-10",
            "",
            "Face cards: J=11  Q=12  K=13  A=14.",
        ],
    },
    Page {
        title: "Fighting",
        lines: &[
            "Every ♠ / ♣ monster must be fought.",
            "",
            "Bare-handed: lose HP equal to its value.",
            "",
            "With a weapon (♦) equipped:",
            "   damage = max(0, monster − weapon)",
            "So a ♦8 blade vs a ♠5 monster costs 0 HP.",
            "The slain monster tucks under the weapon.",
            "",
            "You may ALWAYS choose to fight bare-handed.",
        ],
    },
    Page {
        title: "Weapon Wear",
        lines: &[
            "A weapon weakens each time it kills.",
            "After a kill it may only face WEAKER",
            "monsters than its last slain (STRICT rule).",
            "",
            "Example: ♦9 weapon, STRICT rule",
            "  Slay ♠13 → now capped below 13",
            "  Slay ♣10 → ok (10 < 13), capped below 10",
            "  Face ♠10 → blade REFUSED (10 is not < 10)",
            "         so fight this one bare-handed.",
            "The tucked pile shows the last slain value.",
        ],
    },
    Page {
        title: "Potions & Rooms",
        lines: &[
            "A room is 4 face-up cards.",
            "You must resolve 3 of them; the 4th stays",
            "and the room refills to 4 next turn.",
            "",
            "♥ Potions heal by their value (HP ≤ 20).",
            "Only the FIRST potion drunk each room",
            "has any effect — extra ♥ that room are",
            "wasted.",
            "",
            "So choose which potion to drink first.",
        ],
    },
    Page {
        title: "Avoiding a Room",
        lines: &[
            "Not every room is worth the fight.",
            "",
            "Avoid (a): put all 4 cards on the BOTTOM",
            "of the deck, then deal 4 fresh cards.",
            "",
            "Two limits:",
            "  • You cannot avoid two rooms in a row.",
            "  • You may only avoid BEFORE playing any",
            "    card from the current room.",
        ],
    },
    Page {
        title: "Win, Lose, Score",
        lines: &[
            "WIN by clearing the whole deck.",
            "   Score = your remaining HP.",
            "   Bonus: at full 20 HP, if the very last",
            "   card is a potion, score = 20 + its value.",
            "",
            "LOSE if HP reaches 0.",
            "   Score = HP minus the total value of",
            "   every monster still left in the dungeon",
            "   (a negative number).",
        ],
    },
    Page {
        title: "Weapon Rule  (t)",
        lines: &[
            "Two weapon-wear rules exist:",
            "STRICT: next monster must be <  last slain.",
            "EQUAL : next monster must be ≤  last slain.",
            "",
            "EQUAL is gentler — a blade may re-slay a",
            "monster of the SAME value, so weapons last",
            "longer and rooms play out differently.",
            "",
            "Press t to flip the rule AND replay the",
            "same deck, to compare both strategies.",
        ],
    },
    Page {
        title: "Controls",
        lines: &[
            "arrows / h l    select a card",
            "Enter or Space  smart-use selected card",
            "f               fight with your blade",
            "b               fight bare-handed",
            "a               avoid the room",
            "n               new game",
            "r               retry the same seed",
            "t               flip weapon rule + replay",
            "?               open / close this help",
            "q               quit",
        ],
    },
];

pub const PAGE_COUNT: usize = PAGES.len();

/// Draw the help overlay showing `page_index` (clamped).
pub fn render(f: &mut Frame, area: Rect, page_index: usize) {
    let idx = page_index.min(PAGE_COUNT - 1);
    let page = &PAGES[idx];

    let rect = super::centered(area, 62, 18);
    f.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::PANEL))
        .title(Span::styled(
            format!(" How to play — {} ", page.title),
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(inner);

    let body: Vec<Line> = page
        .lines
        .iter()
        .map(|l| Line::from(Span::styled(*l, Style::default().fg(theme::FG))))
        .collect();
    f.render_widget(Paragraph::new(body), rows[0]);

    let nav = Line::from(vec![
        Span::styled(
            "←/→",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" pages    ", Style::default().fg(theme::DIM)),
        Span::styled(
            format!("{}/{}", idx + 1, PAGE_COUNT),
            Style::default().fg(theme::FG),
        ),
        Span::styled("    ", Style::default().fg(theme::DIM)),
        Span::styled(
            "?",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" close", Style::default().fg(theme::DIM)),
    ])
    .alignment(Alignment::Center);
    f.render_widget(Paragraph::new(nav), rows[1]);
}
