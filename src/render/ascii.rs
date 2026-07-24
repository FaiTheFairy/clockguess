use std::f64::consts::TAU;

use crossterm::style::{Attribute, Color, ContentStyle, Stylize};

use crate::{
    cli::ThemeChoice,
    clock::ClockTime,
    render::{
        ClockRenderer,
        canvas::{Canvas, Point, TextAlign},
    },
};

const DEFAULT_TERMINAL_CELL_ASPECT_RATIO: f64 = 2.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct AsciiTheme {
    pub(crate) minute_tick_ch: char,
    pub(crate) minute_tick_style: ContentStyle,

    pub(crate) hour_number_radius: f64,
    pub(crate) hour_number_style: ContentStyle,

    pub(crate) hour_hand_ch: char,
    pub(crate) hour_hand_style: ContentStyle,
    pub(crate) hour_hand_length: f64,

    pub(crate) minute_hand_ch: char,
    pub(crate) minute_hand_style: ContentStyle,
    pub(crate) minute_hand_length: f64,

    pub(crate) second_hand_ch: char,
    pub(crate) second_hand_style: ContentStyle,
    pub(crate) second_hand_length: f64,

    pub(crate) center_ch: char,
    pub(crate) center_style: ContentStyle,
}

impl AsciiTheme {
    pub fn classic() -> Self {
        Self::default()
    }

    pub fn monochrome() -> Self {
        Self {
            minute_tick_style: ContentStyle::default(),
            hour_number_style: ContentStyle::default(),
            hour_hand_style: ContentStyle::default(),
            minute_hand_style: ContentStyle::default(),
            second_hand_style: ContentStyle::default(),
            center_style: ContentStyle::default(),

            ..Self::default()
        }
    }

    pub fn unicode() -> Self {
        Self {
            minute_tick_ch: '.',
            hour_hand_ch: '█',
            minute_hand_ch: '█',
            second_hand_ch: '│',
            center_ch: '●',

            ..Self::default()
        }
    }
}

impl Default for AsciiTheme {
    fn default() -> Self {
        Self {
            minute_tick_ch: '.',
            minute_tick_style: ContentStyle::default().with(Color::DarkGrey),

            hour_number_radius: 1.0,
            hour_number_style: ContentStyle::default()
                .with(Color::White)
                .attribute(Attribute::Bold),

            hour_hand_ch: '#',
            hour_hand_style: ContentStyle::default()
                .with(Color::Yellow)
                .attribute(Attribute::Bold),
            hour_hand_length: 0.5,

            minute_hand_ch: '=',
            minute_hand_style: ContentStyle::default()
                .with(Color::Cyan)
                .attribute(Attribute::Bold),
            minute_hand_length: 0.75,

            second_hand_ch: '-',
            second_hand_style: ContentStyle::default().with(Color::Red),
            second_hand_length: 0.95,

            center_ch: '@',
            center_style: ContentStyle::default()
                .with(Color::White)
                .attribute(Attribute::Bold),
        }
    }
}

impl From<ThemeChoice> for AsciiTheme {
    fn from(value: ThemeChoice) -> Self {
        match value {
            ThemeChoice::Classic => Self::classic(),
            ThemeChoice::Monochrome => Self::monochrome(),
            ThemeChoice::Unicode => Self::unicode(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AsciiRenderer {
    /// Ratio of terminal cell height over width.
    aspect_ratio: f64,
    theme: AsciiTheme,
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
    pub fn new(theme: AsciiTheme) -> Self {
        Self::validate_length(theme.hour_hand_length, "hour hand length");
        Self::validate_length(theme.minute_hand_length, "minute hand length");
        Self::validate_length(theme.second_hand_length, "second hand length");
        Self::validate_length(theme.hour_number_radius, "hour number radius");

        Self {
            aspect_ratio: DEFAULT_TERMINAL_CELL_ASPECT_RATIO,
            theme,
        }
    }

    fn validate_length(length: f64, name: &str) {
        assert!(
            length.is_finite() && (0.0..=1.0).contains(&length),
            "{name} must be finite and between 0.0 and 1.0"
        );
    }

    fn draw_face(&self, canvas: &mut Canvas, layout: ClockLayout) {
        for index in 0..60 {
            let angle = f64::from(index) / 60.0 * TAU;
            let point = layout.point_at(angle, 1.0);

            let ch = self.theme.minute_tick_ch;
            let style = self.theme.minute_tick_style;

            if index % 5 != 0 {
                canvas.set_styled(point, ch, style);
            }
        }
    }

    fn draw_numbers(&self, canvas: &mut Canvas, layout: ClockLayout) {
        for hour in 1..=12 {
            let angle = f64::from(hour) / 12.0 * TAU;
            let point = layout.point_at(angle, self.theme.hour_number_radius);
            let text = hour.to_string();

            canvas.text_aligned(
                point,
                &text,
                self.theme.hour_number_style,
                TextAlign::Center,
            );
        }
    }

    fn draw_hour_hand(&self, canvas: &mut Canvas, layout: ClockLayout, time: ClockTime) {
        canvas.line_styled(
            layout.center(),
            layout.point_at(time.hour_angle(), self.theme.hour_hand_length),
            self.theme.hour_hand_ch,
            self.theme.hour_hand_style,
        );
    }

    fn draw_minute_hand(&self, canvas: &mut Canvas, layout: ClockLayout, time: ClockTime) {
        canvas.line_styled(
            layout.center(),
            layout.point_at(time.minute_angle(), self.theme.minute_hand_length),
            self.theme.minute_hand_ch,
            self.theme.minute_hand_style,
        );
    }

    fn draw_second_hand(&self, canvas: &mut Canvas, layout: ClockLayout, time: ClockTime) {
        canvas.line_styled(
            layout.center(),
            layout.point_at(time.second_angle(), self.theme.second_hand_length),
            self.theme.second_hand_ch,
            self.theme.second_hand_style,
        );
    }

    fn draw_center(&self, canvas: &mut Canvas, layout: ClockLayout) {
        let point = layout.center();

        canvas.set_styled(point, self.theme.center_ch, self.theme.center_style);
    }
}

impl ClockRenderer for AsciiRenderer {
    type Output = String;

    fn render(&self, time: ClockTime, width: u16, height: u16, show_seconds: bool) -> String {
        let mut canvas = Canvas::new(width, height);
        let layout = ClockLayout::new(width, height, self.aspect_ratio);

        self.draw_face(&mut canvas, layout);
        self.draw_numbers(&mut canvas, layout);
        self.draw_hour_hand(&mut canvas, layout, time);
        self.draw_minute_hand(&mut canvas, layout, time);
        if show_seconds {
            self.draw_second_hand(&mut canvas, layout, time);
        }
        // Redraw the center so the pivot remains visible.
        self.draw_center(&mut canvas, layout);

        canvas.render()
    }
}

impl Default for AsciiRenderer {
    fn default() -> Self {
        Self {
            aspect_ratio: DEFAULT_TERMINAL_CELL_ASPECT_RATIO,
            theme: AsciiTheme::default(),
        }
    }
}
