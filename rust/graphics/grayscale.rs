//! Grayscale Converter — pixel buffers + named conversion algorithms + histogram.
//!
//! Third Graphics project. Introduces the **pixel buffer abstraction**
//! (a width × height × channel grid) and **named conversion strategies**.
//! Grayscale conversion is lossy — 3 channels to 1 — so which single
//! channel to derive matters. Different algorithms emphasise different
//! visual properties: Luminosity (perceptual brightness), Average
//! (mathematical mean), Lightness (max/min midpoint), or single-channel
//! isolation (Red/Green/Blue).
//!
//! Distinctive moves:
//!   * **Pixel buffer** as a `Vec<Rgb>` plus dimensions — row-major layout.
//!   * **Algorithm as enum** — five named variants; conversion dispatches.
//!   * **Luminosity coefficients** (0.2126 / 0.7152 / 0.0722 for Rec.709)
//!      are the standard perceptual formula.
//!   * **Histogram** — 256-bin frequency table of the grayscale values.
//!   * **Contrast metrics** — mean brightness and stddev from histogram.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgb>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, pixels: vec![Rgb::BLACK; (width * height) as usize] }
    }

    pub fn filled(width: u32, height: u32, color: Rgb) -> Self {
        Self { width, height, pixels: vec![color; (width * height) as usize] }
    }

    pub fn from_pixels(width: u32, height: u32, pixels: Vec<Rgb>) -> Option<Self> {
        if pixels.len() != (width * height) as usize { return None; }
        Some(Self { width, height, pixels })
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width || y >= self.height { return None; }
        Some(self.pixels[(y * self.width + x) as usize])
    }

    pub fn set(&mut self, x: u32, y: u32, pixel: Rgb) -> bool {
        if x >= self.width || y >= self.height { return false; }
        self.pixels[(y * self.width + x) as usize] = pixel;
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrayImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, pixels: vec![0; (width * height) as usize] }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<u8> {
        if x >= self.width || y >= self.height { return None; }
        Some(self.pixels[(y * self.width + x) as usize])
    }

    pub fn set(&mut self, x: u32, y: u32, value: u8) -> bool {
        if x >= self.width || y >= self.height { return false; }
        self.pixels[(y * self.width + x) as usize] = value;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// Rec.709 perceptual luminance: 0.2126 R + 0.7152 G + 0.0722 B.
    Luminosity,
    /// Arithmetic mean: (R + G + B) / 3.
    Average,
    /// Midpoint of max and min channels: (max + min) / 2.
    Lightness,
    /// Red channel only.
    Red,
    /// Green channel only.
    Green,
    /// Blue channel only.
    Blue,
}

/// Convert a single RGB pixel to a grayscale byte.
pub fn pixel_to_gray(p: Rgb, algo: Algorithm) -> u8 {
    match algo {
        Algorithm::Luminosity => {
            let v = 0.2126 * p.r as f64 + 0.7152 * p.g as f64 + 0.0722 * p.b as f64;
            v.round().clamp(0.0, 255.0) as u8
        }
        Algorithm::Average => {
            ((p.r as u32 + p.g as u32 + p.b as u32) / 3) as u8
        }
        Algorithm::Lightness => {
            let max = p.r.max(p.g).max(p.b) as u32;
            let min = p.r.min(p.g).min(p.b) as u32;
            ((max + min) / 2) as u8
        }
        Algorithm::Red => p.r,
        Algorithm::Green => p.g,
        Algorithm::Blue => p.b,
    }
}

/// Convert an entire Image to Grayscale using the given algorithm.
pub fn convert(image: &Image, algo: Algorithm) -> GrayImage {
    let pixels: Vec<u8> = image.pixels.iter()
        .map(|p| pixel_to_gray(*p, algo))
        .collect();
    GrayImage { width: image.width, height: image.height, pixels }
}

/// 256-bin histogram of grayscale values.
pub fn histogram(gray: &GrayImage) -> [u32; 256] {
    let mut bins = [0u32; 256];
    for &v in &gray.pixels { bins[v as usize] += 1; }
    bins
}

/// Mean brightness in [0, 255].
pub fn mean_brightness(gray: &GrayImage) -> f64 {
    if gray.pixels.is_empty() { return 0.0; }
    let sum: u64 = gray.pixels.iter().map(|&v| v as u64).sum();
    sum as f64 / gray.pixels.len() as f64
}

/// Standard deviation of brightness — a basic contrast proxy.
pub fn contrast_stddev(gray: &GrayImage) -> f64 {
    if gray.pixels.is_empty() { return 0.0; }
    let mean = mean_brightness(gray);
    let var: f64 = gray.pixels.iter()
        .map(|&v| {
            let d = v as f64 - mean;
            d * d
        })
        .sum::<f64>() / gray.pixels.len() as f64;
    var.sqrt()
}

