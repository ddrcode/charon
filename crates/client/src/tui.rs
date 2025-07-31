use ratatui::{
    Terminal,
    crossterm::{
        ExecutableCommand,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io::{Stdout, stdout};

pub fn suspend_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn resume_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(())
}
