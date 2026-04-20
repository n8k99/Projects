//! Bulk Picture Manipulator — transformation pipeline applied over a batch.
//!
//! Sixth Graphics project. Builds on G116 (pixel buffer), G068 (bulk
//! thumbnail), G092 (bulk renamer). Distinctive moves:
//!
//!   * **Pipeline as data** — `Op` enum; a `Pipeline` is `Vec<Op>`.
//!     Serializable, previewable, replayable. Same recipe on same image
//!     always yields same output.
//!   * **Dry run over dimensions** — a pipeline's effect on image dims
//!     can be computed without touching pixels. Lets the UI preview
//!     "what will this pipeline do" cheaply.
//!   * **Per-image error isolation** — one malformed image doesn't abort
//!     the batch. Each input yields a `BatchResult::Ok | Err`.
//!   * **Ops are pure** — `apply(&self, img) -> Result<Image, OpError>`.
//!     Pipeline is just a fold over ops.

use super::grayscale::{Image, Rgb, Algorithm, pixel_to_gray};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Resize { width: u32, height: u32 },
    Grayscale(Algorithm),
    Rotate90,
    Rotate180,
    Rotate270,
    FlipHorizontal,
    FlipVertical,
    Crop { x: u32, y: u32, width: u32, height: u32 },
    Brightness(i16), // added to each channel, clamped [0, 255]
    Contrast(f32),   // factor; 1.0 = unchanged
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpError {
    CropOutOfBounds,
    ZeroDimension,
}

pub type Pipeline = Vec<Op>;

pub fn apply_op(img: &Image, op: &Op) -> Result<Image, OpError> {
    match op {
        Op::Resize { width, height } => {
            if *width == 0 || *height == 0 { return Err(OpError::ZeroDimension); }
            Ok(resize_nearest(img, *width, *height))
        }
        Op::Grayscale(method) => Ok(grayscale_image(img, *method)),
        Op::Rotate90 => Ok(rotate_90(img)),
        Op::Rotate180 => Ok(rotate_180(img)),
        Op::Rotate270 => Ok(rotate_270(img)),
        Op::FlipHorizontal => Ok(flip_h(img)),
        Op::FlipVertical => Ok(flip_v(img)),
        Op::Crop { x, y, width, height } => {
            if *width == 0 || *height == 0 { return Err(OpError::ZeroDimension); }
            if x.saturating_add(*width) > img.width
                || y.saturating_add(*height) > img.height {
                return Err(OpError::CropOutOfBounds);
            }
            Ok(crop(img, *x, *y, *width, *height))
        }
        Op::Brightness(delta) => Ok(map_pixels(img, |p| adjust_brightness(p, *delta))),
        Op::Contrast(factor) => Ok(map_pixels(img, |p| adjust_contrast(p, *factor))),
    }
}

pub fn apply_pipeline(img: &Image, pipeline: &Pipeline) -> Result<Image, OpError> {
    let mut cur = img.clone();
    for op in pipeline {
        cur = apply_op(&cur, op)?;
    }
    Ok(cur)
}

#[derive(Debug, Clone)]
pub enum BatchResult {
    Ok { index: usize, output: Image },
    Err { index: usize, error: OpError },
}

pub fn run_batch(images: &[Image], pipeline: &Pipeline) -> Vec<BatchResult> {
    images.iter().enumerate().map(|(i, img)| {
        match apply_pipeline(img, pipeline) {
            Ok(out) => BatchResult::Ok { index: i, output: out },
            Err(e) => BatchResult::Err { index: i, error: e },
        }
    }).collect()
}

/// Compute what dimensions each step produces, without running on pixels.
/// Returns the dims after each op, starting from the input dims.
pub fn dry_run_dims(start: (u32, u32), pipeline: &Pipeline)
    -> Result<Vec<(u32, u32)>, OpError>
{
    let mut out = Vec::with_capacity(pipeline.len() + 1);
    out.push(start);
    let mut cur = start;
    for op in pipeline {
        cur = match op {
            Op::Resize { width, height } => {
                if *width == 0 || *height == 0 { return Err(OpError::ZeroDimension); }
                (*width, *height)
            }
            Op::Grayscale(_) | Op::FlipHorizontal | Op::FlipVertical
            | Op::Rotate180 | Op::Brightness(_) | Op::Contrast(_) => cur,
            Op::Rotate90 | Op::Rotate270 => (cur.1, cur.0),
            Op::Crop { x, y, width, height } => {
                if *width == 0 || *height == 0 { return Err(OpError::ZeroDimension); }
                if x.saturating_add(*width) > cur.0
                    || y.saturating_add(*height) > cur.1 {
                    return Err(OpError::CropOutOfBounds);
                }
                (*width, *height)
            }
        };
        out.push(cur);
    }
    Ok(out)
}

// ---------- primitives ----------