/// Min and max brightness values.
pub fn dynamic_range(gray: &GrayImage) -> Option<(u8, u8)> {
    if gray.pixels.is_empty() { return None; }
    let min = *gray.pixels.iter().min().unwrap();
    let max = *gray.pixels.iter().max().unwrap();
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkerboard() -> Image {
        // 2x2: black / red / green / blue
        Image::from_pixels(2, 2, vec![
            Rgb::BLACK,
            Rgb::new(255, 0, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
        ]).unwrap()
    }

    #[test]
    fn pixel_get_set() {
        let mut img = Image::new(2, 2);
        assert_eq!(img.get(0, 0), Some(Rgb::BLACK));
        assert!(img.set(1, 1, Rgb::WHITE));
        assert_eq!(img.get(1, 1), Some(Rgb::WHITE));
    }

    #[test]
    fn pixel_out_of_bounds() {
        let img = Image::new(2, 2);
        assert_eq!(img.get(2, 0), None);
        assert_eq!(img.get(0, 2), None);
    }

    #[test]
    fn from_pixels_rejects_wrong_size() {
        assert!(Image::from_pixels(2, 2, vec![Rgb::BLACK; 3]).is_none());
    }

    #[test]
    fn luminosity_weights_green_highest() {
        // Pure red (255,0,0) should give 0.2126 * 255 ≈ 54
        let g = pixel_to_gray(Rgb::new(255, 0, 0), Algorithm::Luminosity);
        assert_eq!(g, 54);
        // Pure green (0,255,0) → 0.7152 * 255 ≈ 182
        let g = pixel_to_gray(Rgb::new(0, 255, 0), Algorithm::Luminosity);
        assert_eq!(g, 182);
        // Pure blue (0,0,255) → 0.0722 * 255 ≈ 18
        let g = pixel_to_gray(Rgb::new(0, 0, 255), Algorithm::Luminosity);
        assert_eq!(g, 18);
    }

    #[test]
    fn average_takes_arithmetic_mean() {
        let g = pixel_to_gray(Rgb::new(30, 60, 90), Algorithm::Average);
        assert_eq!(g, 60);
    }

    #[test]
    fn lightness_is_max_plus_min_halved() {
        let g = pixel_to_gray(Rgb::new(10, 200, 100), Algorithm::Lightness);
        assert_eq!(g, 105);  // (200 + 10) / 2
    }

    #[test]
    fn single_channel_algorithms() {
        let p = Rgb::new(10, 20, 30);
        assert_eq!(pixel_to_gray(p, Algorithm::Red), 10);
        assert_eq!(pixel_to_gray(p, Algorithm::Green), 20);
        assert_eq!(pixel_to_gray(p, Algorithm::Blue), 30);
    }

    #[test]
    fn convert_preserves_dimensions() {
        let img = checkerboard();
        let g = convert(&img, Algorithm::Luminosity);
        assert_eq!(g.width, img.width);
        assert_eq!(g.height, img.height);
        assert_eq!(g.pixels.len(), img.pixels.len());
    }

    #[test]
    fn convert_black_to_zero() {
        let img = Image::filled(3, 3, Rgb::BLACK);
        for algo in [Algorithm::Luminosity, Algorithm::Average, Algorithm::Lightness] {
            let g = convert(&img, algo);
            assert!(g.pixels.iter().all(|&v| v == 0));
        }
    }

    #[test]
    fn convert_white_to_255() {
        let img = Image::filled(3, 3, Rgb::WHITE);
        for algo in [Algorithm::Luminosity, Algorithm::Average, Algorithm::Lightness] {
            let g = convert(&img, algo);
            assert!(g.pixels.iter().all(|&v| v == 255));
        }
    }

    #[test]
    fn histogram_counts_each_value() {
        let img = Image::filled(10, 10, Rgb::new(128, 128, 128));
        let g = convert(&img, Algorithm::Average);
        let bins = histogram(&g);
        assert_eq!(bins[128], 100);
        assert_eq!(bins.iter().sum::<u32>(), 100);
    }

    #[test]
    fn mean_brightness_of_uniform() {
        let img = Image::filled(5, 5, Rgb::new(100, 100, 100));
        let g = convert(&img, Algorithm::Average);
        let m = mean_brightness(&g);
        assert!((m - 100.0).abs() < 0.01);
    }

    #[test]
    fn contrast_stddev_is_zero_for_uniform() {
        let img = Image::filled(5, 5, Rgb::new(128, 128, 128));
        let g = convert(&img, Algorithm::Average);
        assert_eq!(contrast_stddev(&g), 0.0);
    }

    #[test]
    fn contrast_stddev_is_positive_for_mixed() {
        let img = checkerboard();
        let g = convert(&img, Algorithm::Luminosity);
        assert!(contrast_stddev(&g) > 0.0);
    }

    #[test]
    fn dynamic_range_of_checkerboard() {
        let img = checkerboard();
        let g = convert(&img, Algorithm::Luminosity);
        let (min, max) = dynamic_range(&g).unwrap();
        assert_eq!(min, 0);        // black pixel
        assert_eq!(max, 182);      // green pixel
    }

    #[test]
    fn empty_image_has_zero_metrics() {
        let img = Image::filled(0, 0, Rgb::BLACK);
        let g = convert(&img, Algorithm::Average);
        assert_eq!(mean_brightness(&g), 0.0);
        assert_eq!(contrast_stddev(&g), 0.0);
        assert!(dynamic_range(&g).is_none());
    }

    #[test]
    fn different_algorithms_differ_on_pure_green() {
        let img = Image::filled(1, 1, Rgb::new(0, 255, 0));
        let lum = convert(&img, Algorithm::Luminosity).pixels[0];
        let avg = convert(&img, Algorithm::Average).pixels[0];
        let lit = convert(&img, Algorithm::Lightness).pixels[0];
        assert_ne!(lum, avg);
        assert_ne!(avg, lit);
    }
}
