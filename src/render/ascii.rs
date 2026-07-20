use std::f64::consts::TAU;

use crate::{
    clock::ClockTime,
    render::{
        ClockRenderer,
        canvas::{Canvas, Point},
    },
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AsciiRenderer {
    /// Ratio of terminal cell height over width.
    aspect_ratio: f64,
}

#[derive(Copy, Clone, Debug)]
struct ClockLayout {
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
}

impl ClockLayout {
    fn new(width: u16, height: u16, aspect_ratio: f64) -> Self {
        let center_x = (f64::from(width) - 1.0) / 2.0;
        let center_y = (f64::from(height) - 1.0) / 2.0;

        let available_x = center_x;
        let available_y = center_y;

        let radius_y = available_y.min(available_x / aspect_ratio) * 0.9;
        let radius_x = radius_y * aspect_ratio;

        Self {
            center_x,
            center_y,
            radius_x,
            radius_y,
        }
    }

    fn center(self) -> Point {
        Point {
            x: self.center_x.round() as isize,
            y: self.center_y.round() as isize,
        }
    }

    fn point_at(self, angle: f64, length: f64) -> Point {
        Point {
            x: (self.center_x + self.radius_x * length * angle.sin()).round() as isize,
            y: (self.center_y - self.radius_y * length * angle.cos()).round() as isize,
        }
    }
}

impl AsciiRenderer {
    pub fn new(aspect_ratio: f64) -> Self {
        assert!(
            aspect_ratio.is_finite() && aspect_ratio > 0.0,
            "aspect ratio must be finite and positive"
        );

        Self { aspect_ratio }
    }

    fn draw_face(&self, canvas: &mut Canvas, layout: ClockLayout) {
        for index in 0..60 {
            let angle = f64::from(index) / 60.0 * TAU;
            let point = layout.point_at(angle, 1.0);
            let ch = if index % 5 == 0 { 'O' } else { '.' };

            canvas.set(point.x, point.y, ch);
        }
    }

    fn draw_hour_hand(&self, canvas: &mut Canvas, layout: ClockLayout, time: ClockTime) {
        canvas.line(
            layout.center(),
            layout.point_at(time.hour_angle(), 0.5),
            '#',
        );
    }

    fn draw_minute_hand(&self, canvas: &mut Canvas, layout: ClockLayout, time: ClockTime) {
        canvas.line(
            layout.center(),
            layout.point_at(time.minute_angle(), 0.75),
            '=',
        );
    }

    fn draw_second_hand(&self, canvas: &mut Canvas, layout: ClockLayout, time: ClockTime) {
        canvas.line(
            layout.center(),
            layout.point_at(time.second_angle(), 0.95),
            '-',
        );
    }
}

impl ClockRenderer for AsciiRenderer {
    type Output = String;

    fn render(&self, time: ClockTime, width: u16, height: u16) -> String {
        let mut canvas = Canvas::new(width, height);
        let layout = ClockLayout::new(width, height, self.aspect_ratio);

        self.draw_face(&mut canvas, layout);
        self.draw_hour_hand(&mut canvas, layout, time);
        self.draw_minute_hand(&mut canvas, layout, time);
        self.draw_second_hand(&mut canvas, layout, time);

        // Redraw the center so the pivot remains visible.
        let Point { x, y } = layout.center();
        canvas.set(x, y, '@');

        canvas.render()
    }
}

impl Default for AsciiRenderer {
    fn default() -> Self {
        Self { aspect_ratio: 2.0 }
    }
}