fn map_pixels(img: &Image, f: impl Fn(Rgb) -> Rgb) -> Image {
    Image::from_pixels(
        img.width, img.height,
        img.pixels.iter().map(|p| f(*p)).collect()
    ).unwrap()
}

fn adjust_brightness(p: Rgb, delta: i16) -> Rgb {
    let clamp = |v: u8| -> u8 {
        (v as i32 + delta as i32).clamp(0, 255) as u8
    };
    Rgb::new(clamp(p.r), clamp(p.g), clamp(p.b))
}

fn adjust_contrast(p: Rgb, factor: f32) -> Rgb {
    let adj = |v: u8| -> u8 {
        let centered = v as f32 - 128.0;
        (centered * factor + 128.0).round().clamp(0.0, 255.0) as u8
    };
    Rgb::new(adj(p.r), adj(p.g), adj(p.b))
}

fn grayscale_image(img: &Image, method: Algorithm) -> Image {
    map_pixels(img, |p| {
        let v = pixel_to_gray(p, method);
        Rgb::new(v, v, v)
    })
}

fn resize_nearest(img: &Image, new_w: u32, new_h: u32) -> Image {
    let mut pixels = Vec::with_capacity((new_w * new_h) as usize);
    for y in 0..new_h {
        for x in 0..new_w {
            let sx = (x as u64 * img.width as u64 / new_w as u64) as u32;
            let sy = (y as u64 * img.height as u64 / new_h as u64) as u32;
            pixels.push(img.get(sx.min(img.width - 1), sy.min(img.height - 1)).unwrap());
        }
    }
    Image::from_pixels(new_w, new_h, pixels).unwrap()
}

fn rotate_90(img: &Image) -> Image {
    let mut out = Image::new(img.height, img.width);
    for y in 0..img.height {
        for x in 0..img.width {
            let p = img.get(x, y).unwrap();
            // (x, y) -> (height - 1 - y, x)
            out.set(img.height - 1 - y, x, p);
        }
    }
    out
}

fn rotate_180(img: &Image) -> Image {
    let mut out = Image::new(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let p = img.get(x, y).unwrap();
            out.set(img.width - 1 - x, img.height - 1 - y, p);
        }
    }
    out
}

fn rotate_270(img: &Image) -> Image {
    let mut out = Image::new(img.height, img.width);
    for y in 0..img.height {
        for x in 0..img.width {
            let p = img.get(x, y).unwrap();
            // (x, y) -> (y, width - 1 - x)
            out.set(y, img.width - 1 - x, p);
        }
    }
    out
}

fn flip_h(img: &Image) -> Image {
    let mut out = Image::new(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let p = img.get(x, y).unwrap();
            out.set(img.width - 1 - x, y, p);
        }
    }
    out
}

