use std::io;

use anyhow::Result;
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

pub struct TerminalCleanupGuard {
    active: bool,
}

impl TerminalCleanupGuard {
    pub fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self { active: true })
    }

    pub fn finish(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        self.active = false;
        execute!(io::stdout(), Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for TerminalCleanupGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        self.active = false;
    }
}
