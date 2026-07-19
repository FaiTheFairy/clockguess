pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: Vec::new(),
        }
    }

    fn set(&mut self, x: usize, y: usize, ch: char) {
        if x >= self.width || y >= self.height {
            return;
        }

        self.cells[y * self.width + x] = ch;
    }

    fn set_rounded(&mut self, x: f64, y: f64, ch: char) {
        self.set(x.round() as usize, y.round() as usize, ch);
    }

    fn render(&self) -> String {
        let mut output = String::with_capacity(self.width * self.height + self.height);

        for row in self.cells.chunks(self.width) {
            for ch in row {
                output.push(*ch);
            }
            output.push('\n');
        }

        output
    }
}
