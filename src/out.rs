use crossterm::{
    cursor::{self, SetCursorStyle},
    execute, queue,
    terminal::{self, Clear, ClearType},
};
use std::{
    cmp,
    io::{self, BufWriter, Write},
};

use crate::editor::EditorRows;

pub struct Output {
    size: (usize, usize),
    c_ctrl: CursorController,
}
impl Output {
    pub fn new() -> io::Result<Self> {
        let size = terminal::size().map(|(x, y)| (x as usize, y as usize))?;
        Ok(Self {
            size,
            c_ctrl: CursorController::new(size),
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
        self.c_ctrl.scroll();

        let stdout = io::stdout();
        let mut buffer = BufWriter::new(&stdout);

        queue!(buffer, cursor::Hide, cursor::MoveTo(0, 0))?;

        for i in 0..self.size.1 {
            let i_offset = i + self.c_ctrl.y_offset;
            if i_offset >= rows.num_rows() {
                buffer.write(b"~")?;
                queue!(buffer, Clear(ClearType::UntilNewLine))?;
            } else {
                let row = rows.get(i_offset);
                let len = cmp::min(row.len().saturating_sub(self.c_ctrl.x_offset), self.size.0);
                let start = if len > 0 { self.c_ctrl.x_offset } else { len };
                let content = &rows.get(i_offset)[start..start + len];
                buffer.write(content.as_bytes())?;
            }

            if i < self.size.1 - 1 {
                buffer.write(b"\r\n")?;
            }
        }
        let c_x = (self.c_ctrl.x - self.c_ctrl.x_offset) as u16;
        let c_y = (self.c_ctrl.y - self.c_ctrl.y_offset) as u16;

        queue!(buffer, cursor::Show, cursor::MoveTo(c_x, c_y))?;
        buffer.flush()
    }

    pub fn move_cursor(&mut self, code: char, n_rows: usize) {
        self.c_ctrl.mv(code, n_rows);
    }
}

struct CursorController {
    x: usize,
    y: usize,
    screen_size: (usize, usize),
    y_offset: usize,
    x_offset: usize,
}
impl CursorController {
    fn new(screen_size: (usize, usize)) -> Self {
        Self {
            x: 0,
            y: 0,
            screen_size,
            y_offset: 0,
            x_offset: 0,
        }
    }

    fn mv(&mut self, code: char, n_rows: usize) {
        match code {
            'k' => self.y = self.y.saturating_sub(1),
            'h' => self.x = self.x.saturating_sub(1),
            'j' => {
                if self.y < n_rows - 1 {
                    self.y += 1;
                }
            }
            'l' => self.x += 1,
            _ => unimplemented!(),
        };
    }

    fn scroll(&mut self) {
        self.y_offset = cmp::min(self.y_offset, self.y);
        if self.y >= self.y_offset + self.screen_size.1 {
            self.y_offset = self.y - self.screen_size.1 + 1;
        }

        self.x_offset = cmp::min(self.x_offset, self.x);
        if self.x >= self.x_offset + self.screen_size.0 {
            self.x_offset = self.x - self.screen_size.0 + 1;
        }
    }
}
