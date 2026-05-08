mod app;
mod gh;
mod state;
mod tmux;
mod ui;
mod worktree;

use anyhow::{Context, Result};
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
    /// Workspace directory (defaults to ~/.hcm)
    #[arg(long)]
    workspace: Option<PathBuf>,

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

    let workspace = match cli.workspace {
        Some(p) => p,
        None => dirs::home_dir()
            .context("could not resolve home directory")?
            .join(".hcm"),
    };
    std::fs::create_dir_all(&workspace).context("failed to create hcm workspace dir")?;

    let cwd = std::env::current_dir().context("failed to read current dir")?;

    if !tmux_available() {
        eprintln!("error: tmux not found in PATH. hcm requires tmux on Linux/Mac.");
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend_term = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend_term)?;

    let app = app::App::new(tmux::Tmux::new(), cmd, workspace, cwd);
    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

fn tmux_available() -> bool {
    std::process::Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
