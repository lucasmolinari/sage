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
        self.chars[index] = c;
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
