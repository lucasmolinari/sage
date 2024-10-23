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

use crate::{editor::EditorRows, TAB_SZ};

pub struct Output {
    size: (usize, usize),
    c_ctrl: CursorController,
    out: BufWriter<Stdout>,
}
impl Output {
    pub fn new() -> io::Result<Self> {
        let size = terminal::size().map(|(x, y)| (x as usize, y as usize - 1))?;
        Ok(Self {
            size,
            c_ctrl: CursorController::new(size),
            out: BufWriter::new(io::stdout()),
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

    pub fn render_screen(&mut self, rows: &EditorRows) -> io::Result<()> {
        self.c_ctrl.scroll(rows);

        queue!(self.out, cursor::Hide, cursor::MoveTo(0, 0))?;

        self.render_lines(rows)?;
        self.render_bar(rows)?;

        let c_x = (self.c_ctrl.rx - self.c_ctrl.x_offset) as u16;
        let c_y = (self.c_ctrl.cy - self.c_ctrl.y_offset) as u16;

        queue!(self.out, cursor::Show, cursor::MoveTo(c_x, c_y))?;
        self.out.flush()
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
            "\"{}\" {}L, {}B",
            rows.filename
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("No name"),
            rows.num_rows(),
            rows.filename
                .as_ref()
                .and_then(|p| metadata(p).ok().map(|meta| meta.len()))
                .unwrap_or(0),
        );
        self.out.write(info_f.as_bytes())?;

        let info_c = format!("{}:{}/{}", c_y, c_x, self.c_ctrl.rx);
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
        Ok(())
    }

    pub fn move_cursor(&mut self, code: char, e_rows: &EditorRows) {
        self.c_ctrl.mv(code, e_rows);
    }
}

struct CursorController {
    cx: usize,
    cy: usize,
    rx: usize,
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
            screen_size,
            y_offset: 0,
            x_offset: 0,
        }
    }

    fn mv(&mut self, code: char, e_rows: &EditorRows) {
        let n_rows = e_rows.num_rows() - 1;
        match code {
            'k' => self.cy = self.cy.saturating_sub(1),
            'h' => self.cx = self.cx.saturating_sub(1),
            'j' => {
                if self.cy < n_rows {
                    self.cy += 1;
                }
            }
            'l' => {
                let row = e_rows.get_raw(self.cy);
                if self.cx < row.len().saturating_sub(1) {
                    self.cx += 1;
                    if row.chars().nth(self.cx) == Some('\t') {
                        self.cx += 1;
                    }
                }
            }
            _ => unimplemented!(),
        };
        let r_len = e_rows.get_raw(self.cy).len().saturating_sub(1);
        self.cx = cmp::min(self.cx, r_len);
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
