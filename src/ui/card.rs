//! Rendering a single playing card as a bordered mini-widget.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use super::theme;
use crate::core::Card;

pub const CARD_W: u16 = 11;
pub const CARD_H: u16 = 7;

/// A rect of size `w`×`h` centred (horizontally) at the top of `area`.
pub fn top_centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    Rect::new(x, area.y, w, h)
}

/// Draw a card into `cell`, with an optional caption line beneath it.
pub fn render_card(f: &mut Frame, cell: Rect, card: Card, selected: bool, caption: Line) {
    let area = top_centered(cell, CARD_W, CARD_H);
    let pip = theme::suit_color(card.suit);

    let (border_type, border_style) = if selected {
        (
            BorderType::Double,
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (BorderType::Rounded, Style::default().fg(theme::DIM))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .style(Style::default().bg(theme::PANEL));

    let rank = card.rank_label();
    let pip_style = Style::default().fg(pip).add_modifier(Modifier::BOLD);
    let corner = Style::default().fg(pip);

    let lines = vec![
        Line::from(Span::styled(rank, corner)).alignment(Alignment::Left),
        Line::default(),
        Line::from(Span::styled(card.suit.glyph().to_string(), pip_style))
            .alignment(Alignment::Center),
        Line::default(),
        Line::from(Span::styled(rank, corner)).alignment(Alignment::Right),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);

    // Caption directly beneath the card.
    if area.bottom() < cell.bottom() {
        let cap_area = Rect::new(cell.x, area.bottom(), cell.width, 1);
        f.render_widget(
            Paragraph::new(caption).alignment(Alignment::Center),
            cap_area,
        );
    }
}
