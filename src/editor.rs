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

use crate::grid::{Grid, Point};

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    orig_size: Point,
    size: Point,
    mode: Mode,
    command: String,
    last_c_pos: Point,
    grid: Grid,
}
impl Editor {
    pub fn new() -> io::Result<Self> {
        let orig_size = terminal::size()?;
        Ok(Editor {
            orig_size,
            size: orig_size,
            mode: Mode::Normal,
            command: String::new(),
            last_c_pos: (1, 1),
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
        execute!(&stdout, cursor::MoveTo(c_pos.0, c_pos.1), cursor::Show)?;
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
                    }) => {
                        match self.mode {
                            Mode::Normal => self.handle_normal_press(code)?,
                            Mode::Insert => self.handle_insert_press(code)?,
                            Mode::Command => {
                                let q = self.handle_command_press(code)?;
                                if q {
                                    break;
                                }
                            }
                        }
                        self.render_screen()?;
                    }
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
            KeyCode::Char(':') => {
                self.change_mode(Mode::Command)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_press(&mut self, code: KeyCode) -> io::Result<()> {
        let mut stdout = io::stdout();
        match code {
            KeyCode::Esc => self.change_mode(Mode::Normal)?,
            KeyCode::Char(c) => {
                let c_pos = cursor::position()?;
                self.grid.set(c_pos, c);
                execute!(stdout, cursor::MoveRight(1))?;
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_command_press(&mut self, code: KeyCode) -> io::Result<bool> {
        let mut stdout = io::stdout();
        match code {
            KeyCode::Enter => {
                let q = self.exec_cmd();
                self.change_mode(Mode::Normal)?;
                return q;
            }
            KeyCode::Esc => self.change_mode(Mode::Normal)?,
            KeyCode::Char(c) => {
                let (x, y) = cursor::position()?;
                self.command.push(c);
                self.grid.set((x, y), c);
                execute!(stdout, cursor::MoveRight(1))?;
            }
            _ => {}
        }
        Ok(false)
    }

    fn exec_cmd(&self) -> io::Result<bool> {
        match self.command.as_str() {
            "w" => todo!(),
            "q" => return Ok(true),
            "wq" => todo!(),
            _ => {}
        };
        Ok(false)
    }

    fn change_mode(&mut self, mode: Mode) -> io::Result<()> {
        let mut stdout = io::stdout();

        if self.mode == Mode::Command {
            self.command = "".to_string();

            self.grid.clear_row(self.size.1 - 1);

            let (x, y) = self.last_c_pos;
            execute!(stdout, cursor::MoveTo(x, y))?;
        }

        match mode {
            Mode::Normal => execute!(stdout, SetCursorStyle::BlinkingBlock)?,
            Mode::Insert => execute!(stdout, SetCursorStyle::BlinkingBar)?,
            Mode::Command => {
                self.last_c_pos = cursor::position()?;
                self.grid.set((0, self.size.1 - 1), ':');
                execute!(
                    stdout,
                    cursor::MoveTo(1, self.size.1 - 1),
                    SetCursorStyle::BlinkingBar
                )?
            }
        }
        self.mode = mode;
        Ok(())
    }
}
