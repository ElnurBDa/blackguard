mod app;
mod cli;
mod core;
mod effects;
mod stats;
mod tui;
mod ui;

use clap::Parser;

use crate::app::App;
use crate::cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let app = App::new(cli.resolve_seed(), cli.rule.into());
    tui::run(app)
}
