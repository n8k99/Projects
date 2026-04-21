//! Turtle Graphics — the Rosetta Stone finale.
//!
//! Seventeenth Graphics project, closing the milestone. Distinctive moves:
//!
//!   * **Turtle state with integer-degree heading** — `Turtle { x, y,
//!     heading: u16 (0..360), pen_down, pen_color, pen_width }`. No
//!     floats anywhere; heading wraps via `% 360`.
//!   * **Integer-scaled sin/cos table** — 360-entry table, values
//!     scaled by 10000. `dx = forward * cos_table[heading] / 10000`.
//!     Byte-identical trig across all six languages.
//!   * **Program as data + interpreter** — `Command` ADT with Forward,
//!     Backward, Turn, PenUp/Down, SetColor, PushState, PopState.
//!     `execute(canvas, program)` is a fold: commands in, canvas lines
//!     accumulated.
//!   * **State stack for nested procedures** — PushState snapshots the
//!     turtle; PopState restores. Logo's time-honored abstraction for
//!     "branch off, come back to where I was".
//!   * **Vector-line canvas** — lines are `(x1, y1, x2, y2, color,
//!     width)` records, not rasterized pixels. Unlike G116/G123/G129's
//!     pixel buffers, G130 stays in vector space for the whole program
//!     and only rasterizes on render.

use super::grayscale::{Image, Rgb};

/// sin * 10000 for angles 0..=359 degrees. Integer-scaled so no floats
/// leak across the language boundary. Populated at compile time via
/// `const fn`-equivalent iteration.
pub fn sin_table(angle: u16) -> i32 {
    // We use a 90-degree lookup and sign-flip the other quadrants.
    // Values computed offline: sin(0..=90) * 10000 rounded.
    static SIN_0_90: [i32; 91] = [
        0, 175, 349, 523, 698, 872, 1045, 1219, 1392, 1564,
        1736, 1908, 2079, 2250, 2419, 2588, 2756, 2924, 3090, 3256,
        3420, 3584, 3746, 3907, 4067, 4226, 4384, 4540, 4695, 4848,
        5000, 5150, 5299, 5446, 5592, 5736, 5878, 6018, 6157, 6293,
        6428, 6561, 6691, 6820, 6947, 7071, 7193, 7314, 7431, 7547,
        7660, 7771, 7880, 7986, 8090, 8192, 8290, 8387, 8480, 8572,
        8660, 8746, 8829, 8910, 8988, 9063, 9135, 9205, 9272, 9336,
        9397, 9455, 9511, 9563, 9613, 9659, 9703, 9744, 9781, 9816,
        9848, 9877, 9903, 9925, 9945, 9962, 9976, 9986, 9994, 9998,
        10000,
    ];
    let a = angle % 360;
    match a {
        0..=90 => SIN_0_90[a as usize],
        91..=180 => SIN_0_90[(180 - a) as usize],
        181..=270 => -SIN_0_90[(a - 180) as usize],
        _ => -SIN_0_90[(360 - a) as usize],
    }
}

