use crossterm::{
    cursor::{self},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, BufWriter, Stdout, Write},
    time::Duration,
};

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn init(mut stdout: &Stdout) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        cursor::Hide,
        EnterAlternateScreen,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
    )?;

    let (cols, rows) = terminal::size()?;
    let mut buffer = BufWriter::new(stdout);

    for y in 0..rows {
        buffer.write(b"~")?;
        if y == rows / 3 {
            let title = "Sage Text Editor";
            let padding = (cols - title.len() as u16) / 2 - 1;
            for _ in 0..=padding {
                buffer.write(b" ")?;
            }
            buffer.write(title.as_bytes())?;
        }
        if y != rows - 1 {
            buffer.write(b"\r\n")?;
        }
    }
    buffer.flush()?;
    execute!(stdout, cursor::MoveTo(1, 0), cursor::Show)?;

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
