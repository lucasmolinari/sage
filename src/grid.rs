pub type Point = (u16, u16);
pub struct Grid {
    pub width: u16,
    pub height: u16,
    chars: Vec<char>,
}
impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width * height) as usize;

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

    pub fn clear_row(&mut self, row: u16) {
        if row < self.height {
            for x in 0..self.width {
                let c = if x == 0 { '~' } else { ' ' };
                self.set((x, row), c);
            }
        }
    }
}
