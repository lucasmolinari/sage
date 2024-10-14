use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, Stdout, Write},
    time::Duration,
};

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn init(mut stdout: &Stdout) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, Clear(ClearType::All))?;
    Ok(())
}

pub fn destroy(mut stdout: &Stdout) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

pub fn poll(mut stdout: &Stdout) -> io::Result<()> {
    loop {
        if event::poll(Duration::from_millis(500))? {
            let event = event::read()?;

            match event {
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('q'),
                    ..
                }) => break,
                Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code,
                    ..
                }) => {
                    write!(stdout, "{}", code)?;
                    stdout.flush()?;
                }
                _ => continue,
            }
        }
    }
    Ok(())
}
