use crossterm::{
    cursor::{self, SetCursorStyle},
    execute, queue, style,
    terminal::{self, Clear, ClearType},
};
use std::{
    cmp,
    fs::metadata,
    io::{self, BufWriter, Stdout, Write},
};

use crate::{
    editor::{EditorRows, Mode},
    TAB_SZ,
};

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Output {
    size: (usize, usize),
    c_ctrl: CursorController,
    out: BufWriter<Stdout>,
    stt_msg: Option<StatusMessage>,
    cmd_msg: Option<StatusMessage>,
    pub cmd: Option<String>,
    pub dirty: u64,
}
impl Output {
    pub fn new() -> io::Result<Self> {
        let size = terminal::size().map(|(x, y)| (x as usize, y as usize - 2))?;
        Ok(Self {
            size,
            c_ctrl: CursorController::new(size),
            out: BufWriter::new(io::stdout()),
            stt_msg: None,
            cmd_msg: None,
            cmd: None,
            dirty: 0,
        })
    }

    pub fn clear_screen(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            SetCursorStyle::DefaultUserShape
        )
    }

    pub fn render_screen(&mut self, rows: &EditorRows, mode: &Mode) -> io::Result<()> {
        queue!(self.out, cursor::Hide, cursor::MoveTo(0, 0))?;

        self.c_ctrl.scroll(rows);
        let c_x = (self.c_ctrl.rx - self.c_ctrl.x_offset) as u16;
        let c_y = (self.c_ctrl.cy - self.c_ctrl.y_offset) as u16;

        self.render_lines(rows)?;
        self.render_bar(rows)?;

        match mode {
            Mode::Command => self.render_command()?,
            _ => {
                self.render_message()?;
                queue!(self.out, cursor::Show, cursor::MoveTo(c_x, c_y))?;
            }
        };
        self.out.flush()?;
        Ok(())
    }

    fn render_lines(&mut self, rows: &EditorRows) -> io::Result<()> {
        for i in 0..self.size.1 {
            queue!(self.out, Clear(ClearType::UntilNewLine))?;
            let i_offset = i + self.c_ctrl.y_offset;
            if i_offset >= rows.num_rows() {
                self.out.write(b"~")?;
            } else {
                let row = rows.get_render(i_offset);
                let len = cmp::min(row.len().saturating_sub(self.c_ctrl.x_offset), self.size.0);
                let start = if len > 0 { self.c_ctrl.x_offset } else { len };
                let content = &rows.get_render(i_offset)[start..start + len];
                self.out.write(content.as_bytes())?;
            }
            self.out.write(b"\r\n")?;
        }
        Ok(())
    }

    fn render_bar(&mut self, rows: &EditorRows) -> io::Result<()> {
        let c_x = self.c_ctrl.rx - self.c_ctrl.x_offset;
        let c_y = self.c_ctrl.cy - self.c_ctrl.y_offset;
        self.out
            .write(&style::Attribute::Reverse.to_string().as_bytes())?;
        let info_f = format!(
            "\"{}\"{} {}L, {}B",
            rows.filename
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("No name"),
            if self.dirty > 0 { "*" } else { "" },
            rows.num_rows(),
            rows.filename
                .as_ref()
                .and_then(|p| metadata(p).ok().map(|meta| meta.len()))
                .unwrap_or(0),
        );
        self.out.write(info_f.as_bytes())?;

        let info_c = format!(
            "{}:{}/{} {}",
            c_y + 1,
            c_x + 1,
            self.c_ctrl.rx + 1,
            self.c_ctrl.cmdx
        );
        let info_c_pos = self.size.0 - info_c.len();
        for i in info_f.len()..self.size.0 {
            if i >= info_c_pos {
                let index = i - info_c_pos..i - info_c_pos + 1;
                self.out.write(info_c[index].as_bytes())?;
            } else {
                self.out.write(b" ")?;
            }
        }
        self.out
            .write(&style::Attribute::Reset.to_string().as_bytes())?;
        self.out.write("\r\n".to_string().as_bytes())?;
        Ok(())
    }

    fn render_message(&mut self) -> io::Result<()> {
        queue!(self.out, Clear(ClearType::CurrentLine))?;
        if let Some(msg) = self.cmd_msg.as_ref().or(self.stt_msg.as_ref()) {
            let content = &msg.content;
            let style = match msg.level {
                MessageLevel::Normal => style::Attribute::Reset.to_string(),
                MessageLevel::Danger => style::SetBackgroundColor(style::Color::Red).to_string(),
            };
            self.out.write(style.as_bytes())?;
            self.out
                .write(content[..cmp::min(content.len(), self.size.0)].as_bytes())?;
            self.out
                .write(style::Attribute::Reset.to_string().as_bytes())?;
        }
        Ok(())
    }

    fn render_command(&mut self) -> io::Result<()> {
        let y = (self.size.1 + 2) as u16;
        queue!(
            self.out,
            Clear(ClearType::CurrentLine),
            cursor::Hide,
            cursor::MoveTo(0, y),
        )?;
        self.out.write(b":")?;

        if let Some(cmd) = &self.cmd {
            self.out.write(cmd.as_bytes())?;
        }
        queue!(
            self.out,
            cursor::MoveTo(self.c_ctrl.cmdx as u16, y),
            cursor::Show
        )?;

        Ok(())
    }

    pub fn insert(&mut self, e_rows: &mut EditorRows, c: char) {
        let (x, y) = (self.c_ctrl.cx, self.c_ctrl.cy);
        e_rows.get_erow_mut(y).insert(x, c);
        self.c_ctrl.cx += 1;
        self.dirty += 1;
    }

    pub fn new_line(&mut self, dir: Direction, e_rows: &mut EditorRows) {
        let y = match dir {
            Direction::Up => self.c_ctrl.cy,
            Direction::Down => self.c_ctrl.cy + 1,
            _ => unimplemented!(),
        };
        e_rows.insert_erow(y);
        self.c_ctrl.cy = y;
        self.c_ctrl.cx = 0;
        self.dirty += 1;
    }

    pub fn goto_y(&mut self, y: usize) {
        self.c_ctrl.cy = y;
    }

    pub fn move_cursor(&mut self, dir: Direction, e_rows: &EditorRows, mode: &Mode) {
        self.c_ctrl.mv(dir, e_rows, mode);
    }

    pub fn reset_cmd_cursor(&mut self) {
        self.c_ctrl.cmdx = 1;
    }

    pub fn set_stt_msg(&mut self, msg: &str, level: MessageLevel) {
        self.stt_msg = Some(StatusMessage::new(msg, level));
    }

    pub fn set_cmd_msg(&mut self, msg: &str, level: MessageLevel) {
        self.cmd_msg = Some(StatusMessage::new(msg, level));
    }

    pub fn clear_stt_msg(&mut self) {
        self.stt_msg = None;
    }

    pub fn clear_cmd_msg(&mut self) {
        self.cmd_msg = None;
    }

    pub fn push_cmd(&mut self, c: char) {
        self.cmd.get_or_insert_with(String::new).push(c);
    }

    pub fn get_cmd(&self) -> Option<Vec<&str>> {
        self.cmd.as_ref().map(|cmd| cmd.split(' ').collect())
    }

    pub fn clear_cmd(&mut self) {
        self.cmd = None;
    }
}

