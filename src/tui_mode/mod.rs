pub mod app;
mod ui;
mod helpers;

use anyhow::Result;
use app::App;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    cursor::{SetCursorStyle, Show},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;


pub(crate) fn run_tui() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    ui::run_ui_loop(&mut terminal, &mut app)?;

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, SetCursorStyle::BlinkingBar)?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show, SetCursorStyle::DefaultUserShape)?;
    Ok(())
}
