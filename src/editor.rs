use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    out::{self, Direction},
    TAB_SZ,
};

#[allow(dead_code)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Default)]
pub struct ERow {
    pub raw: String,
    pub render: String,
}
impl ERow {
    fn new(raw: String) -> Self {
        let mut row = Self {
            raw,
            render: String::new(),
        };
        row.render();
        row
    }

    pub fn insert(&mut self, i: usize, c: char) {
        self.raw.insert(i, c);
        self.render();
    }

    fn render(&mut self) {
        let cap = self
            .raw
            .chars()
            .fold(0, |acc, next| acc + if next == '\t' { TAB_SZ } else { 1 });
        self.render = String::with_capacity(cap);

        let mut index = 0;
        self.raw.chars().for_each(|c| {
            index += 1;
            if c == '\t' {
                self.render.push(' ');
                while index % TAB_SZ != 0 {
                    self.render.push(' ');
                    index += 1;
                }
            } else {
                self.render.push(c);
            }
        })
    }
}

pub struct EditorRows {
    rows: Vec<ERow>,
    pub filename: Option<PathBuf>,
}
impl EditorRows {
    fn new() -> io::Result<Self> {
        let args: Vec<String> = env::args().collect();
        match args.get(1) {
            Some(p) => {
                let path = Path::new(p).canonicalize()?;
                Ok(Self::from_file(path)?)
            }
            None => {
                let first_line = ERow::default();
                Ok(Self {
                    rows: vec![first_line],
                    filename: None,
                })
            }
        }
    }

    fn from_file(path: PathBuf) -> io::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        Ok(Self {
            rows: contents.lines().map(|l| ERow::new(l.into())).collect(),
            filename: Some(path),
        })
    }

    pub fn insert_erow(&mut self, i: usize) {
        self.rows.insert(i, ERow::default());
    }

    pub fn get_raw(&self, i: usize) -> &str {
        &self.rows[i].raw
    }

    pub fn get_render(&self, i: usize) -> &String {
        &self.rows[i].render
    }

    pub fn get_erow_mut(&mut self, i: usize) -> &mut ERow {
        &mut self.rows[i]
    }

    pub fn get_erows(&self) -> &Vec<ERow> {
        &self.rows
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
                        modifiers: KeyModifiers::CONTROL,
                        code: KeyCode::Char('s'),
                        ..
                    }) => self.save()?,
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code,
                        ..
                    }) => {
                        match self.mode {
                            Mode::Normal => self.handle_normal_press(code)?,
                            Mode::Insert => self.handle_insert_press(code)?,
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
            KeyCode::Up | KeyCode::Char('k') => {
                self.output.move_cursor(Direction::Up, &self.e_rows)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.output.move_cursor(Direction::Down, &self.e_rows)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.output.move_cursor(Direction::Left, &self.e_rows)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.output.move_cursor(Direction::Right, &self.e_rows)
            }
            KeyCode::Char('i') => self.change_mode(Mode::Insert)?,
            KeyCode::Char('a') => {
                self.change_mode(Mode::Insert)?;
                //self.output.move_cursor(Direction::Right, &self.e_rows);
            }
            KeyCode::Char('o') => {
                self.change_mode(Mode::Insert)?;
                self.output.insert_erow(&mut self.e_rows);
                self.output.move_cursor(Direction::Down, &self.e_rows);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_press(&mut self, code: KeyCode) -> io::Result<()> {
        match code {
            KeyCode::Esc => self.change_mode(Mode::Normal)?,
            KeyCode::Char(c) => self.output.insert(&mut self.e_rows, c),
            _ => {}
        }
        Ok(())
    }

    fn change_mode(&mut self, mode: Mode) -> io::Result<()> {
        let mut stdout = io::stdout();
        match mode {
            Mode::Normal => {
                execute!(stdout, SetCursorStyle::BlinkingBlock)?;
                self.output.move_cursor(Direction::Left, &self.e_rows);
            }
            Mode::Insert => execute!(stdout, SetCursorStyle::BlinkingUnderScore)?,
            Mode::Command => todo!(),
        };
        self.mode = mode;
        Ok(())
    }

    fn save(&self) -> io::Result<()> {
        match &self.e_rows.filename {
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "No file name specified",
            )),
            Some(name) => {
                let mut f = fs::OpenOptions::new().write(true).create(true).open(name)?;
                let contents = self
                    .e_rows
                    .get_erows()
                    .iter()
                    .map(|r| r.raw.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n");
                f.set_len(contents.len() as u64)?;
                f.write_all(contents.as_bytes())
            }
        }
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
