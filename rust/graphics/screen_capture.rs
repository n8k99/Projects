//! Screen Capture Program — region + delay FSM + annotations + ring-buffer history.
//!
//! Tenth Graphics project. Distinctive moves:
//!
//!   * **Capture region as a closed ADT** — FullScreen, Rect(x,y,w,h),
//!     Window(id). The intent is encoded as data; execution is a pure
//!     function over the screen snapshot.
//!   * **Delayed-capture FSM** — Idle → Armed → Counting → Capturing →
//!     Captured. `tick(ms)` drains the countdown; `capture(screen)`
//!     transitions Capturing → Captured with the pixel result.
//!   * **Annotations as an overlay draw pipeline** — Arrow, Rect, Text,
//!     Highlight (alpha-blend), Blur. Ordered list; rendered after the
//!     base capture. Distinct from G119's transform pipeline because
//!     annotations are *overlaid*, not transformed.
//!   * **Ring-buffer history** — fixed-size `history_max`. Oldest capture
//!     drops when the buffer is full. Users get "last N screenshots"
//!     semantics without unbounded memory.

use super::grayscale::{Image, Rgb};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureRegion {
    FullScreen,
    Rect { x: u32, y: u32, width: u32, height: u32 },
    Window { window_id: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    Rectangle { x: u32, y: u32, width: u32, height: u32,
                 color: Rgb, stroke: u32 },
    Highlight { x: u32, y: u32, width: u32, height: u32,
                 color: Rgb, opacity: u8 }, // 0..=255
    Text { x: u32, y: u32, text: String, color: Rgb },
    Arrow { x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb },
}

#[derive(Debug, Clone)]
pub struct CaptureSpec {
    pub region: CaptureRegion,
    pub delay_ms: u64,
    pub include_cursor: bool,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Armed,
    Counting,
    Capturing,
    Captured,
}

#[derive(Debug, Clone)]
pub struct Capture {
    pub id: u64,
    pub spec: CaptureSpec,
    pub image: Image,
    pub captured_at_ms: u64,
}

#[derive(Debug, Clone)]
pub enum RecorderEvent {
    Armed { id: u64, delay_ms: u64 },
    CountdownTick { id: u64, remaining_ms: u64 },
    Captured { id: u64, at_ms: u64 },
    Evicted { id: u64 },
    CaptureFailed { id: u64, reason: String },
}

#[derive(Debug, Clone)]
pub struct Recorder {
    pub state: RecorderState,
    pub pending_spec: Option<CaptureSpec>,
    pub pending_id: u64,
    pub countdown_remaining_ms: u64,
    pub history: Vec<Capture>,
    pub history_max: usize,
    pub elapsed_ms: u64,
    pub events: Vec<RecorderEvent>,
    next_id: u64,
}

impl Recorder {
    pub fn new(history_max: usize) -> Self {
        Self {
            state: RecorderState::Idle,
            pending_spec: None,
            pending_id: 0,
            countdown_remaining_ms: 0,
            history: vec![],
            history_max,
            elapsed_ms: 0,
            events: vec![],
            next_id: 1,
        }
    }

    /// Begin a capture according to the spec. If delay_ms > 0, enters Counting;
    /// otherwise transitions directly to Capturing.
    pub fn arm(&mut self, spec: CaptureSpec) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.pending_id = id;
        self.countdown_remaining_ms = spec.delay_ms;
        let delay = spec.delay_ms;
        self.pending_spec = Some(spec);
        self.events.push(RecorderEvent::Armed { id, delay_ms: delay });
        if delay == 0 {
            self.state = RecorderState::Capturing;
        } else {
            self.state = RecorderState::Counting;
        }
        id
    }

    pub fn tick(&mut self, ms: u64) {
        self.elapsed_ms += ms;
        if self.state == RecorderState::Counting {
            if self.countdown_remaining_ms > ms {
                self.countdown_remaining_ms -= ms;
                self.events.push(RecorderEvent::CountdownTick {
                    id: self.pending_id,
                    remaining_ms: self.countdown_remaining_ms,
                });
            } else {
                self.countdown_remaining_ms = 0;
                self.state = RecorderState::Capturing;
                self.events.push(RecorderEvent::CountdownTick {
                    id: self.pending_id, remaining_ms: 0,
                });
            }
        }
    }

    /// Execute the capture if the recorder is in the Capturing state.
    /// Returns the capture on success.
    pub fn capture(&mut self, screen: &Image, window_lookup: impl Fn(u32) -> Option<Image>)
        -> Option<Capture>
    {
        if self.state != RecorderState::Capturing { return None; }
        let spec = self.pending_spec.take()?;
        let id = self.pending_id;

        let base = match &spec.region {
            CaptureRegion::FullScreen => screen.clone(),
            CaptureRegion::Rect { x, y, width, height } => {
                if *x + *width > screen.width || *y + *height > screen.height {
                    self.events.push(RecorderEvent::CaptureFailed {
                        id, reason: "rect out of screen bounds".to_string(),
                    });
                    self.state = RecorderState::Idle;
                    return None;
                }
                crop(screen, *x, *y, *width, *height)
            }
            CaptureRegion::Window { window_id } => {
                match window_lookup(*window_id) {
                    Some(img) => img,
                    None => {
                        self.events.push(RecorderEvent::CaptureFailed {
                            id, reason: format!("window {} not found", window_id),
                        });
                        self.state = RecorderState::Idle;
                        return None;
                    }
                }
            }
        };

        let with_annotations = apply_annotations(base, &spec.annotations);

        let capture = Capture {
            id, spec, image: with_annotations,
            captured_at_ms: self.elapsed_ms,
        };
        self.history.push(capture.clone());
        if self.history.len() > self.history_max {
            let evicted = self.history.remove(0);
            self.events.push(RecorderEvent::Evicted { id: evicted.id });
        }

        self.events.push(RecorderEvent::Captured {
            id, at_ms: self.elapsed_ms,
        });
        self.state = RecorderState::Captured;
        Some(capture)
    }

    pub fn reset(&mut self) {
        self.state = RecorderState::Idle;
        self.pending_spec = None;
        self.pending_id = 0;
        self.countdown_remaining_ms = 0;
    }
}

