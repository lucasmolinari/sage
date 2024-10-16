use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{Clear, ClearType, SetSize},
};
use std::{
    io::{self, BufWriter, Write},
    time::Duration,
    usize,
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

type Point = (u16, u16);
struct Grid {
    chars: Vec<char>,
    width: u16,
    height: u16,
}
impl Grid {
    fn new(width: u16, height: u16) -> Self {
        let size = (width * height) as usize;

        let mut chars = vec![' '; size];
        for y in 0..height {
            let index = (y * width) as usize;
            chars[index] = '~';
        }
        Self {
            chars,
            width,
            height,
        }
    }

    fn get(&self, p: Point) -> &char {
        let index = (p.1 * self.width + p.0) as usize;
        &self.chars[index]
    }

    fn set(&mut self, p: Point, c: char) {
        let index = (p.1 * self.width + p.0) as usize;
        self.chars[index] = c;
    }
}

pub struct Editor {
    orig_size: Point,
    size: Point,
    mode: Mode,
    grid: Grid,
}
impl Editor {
    pub fn new() -> io::Result<Self> {
        let orig_size = terminal::size()?;
        Ok(Editor {
            orig_size,
            size: orig_size,
            mode: Mode::Normal,
            grid: Grid::new(orig_size.0, orig_size.1),
        })
    }
    pub fn init(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            SetCursorStyle::BlinkingBlock,
            EnterAlternateScreen,
            cursor::MoveTo(0, 0),
        )?;

        self.render_screen()?;
        execute!(stdout, cursor::MoveTo(1, 0))?;
        Ok(())
    }

    fn refresh_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All))?;
        Ok(())
    }

    fn render_screen(&self) -> io::Result<()> {
        let stdout = io::stdout();

        let c_pos = cursor::position()?;
        execute!(&stdout, cursor::Hide, cursor::MoveTo(0, 0))?;

        let mut buffer = BufWriter::new(&stdout);
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let char = self.grid.get((x, y));
                buffer.write(char.to_string().as_bytes())?;
            }
        }
        buffer.flush()?;
        execute!(&stdout, cursor::MoveTo(c_pos.0 + 1, c_pos.1), cursor::Show)?;
        Ok(())
    }

    pub fn destroy(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        self.refresh_screen()?;
        terminal::disable_raw_mode()?;

        let (rows, cols) = self.orig_size;
        execute!(
            stdout,
            LeaveAlternateScreen,
            SetSize(rows, cols),
            SetCursorStyle::DefaultUserShape
        )?;

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
        let c_pos = cursor::position()?;
        match code {
            KeyCode::Char('h') => {
                if c_pos.0 != 1 {
                    execute!(stdout, cursor::MoveLeft(1))?
                }
            }
            KeyCode::Char('l') => {
                if c_pos.0 != self.size.0 {
                    execute!(stdout, cursor::MoveRight(1))?
                }
            }
            KeyCode::Char('k') => {
                if c_pos.1 != 0 {
                    execute!(stdout, cursor::MoveUp(1))?
                }
            }
            KeyCode::Char('j') => {
                if c_pos.1 != self.size.1 {
                    execute!(stdout, cursor::MoveDown(1))?
                }
            }
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
        match code {
            KeyCode::Esc => self.change_mode(Mode::Normal)?,
            KeyCode::Char(c) => {
                let c_pos = cursor::position()?;
                self.grid.set(c_pos, c);
                self.render_screen()?;
            }
            _ => {}
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
