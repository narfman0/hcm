mod app;
mod backend;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hcm", about = "Manage multiple Claude Code / terminal sessions")]
struct Cli {
    /// Workspace directory
    #[arg(long, default_value = "~/.hcm")]
    workspace: PathBuf,

    /// Command to launch for new sessions
    #[arg(long, default_value = "claude --dangerously-skip-permissions")]
    cmd: String,

    /// Use plain "claude" without bypassing permissions
    #[arg(long)]
    no_dangerous: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let cmd = if cli.no_dangerous {
        "claude".to_string()
    } else {
        cli.cmd.clone()
    };

    let backend = backend::detect_backend();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend_term = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend_term)?;

    let app = app::App::new(backend, cmd);
    let result = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