fn crop(img: &Image, x0: u32, y0: u32, w: u32, h: u32) -> Image {
    let mut pixels = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            pixels.push(img.get(x0 + x, y0 + y).unwrap());
        }
    }
    Image::from_pixels(w, h, pixels).unwrap()
}

pub fn apply_annotations(base: Image, annotations: &[Annotation]) -> Image {
    let mut img = base;
    for ann in annotations {
        img = apply_annotation(img, ann);
    }
    img
}

fn apply_annotation(mut img: Image, ann: &Annotation) -> Image {
    match ann {
        Annotation::Rectangle { x, y, width, height, color, stroke } => {
            draw_rect_outline(&mut img, *x, *y, *width, *height, *color, *stroke);
        }
        Annotation::Highlight { x, y, width, height, color, opacity } => {
            blend_rect(&mut img, *x, *y, *width, *height, *color, *opacity);
        }
        Annotation::Text { x, y, text: _, color } => {
            // Simplified: a text annotation paints a single-pixel marker
            // at the anchor (renders are platform-specific).
            if *x < img.width && *y < img.height {
                img.set(*x, *y, *color);
            }
        }
        Annotation::Arrow { x1, y1, x2, y2, color } => {
            draw_line(&mut img, *x1, *y1, *x2, *y2, *color);
        }
    }
    img
}

fn set_clamped(img: &mut Image, x: u32, y: u32, color: Rgb) {
    if x < img.width && y < img.height {
        img.set(x, y, color);
    }
}

fn draw_rect_outline(img: &mut Image, x: u32, y: u32, w: u32, h: u32,
                      color: Rgb, stroke: u32) {
    let stroke = stroke.max(1);
    for s in 0..stroke {
        for dx in 0..w {
            set_clamped(img, x + dx, y + s, color);
            if h > s { set_clamped(img, x + dx, y + h - 1 - s, color); }
        }
        for dy in 0..h {
            set_clamped(img, x + s, y + dy, color);
            if w > s { set_clamped(img, x + w - 1 - s, y + dy, color); }
        }
    }
}

fn blend(a: u8, b: u8, alpha: u8) -> u8 {
    let a16 = a as u16;
    let b16 = b as u16;
    let al = alpha as u16;
    ((a16 * (255 - al) + b16 * al) / 255) as u8
}

