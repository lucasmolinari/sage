pub struct Point {
    pub x: usize,
    pub y: usize,
}

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
            if index != 0 {
                chars[index] = '~';
            }
        }
        Self {
            chars,
            width,
            height,
        }
    }

    pub fn point_index(&self, p: &Point) -> usize {
        p.y * self.width + p.x
    }

    pub fn get(&self, p: &Point) -> &char {
        let index = self.point_index(&p);
        &self.chars[index]
    }

    pub fn set(&mut self, p: &Point, c: char) {
        let index = self.point_index(&p);
        if self.chars[index] != ' ' && self.chars[index] != '~' && self.chars[index] != ':' {
            self.shift_row_right(p);
        }
        self.chars[index] = c;
    }

    fn shift_row_right(&mut self, p: &Point) {
        let start = self.point_index(&p);
        let end = (p.y + 1) * self.width - 1;
        let last_char = self.chars[end];

        for i in (start..end).rev() {
            self.chars[i + 1] = self.chars[i];
        }

        if p.y + 1 < self.height && last_char != ' ' {
            self.set(&Point { x: 0, y: p.y + 1 }, last_char);
        }
    }

    fn shift_row_left(&mut self, p: &Point) {
        let start = self.point_index(&p);
        let end = (p.y + 1) * self.width - 1;

        for i in start..end {
            self.chars[i] = self.chars[i + 1];
        }
        self.chars[end] = ' '
    }

    pub fn clear_char(&mut self, p: &Point) {
        let index = self.point_index(&p);

        self.chars[index] = ' ';
        self.shift_row_left(&p);
    }

    pub fn clear_row(&mut self, y: usize) {
        if y < self.height {
            for x in 0..self.width {
                let c = if x == 0 { '~' } else { ' ' };
                self.set(&Point { x, y }, c);
            }
        }
    }
}
