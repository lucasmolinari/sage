use std::{
    env, fs,
    io::{self},
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen},
};

use crate::out;

#[allow(dead_code)]
enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct EditorRows {
    rows: Vec<Box<str>>,
}
impl EditorRows {
    fn new() -> io::Result<Self> {
        let args: Vec<String> = env::args().collect();

        match args.get(1) {
            Some(p) => {
                let path = Path::new(p).canonicalize()?;
                Ok(Self::from_file(&path)?)
            }
            None => Ok(Self {
                rows: vec![" ".into()],
            }),
        }
    }

    fn from_file(path: &PathBuf) -> io::Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(Self {
            rows: contents.lines().map(|l| l.into()).collect(),
        })
    }

    pub fn get(&self, i: usize) -> &str {
        &self.rows[i]
    }

    pub fn num_rows(&self) -> usize {
        self.rows.len()
    }
}

pub struct Editor {
    mode: Mode,
    output: out::Output,
    e_rows: EditorRows,
}
impl Editor {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            mode: Mode::Normal,
            output: out::Output::new()?,
            e_rows: EditorRows::new()?,
        })
    }
    pub fn init(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            SetCursorStyle::BlinkingBlock,
            EnterAlternateScreen,
            cursor::MoveTo(0, 0)
        )?;
        self.output.render_screen(&self.e_rows)?;
        Ok(())
    }
    pub fn poll(&mut self) -> io::Result<()> {
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
                        match self.mode {
                            Mode::Normal => self.handle_normal_press(code)?,
                            Mode::Insert => todo!(),
                            Mode::Command => todo!(),
                        }
                        self.output.render_screen(&self.e_rows)?;
                    }
                    _ => continue,
                }
            }
        }
        Ok(())
    }

    fn handle_normal_press(&mut self, code: KeyCode) -> io::Result<()> {
        match code {
            KeyCode::Char(c @ ('k' | 'j' | 'h' | 'l')) => {
                self.output.move_cursor(c, self.e_rows.num_rows())
            }
            _ => {}
        }
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disable raw mode");
        self.output.clear_screen().expect("Failed to clear screen");
    }
}
