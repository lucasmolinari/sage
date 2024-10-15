use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{Clear, ClearType, SetSize},
};
use std::{
    io::{self, BufWriter, Write},
    time::Duration,
};

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    c_pos: (u16, u16),
    orig_size: (u16, u16),
    size: (u16, u16),
    mode: Mode,
}
impl Editor {
    pub fn new() -> io::Result<Self> {
        let size = terminal::size()?;
        Ok(Editor {
            c_pos: (0, 0),
            orig_size: size,
            size,
            mode: Mode::Normal,
        })
    }
    pub fn init(&mut self) -> io::Result<()> {
        let stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            &stdout,
            cursor::Hide,
            SetCursorStyle::BlinkingBlock,
            EnterAlternateScreen,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        let (cols, rows) = self.size;
        let mut buffer = BufWriter::new(&stdout);

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
        execute!(&stdout, cursor::MoveTo(1, 0), cursor::Show)?;
        self.c_pos = (1, 0);

        Ok(())
    }

    pub fn destroy(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::disable_raw_mode()?;

        let (rows, cols) = self.orig_size;
        execute!(stdout, LeaveAlternateScreen, SetSize(rows, cols))?;

        Ok(())
    }

    pub fn poll(&mut self) -> io::Result<()> {
        loop {
            if event::poll(Duration::from_millis(100))? {
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
                    }) => match self.mode {
                        Mode::Normal => self.handle_normal_press(code)?,
                        Mode::Insert => self.handle_insert_press(code)?,
                        Mode::Command => todo!(),
                    },
                    _ => continue,
                }
            }
        }
        Ok(())
    }

    fn handle_normal_press(&mut self, code: KeyCode) -> io::Result<()> {
        let mut stdout = io::stdout();
        match code {
            KeyCode::Char('h') => execute!(stdout, cursor::MoveLeft(1))?,
            KeyCode::Char('l') => execute!(stdout, cursor::MoveRight(1))?,
            KeyCode::Char('k') => execute!(stdout, cursor::MoveUp(1))?,
            KeyCode::Char('j') => execute!(stdout, cursor::MoveDown(1))?,
            KeyCode::Char('i') => {
                execute!(stdout, cursor::MoveLeft(1))?;
                self.change_mode(Mode::Insert)?;
            }
            KeyCode::Char('a') => {
                execute!(stdout, cursor::MoveRight(1))?;
                self.change_mode(Mode::Insert)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_press(&mut self, code: KeyCode) -> io::Result<()> {
        let mut stdout = io::stdout();
        match code {
            KeyCode::Esc => self.change_mode(Mode::Normal)?,
            _ => {
                write!(stdout, "{}", code)?;
                stdout.flush()?;
            }
        };
        Ok(())
    }
    fn change_mode(&mut self, mode: Mode) -> io::Result<()> {
        let mut stdout = io::stdout();
        match mode {
            Mode::Normal => execute!(stdout, SetCursorStyle::BlinkingBlock)?,
            Mode::Insert => execute!(stdout, SetCursorStyle::BlinkingBar)?,
            Mode::Command => todo!(),
        }
        self.mode = mode;
        Ok(())
    }
}
