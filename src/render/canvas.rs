use std::io::{self, Write};

use crossterm::{
    QueueableCommand,
    style::{ContentStyle, PrintStyledContent},
};

pub(super) struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Cell {
    ch: char,
    style: ContentStyle,
}

impl Cell {
    pub(super) fn plain(ch: char) -> Self {
        Self {
            ch,
            style: ContentStyle::default(),
        }
    }

    pub(super) const fn styled(ch: char, style: ContentStyle) -> Self {
        Self { ch, style }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Point {
    pub(super) x: isize,
    pub(super) y: isize,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum TextAlign {
    Left,
    Center,
    Right,
}

impl Canvas {
    pub(super) fn new(width: u16, height: u16) -> Self {
        assert!(width > 0, "canvas width must be greater than zero");
        assert!(height > 0, "canvas height must be greater than zero");
        let (width, height) = (usize::from(width), usize::from(height));
        Self {
            width,
            height,
            cells: vec![Cell::plain(' '); width * height],
        }
    }

    fn set_cell(&mut self, x: isize, y: isize, cell: Cell) {
        if x < 0 || y < 0 {
            return;
        }

        let (x, y) = (x.cast_unsigned(), y.cast_unsigned());

        if x >= self.width || y >= self.height {
            return;
        }

        self.cells[y * self.width + x] = cell;
    }

    #[allow(dead_code)]
    pub(super) fn set(&mut self, x: isize, y: isize, ch: char) {
        self.set_cell(x, y, Cell::plain(ch));
    }

    pub(super) fn set_styled(&mut self, point: Point, ch: char, style: ContentStyle) {
        self.set_cell(point.x, point.y, Cell::styled(ch, style));
    }

    /// Text placement assumes every character occupies one terminal column
    pub(super) fn text(&mut self, start: Point, text: &str, style: ContentStyle) {
        for (offset, ch) in text.chars().enumerate() {
            self.set_cell(
                start.x + offset.cast_signed(),
                start.y,
                Cell::styled(ch, style),
            );
        }
    }

    /// Text placement assumes every character occupies one terminal column
    pub(super) fn text_aligned(
        &mut self,
        anchor: Point,
        text: &str,
        style: ContentStyle,
        alignment: TextAlign,
    ) {
        let width = text.chars().count().cast_signed();

        let start_x = match alignment {
            TextAlign::Left => anchor.x,
            TextAlign::Center => anchor.x - width / 2,
            TextAlign::Right => anchor.x - width + 1,
        };

        self.text(
            Point {
                x: start_x,
                y: anchor.y,
            },
            text,
            style,
        );
    }

    fn line_cell(&mut self, start: Point, end: Point, cell: Cell) {
        let mut x = start.x;
        let mut y = start.y;

        let dx = (end.x - start.x).abs();
        let sx = if start.x < end.x { 1 } else { -1 };

        let dy = -((end.y - start.y).abs());
        let sy = if start.y < end.y { 1 } else { -1 };

        let mut error = dx + dy;

        loop {
            self.set_cell(x, y, cell);

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

    #[allow(dead_code)]
    pub(super) fn line(&mut self, start: Point, end: Point, ch: char) {
        self.line_cell(start, end, Cell::plain(ch));
    }

    pub(super) fn line_styled(&mut self, start: Point, end: Point, ch: char, style: ContentStyle) {
        self.line_cell(start, end, Cell::styled(ch, style));
    }

    /// Renders canvas.
    ///
    /// # Note
    /// - `render` does not add a trailing newline after the final row.
    pub(super) fn render(&self) -> String {
        let mut output = Vec::new();

        self.write_to(&mut output)
            .expect("writing canvas to memory should succeed");

        String::from_utf8(output).expect("crossterm generated valid UTF-8")
    }

    #[allow(dead_code)]
    pub(super) const fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    pub(super) const fn height(&self) -> usize {
        self.height
    }

    #[allow(dead_code)]
    pub(super) fn clear(&mut self, ch: char) {
        self.cells.fill(Cell::plain(ch));
    }

    fn write_to<W: Write>(&self, output: &mut W) -> io::Result<()> {
        for (row_index, row) in self.cells.chunks(self.width).enumerate() {
            if row_index > 0 {
                output.write_all(b"\n")?;
            }

            for cell in row {
                output.queue(PrintStyledContent(cell.style.apply(cell.ch)))?;
            }
        }

        Ok(())
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

    #[test]
    fn draws_text() {
        let mut canvas = Canvas::new(5, 1);

        canvas.text(Point { x: 1, y: 0 }, "12", ContentStyle::default());

        assert_eq!(canvas.render(), " 12  ");
    }

    #[test]
    fn centers_text() {
        let mut canvas = Canvas::new(5, 1);

        canvas.text_aligned(
            Point { x: 2, y: 0 },
            "12",
            ContentStyle::default(),
            TextAlign::Center,
        );

        assert_eq!(canvas.render(), " 12  ");
    }

    #[test]
    fn renders_styled_text() {
        use crossterm::style::{Color, Stylize};

        let mut canvas = Canvas::new(1, 1);

        canvas.set_styled(
            Point { x: 0, y: 0 },
            '#',
            ContentStyle::default().with(Color::Red),
        );

        let rendered = canvas.render();

        assert!(rendered.contains('#'));
        assert_ne!(rendered, "#");
    }
}
