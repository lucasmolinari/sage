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
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{out, TAB_SZ};

#[allow(dead_code)]
enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct ERow {
    pub raw: Box<str>,
    pub render: String,
}
impl ERow {
    fn new(raw: Box<str>, render: String) -> Self {
        Self { raw, render }
    }
}

pub struct EditorRows {
    rows: Vec<ERow>,
    pub filename: Option<PathBuf>,
}
impl EditorRows {
    fn new() -> io::Result<Self> {
        let args: Vec<String> = env::args().collect();
        let first_line = ERow::new("".into(), String::new());
        match args.get(1) {
            Some(p) => {
                let path = Path::new(p).canonicalize()?;
                Ok(Self::from_file(path)?)
            }
            None => Ok(Self {
                rows: vec![first_line],
                filename: None,
            }),
        }
    }

    fn from_file(path: PathBuf) -> io::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        Ok(Self {
            rows: contents
                .lines()
                .map(|l| {
                    let mut row = ERow::new(l.into(), String::new());
                    Self::render_erow(&mut row);
                    row
                })
                .collect(),
            filename: Some(path),
        })
    }

    fn render_erow(row: &mut ERow) {
        let cap = row
            .raw
            .chars()
            .fold(0, |acc, next| acc + if next == '\t' { TAB_SZ } else { 1 });
        row.render = String::with_capacity(cap);

        let mut index = 0;
        row.raw.chars().for_each(|c| {
            index += 1;
            if c == '\t' {
                row.render.push(' ');
                while index % TAB_SZ != 0 {
                    row.render.push(' ');
                    index += 1;
                }
            } else {
                row.render.push(c);
            }
        })
    }

    pub fn get_raw(&self, i: usize) -> &str {
        &self.rows[i].raw
    }

    pub fn get_render(&self, i: usize) -> &String {
        &self.rows[i].render
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
            KeyCode::Char(c @ ('k' | 'j' | 'h' | 'l')) => self.output.move_cursor(c, &self.e_rows),
            _ => {}
        }
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        self.output.clear_screen().expect("Failed to clear screen");
        terminal::disable_raw_mode().expect("Failed to disable raw mode");
        execute!(stdout, LeaveAlternateScreen).expect("Failed to leave alternate screen");
    }
}
