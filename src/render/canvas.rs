pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Canvas {
    pub(super) fn new(width: u16, height: u16) -> Self {
        assert!(width > 0, "canvas width must be greater than zero");
        assert!(height > 0, "canvas height must be greater than zero");
        let (width, height) = (usize::from(width), usize::from(height));
        Self {
            width,
            height,
            cells: vec![' '; width * height],
        }
    }

    pub(super) fn set(&mut self, x: isize, y: isize, ch: char) {
        if x < 0 || y < 0 {
            return;
        }

        let (x, y) = (x as usize, y as usize);

        if x >= self.width || y >= self.height {
            return;
        }

        self.cells[y * self.width + x] = ch;
    }

    pub(super) fn set_rounded(&mut self, x: f64, y: f64, ch: char) {
        self.set(x.round() as isize, y.round() as isize, ch);
    }

    pub(super) fn line(&mut self, start: Point, end: Point, ch: char) {
        let mut x = start.x;
        let mut y = start.y;

        let dx = (end.x - start.x).abs();
        let sx = if start.x < end.x { 1 } else { -1 };

        let dy = -((end.y - start.y).abs());
        let sy = if start.y < end.y { 1 } else { -1 };

        let mut error = dx + dy;

        loop {
            self.set(x, y, ch);

            if x == end.x && y == end.y {
                break;
            }

            let error_twice = 2 * error;

            if error_twice >= dy {
                error += dy;
                x += sx;
            }

            if error_twice <= dx {
                error += dx;
                y += sy;
            }
        }
    }

    /// Renders canvas.
    ///
    /// # Note
    /// - `render` does not add a trailing newline after the final row.
    pub(super) fn render(&self) -> String {
        let mut output = String::with_capacity(self.cells.len() + self.height - 1);

        for (idx, row) in self.cells.chunks(self.width).enumerate() {
            if idx > 0 {
                output.push('\n');
            }
            output.extend(row);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_horizontal_line() {
        let mut canvas = Canvas::new(5, 3);

        canvas.line(Point { x: 0, y: 1 }, Point { x: 4, y: 1 }, '#');

        assert_eq!(canvas.render(), "     \n#####\n     ");
    }

    #[test]
    fn draws_vertical_line() {
        let mut canvas = Canvas::new(3, 3);

        canvas.line(Point { x: 1, y: 0 }, Point { x: 1, y: 2 }, '#');

        assert_eq!(canvas.render(), " # \n # \n # ");
    }

    #[test]
    fn draws_diagonal_line() {
        let mut canvas = Canvas::new(3, 3);

        canvas.line(Point { x: 0, y: 0 }, Point { x: 2, y: 2 }, '#');

        assert_eq!(canvas.render(), "#  \n # \n  #");
    }

    #[test]
    fn draws_steep_line() {
        let mut canvas = Canvas::new(3, 5);

        canvas.line(Point { x: 0, y: 0 }, Point { x: 2, y: 4 }, '#');

        assert_eq!(canvas.render(), "#  \n # \n # \n  #\n  #");
    }
}