fn flip_v(img: &Image) -> Image {
    let mut out = Image::new(img.width, img.height);
    for y in 0..img.height {
        for x in 0..img.width {
            let p = img.get(x, y).unwrap();
            out.set(x, img.height - 1 - y, p);
        }
    }
    out
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

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(w: u32, h: u32) -> Image {
        let mut pixels = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                if (x + y) % 2 == 0 { pixels.push(Rgb::new(200, 100, 50)); }
                else { pixels.push(Rgb::new(50, 100, 200)); }
            }
        }
        Image::from_pixels(w, h, pixels).unwrap()
    }

    #[test]
    fn empty_pipeline_is_identity() {
        let img = checker(4, 4);
        let out = apply_pipeline(&img, &vec![]).unwrap();
        assert_eq!(out, img);
    }

    #[test]
    fn resize_updates_dims() {
        let img = checker(4, 4);
        let out = apply_op(&img, &Op::Resize { width: 8, height: 2 }).unwrap();
        assert_eq!(out.width, 8);
        assert_eq!(out.height, 2);
    }

    #[test]
    fn rotate90_swaps_dims() {
        let img = checker(3, 5);
        let out = apply_op(&img, &Op::Rotate90).unwrap();
        assert_eq!((out.width, out.height), (5, 3));
    }

    #[test]
    fn rotate_four_times_is_identity() {
        let img = checker(3, 4);
        let mut cur = img.clone();
        for _ in 0..4 {
            cur = apply_op(&cur, &Op::Rotate90).unwrap();
        }
        assert_eq!(cur, img);
    }

    #[test]
    fn rotate180_preserves_dims() {
        let img = checker(4, 6);
        let out = apply_op(&img, &Op::Rotate180).unwrap();
        assert_eq!((out.width, out.height), (4, 6));
        // double rotate = identity
        let back = apply_op(&out, &Op::Rotate180).unwrap();
        assert_eq!(back, img);
    }

    #[test]
    fn flip_twice_is_identity() {
        let img = checker(5, 3);
        let out = apply_pipeline(&img, &vec![Op::FlipHorizontal, Op::FlipHorizontal]).unwrap();
        assert_eq!(out, img);
        let out2 = apply_pipeline(&img, &vec![Op::FlipVertical, Op::FlipVertical]).unwrap();
        assert_eq!(out2, img);
    }

    #[test]
    fn crop_extracts_region() {
        let img = checker(6, 6);
        let out = apply_op(&img, &Op::Crop { x: 1, y: 1, width: 2, height: 2 }).unwrap();
        assert_eq!((out.width, out.height), (2, 2));
        assert_eq!(out.get(0, 0).unwrap(), img.get(1, 1).unwrap());
        assert_eq!(out.get(1, 1).unwrap(), img.get(2, 2).unwrap());
    }

    #[test]
    fn crop_out_of_bounds_errors() {
        let img = checker(4, 4);
        let r = apply_op(&img, &Op::Crop { x: 2, y: 2, width: 4, height: 4 });
        assert_eq!(r, Err(OpError::CropOutOfBounds));
    }

    #[test]
    fn brightness_clamps() {
        let img = Image::filled(2, 2, Rgb::new(250, 10, 128));
        let out = apply_op(&img, &Op::Brightness(20)).unwrap();
        assert_eq!(out.get(0, 0).unwrap(), Rgb::new(255, 30, 148)); // r clamped
        let out2 = apply_op(&img, &Op::Brightness(-20)).unwrap();
        assert_eq!(out2.get(0, 0).unwrap(), Rgb::new(230, 0, 108)); // g clamped
    }

    #[test]
    fn contrast_identity() {
        let img = checker(3, 3);
        let out = apply_op(&img, &Op::Contrast(1.0)).unwrap();
        assert_eq!(out, img);
    }

    #[test]
    fn grayscale_produces_gray_pixels() {
        let img = Image::filled(2, 2, Rgb::new(255, 0, 0));
        let out = apply_op(&img, &Op::Grayscale(Algorithm::Luminosity)).unwrap();
        let p = out.get(0, 0).unwrap();
        assert_eq!(p.r, p.g);
        assert_eq!(p.g, p.b);
    }

    #[test]
    fn pipeline_is_fold() {
        let img = checker(4, 4);
        let pipeline = vec![
            Op::Rotate90,
            Op::FlipHorizontal,
            Op::Brightness(10),
        ];
        let by_fold = apply_pipeline(&img, &pipeline).unwrap();
        let mut cur = img.clone();
        for op in &pipeline {
            cur = apply_op(&cur, op).unwrap();
        }
        assert_eq!(by_fold, cur);
    }

    #[test]
    fn dry_run_predicts_dims() {
        let pipeline = vec![
            Op::Rotate90,
            Op::Resize { width: 10, height: 5 },
            Op::Crop { x: 0, y: 0, width: 4, height: 2 },
        ];
        let dims = dry_run_dims((8, 6), &pipeline).unwrap();
        assert_eq!(dims, vec![(8, 6), (6, 8), (10, 5), (4, 2)]);
    }

    #[test]
    fn dry_run_catches_bad_crop() {
        let pipeline = vec![Op::Crop { x: 10, y: 0, width: 5, height: 5 }];
        let r = dry_run_dims((8, 8), &pipeline);
        assert_eq!(r, Err(OpError::CropOutOfBounds));
    }

    #[test]
    fn batch_isolates_errors() {
        let images = vec![
            checker(4, 4),
            checker(2, 2),
            checker(10, 10),
        ];
        let pipeline = vec![Op::Crop { x: 0, y: 0, width: 3, height: 3 }];
        let results = run_batch(&images, &pipeline);
        assert_eq!(results.len(), 3);
        match &results[0] { BatchResult::Ok { output, .. } => assert_eq!((output.width, output.height), (3, 3)), _ => panic!() }
        match &results[1] { BatchResult::Err { error, .. } => assert_eq!(*error, OpError::CropOutOfBounds), _ => panic!() }
        match &results[2] { BatchResult::Ok { output, .. } => assert_eq!((output.width, output.height), (3, 3)), _ => panic!() }
    }

    #[test]
    fn pipeline_is_data_and_equal() {
        let a: Pipeline = vec![Op::Rotate90, Op::Brightness(5)];
        let b: Pipeline = vec![Op::Rotate90, Op::Brightness(5)];
        assert_eq!(a, b);
    }

    #[test]
    fn pipeline_order_matters() {
        let img = Image::filled(3, 3, Rgb::new(100, 100, 100));
        let a = apply_pipeline(&img, &vec![Op::Brightness(50), Op::Contrast(2.0)]).unwrap();
        let b = apply_pipeline(&img, &vec![Op::Contrast(2.0), Op::Brightness(50)]).unwrap();
        // contrast uses 128 as the mid; order affects output
        assert_ne!(a, b);
    }

    #[test]
    fn resize_zero_errors() {
        let img = checker(4, 4);
        let r = apply_op(&img, &Op::Resize { width: 0, height: 4 });
        assert_eq!(r, Err(OpError::ZeroDimension));
    }
}