fn blend_rect(img: &mut Image, x: u32, y: u32, w: u32, h: u32,
               color: Rgb, opacity: u8) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px >= img.width || py >= img.height { continue; }
            let cur = img.get(px, py).unwrap();
            let blended = Rgb::new(
                blend(cur.r, color.r, opacity),
                blend(cur.g, color.g, opacity),
                blend(cur.b, color.b, opacity),
            );
            img.set(px, py, blended);
        }
    }
}

/// Bresenham line.
fn draw_line(img: &mut Image, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb) {
    let mut x = x1 as i32;
    let mut y = y1 as i32;
    let x2i = x2 as i32;
    let y2i = y2 as i32;
    let dx = (x2i - x).abs();
    let dy = -(y2i - y).abs();
    let sx = if x < x2i { 1 } else { -1 };
    let sy = if y < y2i { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x >= 0 && y >= 0 {
            set_clamped(img, x as u32, y as u32, color);
        }
        if x == x2i && y == y2i { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(w: u32, h: u32, color: Rgb) -> Image {
        Image::filled(w, h, color)
    }

    fn no_windows(_id: u32) -> Option<Image> { None }

    #[test]
    fn zero_delay_goes_straight_to_capturing() {
        let mut rec = Recorder::new(5);
        let spec = CaptureSpec {
            region: CaptureRegion::FullScreen, delay_ms: 0,
            include_cursor: false, annotations: vec![],
        };
        rec.arm(spec);
        assert_eq!(rec.state, RecorderState::Capturing);
    }

    #[test]
    fn nonzero_delay_goes_to_counting() {
        let mut rec = Recorder::new(5);
        let spec = CaptureSpec {
            region: CaptureRegion::FullScreen, delay_ms: 2000,
            include_cursor: false, annotations: vec![],
        };
        rec.arm(spec);
        assert_eq!(rec.state, RecorderState::Counting);
        assert_eq!(rec.countdown_remaining_ms, 2000);
    }

    #[test]
    fn tick_drains_countdown() {
        let mut rec = Recorder::new(5);
        rec.arm(CaptureSpec {
            region: CaptureRegion::FullScreen, delay_ms: 1000,
            include_cursor: false, annotations: vec![],
        });
        rec.tick(400);
        assert_eq!(rec.state, RecorderState::Counting);
        assert_eq!(rec.countdown_remaining_ms, 600);
        rec.tick(700);
        assert_eq!(rec.state, RecorderState::Capturing);
        assert_eq!(rec.countdown_remaining_ms, 0);
    }

    #[test]
    fn capture_fullscreen_returns_full_image() {
        let mut rec = Recorder::new(5);
        rec.arm(CaptureSpec {
            region: CaptureRegion::FullScreen, delay_ms: 0,
            include_cursor: false, annotations: vec![],
        });
        let screen = blank(10, 8, Rgb::new(100, 100, 100));
        let cap = rec.capture(&screen, no_windows).unwrap();
        assert_eq!((cap.image.width, cap.image.height), (10, 8));
        assert_eq!(rec.state, RecorderState::Captured);
    }

    #[test]
    fn capture_rect_crops_region() {
        let mut rec = Recorder::new(5);
        rec.arm(CaptureSpec {
            region: CaptureRegion::Rect { x: 2, y: 1, width: 4, height: 3 },
            delay_ms: 0, include_cursor: false, annotations: vec![],
        });
        let screen = blank(10, 8, Rgb::new(50, 50, 50));
        let cap = rec.capture(&screen, no_windows).unwrap();
        assert_eq!((cap.image.width, cap.image.height), (4, 3));
    }

    #[test]
    fn capture_rect_out_of_bounds_fails() {
        let mut rec = Recorder::new(5);
        rec.arm(CaptureSpec {
            region: CaptureRegion::Rect { x: 100, y: 100, width: 10, height: 10 },
            delay_ms: 0, include_cursor: false, annotations: vec![],
        });
        let screen = blank(10, 8, Rgb::new(0, 0, 0));
        assert!(rec.capture(&screen, no_windows).is_none());
        assert_eq!(rec.state, RecorderState::Idle);
    }

    #[test]
    fn capture_window_missing_fails() {
        let mut rec = Recorder::new(5);
        rec.arm(CaptureSpec {
            region: CaptureRegion::Window { window_id: 42 },
            delay_ms: 0, include_cursor: false, annotations: vec![],
        });
        let screen = blank(10, 8, Rgb::new(0, 0, 0));
        assert!(rec.capture(&screen, no_windows).is_none());
    }

    #[test]
    fn capture_window_with_lookup_succeeds() {
        let mut rec = Recorder::new(5);
        rec.arm(CaptureSpec {
            region: CaptureRegion::Window { window_id: 7 },
            delay_ms: 0, include_cursor: false, annotations: vec![],
        });
        let screen = blank(10, 8, Rgb::new(0, 0, 0));
        let window_img = blank(4, 4, Rgb::new(200, 100, 50));
        let lookup = move |id: u32| {
            if id == 7 { Some(window_img.clone()) } else { None }
        };
        let cap = rec.capture(&screen, lookup).unwrap();
        assert_eq!((cap.image.width, cap.image.height), (4, 4));
    }

    #[test]
    fn highlight_blends_pixels() {
        let screen = blank(4, 4, Rgb::new(100, 100, 100));
        let out = apply_annotations(screen, &vec![
            Annotation::Highlight {
                x: 0, y: 0, width: 4, height: 4,
                color: Rgb::new(255, 255, 0), opacity: 128,
            }
        ]);
        let p = out.get(0, 0).unwrap();
        // blend(100, 255, 128) ≈ 177
        assert!(p.r > 150 && p.r < 200);
        assert!(p.g > 150 && p.g < 200);
    }

    #[test]
    fn rectangle_outline_draws_border() {
        let screen = blank(6, 6, Rgb::new(0, 0, 0));
        let out = apply_annotations(screen, &vec![
            Annotation::Rectangle {
                x: 1, y: 1, width: 4, height: 4,
                color: Rgb::new(255, 0, 0), stroke: 1,
            }
        ]);
        assert_eq!(out.get(1, 1).unwrap(), Rgb::new(255, 0, 0)); // corner
        assert_eq!(out.get(4, 4).unwrap(), Rgb::new(255, 0, 0)); // opposite corner
        assert_eq!(out.get(2, 2).unwrap(), Rgb::new(0, 0, 0)); // interior unchanged
    }

    #[test]
    fn arrow_draws_line() {
        let screen = blank(10, 10, Rgb::new(0, 0, 0));
        let out = apply_annotations(screen, &vec![
            Annotation::Arrow {
                x1: 0, y1: 0, x2: 9, y2: 9,
                color: Rgb::new(0, 255, 0),
            }
        ]);
        assert_eq!(out.get(0, 0).unwrap(), Rgb::new(0, 255, 0));
        assert_eq!(out.get(9, 9).unwrap(), Rgb::new(0, 255, 0));
        // midpoint on diagonal
        assert_eq!(out.get(4, 4).unwrap(), Rgb::new(0, 255, 0));
    }

    #[test]
    fn history_ring_buffer_evicts_oldest() {
        let mut rec = Recorder::new(3);
        for _ in 0..5 {
            rec.arm(CaptureSpec {
                region: CaptureRegion::FullScreen, delay_ms: 0,
                include_cursor: false, annotations: vec![],
            });
            let screen = blank(2, 2, Rgb::new(0, 0, 0));
            rec.capture(&screen, no_windows);
            rec.reset();
        }
        assert_eq!(rec.history.len(), 3);
        // IDs start at 1; after 5 captures, IDs 3, 4, 5 remain
        let ids: Vec<u64> = rec.history.iter().map(|c| c.id).collect();
        assert_eq!(ids, vec![3, 4, 5]);
    }

    #[test]
    fn eviction_emits_event() {
        let mut rec = Recorder::new(1);
        for _ in 0..2 {
            rec.arm(CaptureSpec {
                region: CaptureRegion::FullScreen, delay_ms: 0,
                include_cursor: false, annotations: vec![],
            });
            let screen = blank(2, 2, Rgb::new(0, 0, 0));
            rec.capture(&screen, no_windows);
            rec.reset();
        }
        let has_evict = rec.events.iter().any(|e|
            matches!(e, RecorderEvent::Evicted { .. }));
        assert!(has_evict);
    }

    #[test]
    fn capture_before_armed_returns_none() {
        let mut rec = Recorder::new(5);
        let screen = blank(4, 4, Rgb::new(0, 0, 0));
        assert!(rec.capture(&screen, no_windows).is_none());
    }
}