struct CursorController {
    cx: usize,
    cy: usize,
    rx: usize,
    cmdx: usize,
    screen_size: (usize, usize),
    y_offset: usize,
    x_offset: usize,
}
impl CursorController {
    fn new(screen_size: (usize, usize)) -> Self {
        Self {
            cx: 0,
            cy: 0,
            rx: 0,
            cmdx: 1,
            screen_size,
            y_offset: 0,
            x_offset: 0,
        }
    }

    fn mv(&mut self, dir: Direction, e_rows: &EditorRows, mode: &Mode) {
        let n_rows = e_rows.num_rows() - 1;
        let row = e_rows.get_raw(self.cy);
        let row_len = match mode {
            Mode::Normal => row.len().saturating_sub(1),
            _ => e_rows.get_raw(self.cy).len(),
        };
        match mode {
            Mode::Command => match dir {
                Direction::Left => self.cmdx = self.cmdx.saturating_sub(1),
                Direction::Right => {
                    if self.cmdx < row_len {
                        self.cmdx += 1;
                    }
                }
                _ => {}
            },
            _ => {
                match dir {
                    Direction::Up => self.cy = self.cy.saturating_sub(1),
                    Direction::Left => self.cx = self.cx.saturating_sub(1),
                    Direction::Down => {
                        if self.cy < n_rows {
                            self.cy += 1;
                        }
                    }
                    Direction::Right => {
                        if self.cx < row_len {
                            self.cx += 1;
                            if row.chars().nth(self.cx) == Some('\t') {
                                self.cx += 1;
                            }
                        }
                    }
                };
                self.cx = cmp::min(self.cx, row_len);
            }
        }
    }
    fn get_rx(&self, raw: &str) -> usize {
        raw.chars().take(self.cx).fold(0, |rx, c| {
            if c == '\t' {
                (rx + TAB_SZ) & !(TAB_SZ - 1)
            } else {
                rx + 1
            }
        })
    }

    fn scroll(&mut self, e_rows: &EditorRows) {
        self.rx = 0;
        if self.cy < e_rows.num_rows() {
            let row = e_rows.get_raw(self.cy);
            if self.cx == 0 && row.starts_with('\t') {
                self.cx = 1;
            }
            self.rx = self.get_rx(row);
        }

        self.y_offset = cmp::min(self.y_offset, self.cy);
        if self.cy >= self.y_offset + self.screen_size.1 {
            self.y_offset = self.cy - self.screen_size.1 + 1;
        }

        self.x_offset = cmp::min(self.x_offset, self.rx);
        if self.rx >= self.x_offset + self.screen_size.0 {
            self.x_offset = self.rx - self.screen_size.0 + 1;
        }
    }
}

pub enum MessageLevel {
    Normal,
    Danger,
}
struct StatusMessage {
    content: String,
    level: MessageLevel,
}
impl StatusMessage {
    fn new(msg: &str, level: MessageLevel) -> Self {
        Self {
            content: msg.into(),
            level,
        }
    }
}
