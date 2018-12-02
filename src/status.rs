//! the status module manages writing information on the grey line
//!  near the bottom of the screen

use std::io::{self, Write};
use termion::color;

use screens::Screen;

pub trait Status {
    fn write_status_text(&mut self, text: &str) -> io::Result<()>;
    fn write_status_err(&mut self, text: &str) -> io::Result<()>;
}

impl Status for Screen {
    fn write_status_err(&mut self, text: &str) -> io::Result<()> {
        let y = self.h - 1;
        write!(
            self.stdout,
            "{}{}{}{}{}{}{}",
            termion::cursor::Goto(1, y),
            color::Bg(color::AnsiValue::grayscale(2)),
            color::Fg(color::Red),
            termion::clear::CurrentLine,
            text,
            color::Bg(color::Reset),
            color::Fg(color::Reset),
        )?;
        self.stdout.flush()?;
        Ok(())
    }
    fn write_status_text(&mut self, text: &str) -> io::Result<()> {
        let y = self.h - 1;
        write!(
            self.stdout,
            "{}{}{}{}{}",
            termion::cursor::Goto(1, y),
            color::Bg(color::AnsiValue::grayscale(2)),
            termion::clear::CurrentLine,
            text,
            color::Bg(color::Reset),
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}