pub type Point = (usize, usize);
pub struct Grid {
    pub width: usize,
    pub height: usize,
    chars: Vec<char>,
}
impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;

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

    pub fn get(&self, p: Point) -> &char {
        let index = (p.1 * self.width + p.0) as usize;
        &self.chars[index]
    }

    pub fn set(&mut self, p: Point, c: char) {
        let index = (p.1 * self.width + p.0) as usize;
        if self.chars[index] != ' ' && self.chars[index] != '~' && self.chars[index] != ':' {
            self.shift_row(p.1, index);
        }
        self.chars[index] = c;
    }
    fn shift_row(&mut self, row: usize, start: usize) {
        let end = row * self.width + self.width;
        for i in (start..end - 1).rev() {
            let is_last_row = row + 1 >= self.height;
            if !is_last_row && i == end - 1 {
                self.set((0, row + 1), self.chars[i])
            } else {
                self.chars[i + 1] = self.chars[i];
            }
        }
    }

    pub fn clear_row(&mut self, row: usize) {
        if row < self.height {
            for x in 0..self.width {
                let c = if x == 0 { '~' } else { ' ' };
                self.set((x, row), c);
            }
        }
    }
}
