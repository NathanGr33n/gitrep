//! GitRep - Git Repository Inspector TUI
//!
//! A high-performance terminal user interface for exploring and analyzing Git repositories.

use anyhow::Result;
use gitrep::app::App;
use gitrep::ui::Tui;
use std::env;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;

    // Determine repository path (current directory or argument)
    let repo_path = get_repo_path()?;

    // Initialize and run the application
    let mut app = App::new(repo_path)?;
    let mut tui = Tui::new()?;

    tui.enter()?;
    let result = app.run(&mut tui).await;
    tui.exit()?;

    result
}

/// Initialize the tracing subscriber for logging
fn init_logging() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    Ok(())
}

/// Get the repository path from command line argument or current directory
fn get_repo_path() -> Result<PathBuf> {
    let args: Vec<String> = env::args().collect();

    let path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir()?
    };

    Ok(path)
}