pub fn cos_table(angle: u16) -> i32 {
    // cos(a) = sin(a + 90)
    sin_table((angle + 90) % 360)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turtle {
    pub x: i32,
    pub y: i32,
    pub heading: u16, // degrees, 0..=359. 0 = east, increases clockwise in screen coords
    pub pen_down: bool,
    pub pen_color: Rgb,
    pub pen_width: u32,
}

impl Default for Turtle {
    fn default() -> Self {
        Self {
            x: 0, y: 0, heading: 0, pen_down: true,
            pen_color: Rgb::BLACK, pen_width: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub x1: i32, pub y1: i32,
    pub x2: i32, pub y2: i32,
    pub color: Rgb,
    pub width: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Forward(i32),   // positive = forward, negative = backward
    Backward(i32),  // syntactic sugar
    Turn(i32),      // positive = clockwise (right); negative = ccw (left)
    SetHeading(u16),
    PenUp,
    PenDown,
    SetColor(Rgb),
    SetPenWidth(u32),
    PushState,
    PopState,
    GoTo { x: i32, y: i32 },
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub lines: Vec<Line>,
    pub turtle: Turtle,
    pub state_stack: Vec<Turtle>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            lines: vec![], turtle: Turtle::default(), state_stack: vec![],
        }
    }

    pub fn from_turtle(t: Turtle) -> Self {
        Self { lines: vec![], turtle: t, state_stack: vec![] }
    }

    /// Execute a single command.
    pub fn step(&mut self, cmd: &Command) {
        match *cmd {
            Command::Forward(n) => self.move_by(n),
            Command::Backward(n) => self.move_by(-n),
            Command::Turn(deg) => {
                let new_heading = (self.turtle.heading as i32 + deg).rem_euclid(360) as u16;
                self.turtle.heading = new_heading;
            }
            Command::SetHeading(h) => self.turtle.heading = h % 360,
            Command::PenUp => self.turtle.pen_down = false,
            Command::PenDown => self.turtle.pen_down = true,
            Command::SetColor(c) => self.turtle.pen_color = c,
            Command::SetPenWidth(w) => self.turtle.pen_width = w,
            Command::PushState => self.state_stack.push(self.turtle),
            Command::PopState => {
                if let Some(s) = self.state_stack.pop() {
                    self.turtle = s;
                }
            }
            Command::GoTo { x, y } => {
                let old_x = self.turtle.x;
                let old_y = self.turtle.y;
                self.turtle.x = x;
                self.turtle.y = y;
                if self.turtle.pen_down {
                    self.lines.push(Line {
                        x1: old_x, y1: old_y, x2: x, y2: y,
                        color: self.turtle.pen_color,
                        width: self.turtle.pen_width,
                    });
                }
            }
        }
    }

    fn move_by(&mut self, distance: i32) {
        let cos = cos_table(self.turtle.heading) as i64;
        let sin = sin_table(self.turtle.heading) as i64;
        let new_x = self.turtle.x + ((distance as i64 * cos) / 10000) as i32;
        let new_y = self.turtle.y + ((distance as i64 * sin) / 10000) as i32;
        if self.turtle.pen_down {
            self.lines.push(Line {
                x1: self.turtle.x, y1: self.turtle.y,
                x2: new_x, y2: new_y,
                color: self.turtle.pen_color,
                width: self.turtle.pen_width,
            });
        }
        self.turtle.x = new_x;
        self.turtle.y = new_y;
    }

    /// Execute a program: a list of commands.
    pub fn execute(&mut self, program: &[Command]) {
        for cmd in program {
            self.step(cmd);
        }
    }

    /// Render the line list to a pixel image. Simple stamped-disk line.
    pub fn render(&self, width: u32, height: u32, bg: Rgb) -> Image {
        let mut img = Image::filled(width, height, bg);
        for line in &self.lines {
            stamp_line(&mut img, line.x1, line.y1, line.x2, line.y2,
                       line.width, line.color);
        }
        img
    }
}

fn stamp_disk(img: &mut Image, cx: i32, cy: i32, radius: u32, color: Rgb) {
    let r = radius as i32;
    let r2 = (r * r) as i64;
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = dx as i64 * dx as i64 + dy as i64 * dy as i64;
            if d2 > r2 { continue; }
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 { continue; }
            if (x as u32) >= img.width || (y as u32) >= img.height { continue; }
            img.set(x as u32, y as u32, color);
        }
    }
}

