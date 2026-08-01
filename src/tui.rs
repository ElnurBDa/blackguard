//! Terminal lifecycle and the blocking event loop.

use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::app::App;
use crate::ui;

/// Set up the terminal, run the loop, and always restore on the way out —
/// including on panic, so a crash never leaves a broken terminal.
pub fn run(mut app: App) -> Result<()> {
    install_panic_hook();
    let mut terminal = ratatui::init();
    app.fx.reveal_room(); // animate the opening deal
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    let mut last = Instant::now();
    while !app.should_quit {
        let now = Instant::now();
        let elapsed = now - last;
        last = now;

        terminal.draw(|f| ui::draw(f, app, elapsed))?;
        app.fx.gc();

        // Tick ~60fps while an animation is running; otherwise idle on input.
        let timeout = if app.fx.active() {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(200)
        };
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }
    }
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}
