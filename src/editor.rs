use std::{
    env, fs,
    io::{self, Write},
    path::{self, PathBuf},
    time::Duration,
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    out::{self, Direction, MessageLevel},
    TAB_SZ,
};

#[derive(PartialEq)]
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
                let path = path::absolute(p)?;
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
        let contents = if path.try_exists()? {
            fs::read_to_string(&path)?
        } else {
            String::new()
        };

        let rows = if contents.is_empty() {
            vec![ERow::new(String::new())]
        } else {
            contents.lines().map(|l| ERow::new(l.into())).collect()
        };

        Ok(Self {
            rows,
            filename: Some(path),
        })
    }

    fn set_filename(&mut self, name: &str) {
        self.filename = Some(name.into());
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
        self.output.render_screen(&self.e_rows, &self.mode)?;
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
                    }) => {
                        self.save().map(|len| {
                            self.output.set_message(
                                &format!("{} bytes written to disk", len),
                                MessageLevel::Normal,
                            );
                            self.output.dirty = 0;
                        })?;
                        self.output.render_screen(&self.e_rows, &self.mode)?;
                    }
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
                        self.output.render_screen(&self.e_rows, &self.mode)?;
                    }
                    _ => continue,
                }
            }
        }
        Ok(())
    }

    fn handle_normal_press(&mut self, code: KeyCode) -> io::Result<()> {
        match code {
            KeyCode::Char(':') => self.change_mode(Mode::Command)?,
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

    fn handle_command_press(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Esc => self.change_mode(Mode::Normal)?,
            KeyCode::Char(c) => self.output.push_cmd(c),
            KeyCode::Enter => {
                let q = self.exec_cmd()?;
                self.change_mode(Mode::Normal)?;
                return Ok(q);
            }
            _ => {}
        }
        Ok(false)
    }

    fn exec_cmd(&mut self) -> io::Result<bool> {
        if let Some(it) = &self.output.get_cmd() {
            let q = match it[..] {
                ["q"] => {
                    if self.output.dirty > 0 {
                        self.output.set_message(
                            "Found unsaved changes, q! to force quit",
                            MessageLevel::Danger,
                        );
                        false
                    } else {
                        true
                    }
                }
                ["q!"] => true,
                ["w"] => {
                    if self.e_rows.filename.is_none() {
                        self.output
                            .set_message("No file name specified", MessageLevel::Danger);
                        return Ok(false);
                    }
                    self.save().map(|len| {
                        self.output.set_message(
                            &format!("{} bytes written to disk", len),
                            MessageLevel::Normal,
                        );
                        self.output.dirty = 0;
                    })?;
                    false
                }
                ["wq"] => {
                    match self.save() {
                        Ok(len) => {
                            self.output.set_message(
                                &format!("{} bytes written to disk", len),
                                MessageLevel::Normal,
                            );
                            self.output.dirty = 0;
                        }
                        Err(e) => {
                            self.output
                                .set_message(&e.to_string(), MessageLevel::Danger);
                            return Ok(false);
                        }
                    }
                    true
                }
                ["w", name] => {
                    self.e_rows.set_filename(name);
                    match self.save() {
                        Ok(len) => {
                            self.output.set_message(
                                &format!("{} bytes written to disk", len),
                                MessageLevel::Normal,
                            );
                            self.output.dirty = 0;
                        }
                        Err(e) => {
                            self.output
                                .set_message(&e.to_string(), MessageLevel::Danger);
                            return Ok(false);
                        }
                    };
                    false
                }
                _ => {
                    self.output.set_message(
                        &format!("Unknown command \"{}\"", it.concat()),
                        MessageLevel::Danger,
                    );
                    false
                }
            };
            return Ok(q);
        } else {
            self.output
                .set_message("No command found", MessageLevel::Danger);
            Ok(false)
        }
    }
    fn change_mode(&mut self, mode: Mode) -> io::Result<()> {
        let mut stdout = io::stdout();

        if self.mode == Mode::Command {
            self.output.clear_cmd();
        }
        match mode {
            Mode::Normal => {
                execute!(stdout, SetCursorStyle::BlinkingBlock)?;
                self.output.move_cursor(Direction::Left, &self.e_rows);
            }
            Mode::Insert => {
                execute!(stdout, SetCursorStyle::BlinkingUnderScore)?;
                self.output
                    .set_message("-- INSERT --", MessageLevel::Normal);
            }
            Mode::Command => {
                execute!(stdout, SetCursorStyle::BlinkingUnderScore)?;
            }
        };
        self.mode = mode;
        Ok(())
    }

    fn save(&self) -> io::Result<usize> {
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

                let bytes = contents.as_bytes();
                f.write_all(bytes)?;
                Ok(bytes.len())
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