fn stamp_line(img: &mut Image, x1: i32, y1: i32, x2: i32, y2: i32,
               width: u32, color: Rgb) {
    let radius = (width / 2).max(1);
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x1;
    let mut y = y1;
    loop {
        stamp_disk(img, x, y, radius, color);
        if x == x2 && y == y2 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_cos_key_angles() {
        assert_eq!(sin_table(0), 0);
        assert_eq!(cos_table(0), 10000);
        assert_eq!(sin_table(90), 10000);
        assert_eq!(cos_table(90), 0);
        assert_eq!(sin_table(180), 0);
        assert_eq!(cos_table(180), -10000);
        assert_eq!(sin_table(270), -10000);
        assert_eq!(cos_table(270), 0);
    }

    #[test]
    fn sin_wraps_at_360() {
        assert_eq!(sin_table(360), sin_table(0));
        assert_eq!(sin_table(450), sin_table(90));
    }

    #[test]
    fn forward_east_moves_x() {
        let mut c = Canvas::new();
        c.execute(&[Command::Forward(100)]);
        assert_eq!(c.turtle.x, 100);
        assert_eq!(c.turtle.y, 0);
        assert_eq!(c.lines.len(), 1);
    }

    #[test]
    fn forward_north_moves_y() {
        let mut c = Canvas::new();
        c.execute(&[Command::Turn(90), Command::Forward(50)]);
        assert_eq!(c.turtle.x, 0);
        assert_eq!(c.turtle.y, 50);
    }

    #[test]
    fn turn_full_circle_is_identity() {
        let mut c = Canvas::new();
        c.execute(&[Command::Turn(360)]);
        assert_eq!(c.turtle.heading, 0);
    }

    #[test]
    fn negative_turn_goes_counterclockwise() {
        let mut c = Canvas::new();
        c.execute(&[Command::Turn(-90)]);
        assert_eq!(c.turtle.heading, 270);
    }

    #[test]
    fn pen_up_no_line_drawn() {
        let mut c = Canvas::new();
        c.execute(&[Command::PenUp, Command::Forward(50)]);
        assert!(c.lines.is_empty());
        assert_eq!(c.turtle.x, 50);
    }

    #[test]
    fn pen_down_after_up_draws_from_new_position() {
        let mut c = Canvas::new();
        c.execute(&[
            Command::PenUp,
            Command::Forward(50),
            Command::PenDown,
            Command::Forward(50),
        ]);
        assert_eq!(c.lines.len(), 1);
        assert_eq!(c.lines[0].x1, 50);
        assert_eq!(c.lines[0].x2, 100);
    }

    #[test]
    fn backward_moves_opposite() {
        let mut c = Canvas::new();
        c.execute(&[Command::Backward(30)]);
        assert_eq!(c.turtle.x, -30);
    }

    #[test]
    fn square_program_closes() {
        let mut c = Canvas::new();
        let program = vec![
            Command::Forward(100), Command::Turn(90),
            Command::Forward(100), Command::Turn(90),
            Command::Forward(100), Command::Turn(90),
            Command::Forward(100), Command::Turn(90),
        ];
        c.execute(&program);
        assert_eq!(c.turtle.x, 0);
        assert_eq!(c.turtle.y, 0);
        assert_eq!(c.turtle.heading, 0);
        assert_eq!(c.lines.len(), 4);
    }

    #[test]
    fn state_stack_push_pop_restores_turtle() {
        let mut c = Canvas::new();
        c.execute(&[
            Command::Forward(50),
            Command::PushState,
            Command::Turn(90),
            Command::Forward(30),
            Command::PopState,
        ]);
        // After pop, turtle is back at (50, 0) heading 0.
        assert_eq!(c.turtle.x, 50);
        assert_eq!(c.turtle.y, 0);
        assert_eq!(c.turtle.heading, 0);
        // But lines from the branch are preserved
        assert_eq!(c.lines.len(), 2);
    }

    #[test]
    fn nested_push_pop_is_stack() {
        let mut c = Canvas::new();
        c.execute(&[
            Command::Turn(45),
            Command::PushState,
            Command::Turn(45),    // total 90 now
            Command::PushState,
            Command::Turn(90),    // total 180
            Command::PopState,    // back to 90
            Command::PopState,    // back to 45
        ]);
        assert_eq!(c.turtle.heading, 45);
    }

    #[test]
    fn pop_on_empty_stack_is_noop() {
        let mut c = Canvas::new();
        c.turtle.heading = 42;
        c.execute(&[Command::PopState]);
        // Turtle unchanged
        assert_eq!(c.turtle.heading, 42);
    }

    #[test]
    fn goto_teleports_and_draws_if_pen_down() {
        let mut c = Canvas::new();
        c.execute(&[Command::GoTo { x: 100, y: 200 }]);
        assert_eq!(c.turtle.x, 100);
        assert_eq!(c.turtle.y, 200);
        assert_eq!(c.lines.len(), 1);
    }

    #[test]
    fn goto_no_draw_when_pen_up() {
        let mut c = Canvas::new();
        c.execute(&[Command::PenUp, Command::GoTo { x: 100, y: 200 }]);
        assert!(c.lines.is_empty());
    }

    #[test]
    fn set_color_applies_to_next_line() {
        let mut c = Canvas::new();
        c.execute(&[
            Command::Forward(10),
            Command::SetColor(Rgb::new(255, 0, 0)),
            Command::Forward(10),
        ]);
        assert_eq!(c.lines[0].color, Rgb::BLACK);
        assert_eq!(c.lines[1].color, Rgb::new(255, 0, 0));
    }

    #[test]
    fn render_produces_colored_pixels() {
        let mut c = Canvas::new();
        c.turtle.x = 5; c.turtle.y = 5;
        c.execute(&[Command::SetColor(Rgb::new(255, 0, 0)), Command::Forward(10)]);
        let img = c.render(20, 20, Rgb::WHITE);
        let has_red = img.pixels.iter().any(|p| *p == Rgb::new(255, 0, 0));
        assert!(has_red);
    }

    #[test]
    fn square_45_degrees_rotated() {
        // Draw a square rotated 45°; endpoint should return to origin
        let mut c = Canvas::new();
        let program = vec![
            Command::Turn(45),
            Command::Forward(100), Command::Turn(90),
            Command::Forward(100), Command::Turn(90),
            Command::Forward(100), Command::Turn(90),
            Command::Forward(100), Command::Turn(90),
        ];
        c.execute(&program);
        // After closing the square, turtle is back at origin (within integer rounding)
        assert!(c.turtle.x.abs() <= 2);
        assert!(c.turtle.y.abs() <= 2);
    }
}
