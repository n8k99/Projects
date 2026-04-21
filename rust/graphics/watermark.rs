//! Watermarking Application — LSB steganography + visible overlay + position ADT.
//!
//! Sixteenth Graphics project. Distinctive moves:
//!
//!   * **LSB steganography with length-prefix header** — embed arbitrary
//!     bytes in the low bits of pixel channels. First 32 bits encode
//!     length in bytes. `bits_per_channel ∈ 1..=4`. Higher = more
//!     capacity, more visible artifacts.
//!   * **Capacity calculation** — pure function over (image size,
//!     bits_per_channel). Tells callers up-front if a payload fits.
//!   * **Visible watermark with position ADT** — `Position::{TopLeft,
//!     TopRight, BottomLeft, BottomRight, Center, Tiled}`. Position
//!     is encoded as data; the compositor resolves it to coordinates
//!     at apply-time.
//!   * **Alpha-blend compositing** — overlay uses the same formula as
//!     G123's highlight (base*(255-α) + wm*α)/255, but applied to the
//!     whole watermark image, not a rectangle.

use super::grayscale::{Image, Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Tiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkError {
    PayloadTooLarge { needed_bytes: usize, capacity_bytes: usize },
    InvalidBitsPerChannel,
    CorruptHeader,
    TruncatedData,
}

/// Bytes of capacity for hiding data in an image at `bits_per_channel`.
/// Accounts for the 4-byte length header.
pub fn lsb_capacity(image: &Image, bits_per_channel: u8) -> usize {
    if bits_per_channel == 0 || bits_per_channel > 4 { return 0; }
    let total_bits = (image.pixels.len() * 3) * bits_per_channel as usize;
    let total_bytes = total_bits / 8;
    total_bytes.saturating_sub(4) // 4-byte length header
}

/// Embed `payload` into `image`'s low bits. Returns a new image.
pub fn embed_lsb(image: &Image, payload: &[u8], bits_per_channel: u8)
    -> Result<Image, WatermarkError>
{
    if bits_per_channel == 0 || bits_per_channel > 4 {
        return Err(WatermarkError::InvalidBitsPerChannel);
    }
    let capacity = lsb_capacity(image, bits_per_channel);
    if payload.len() > capacity {
        return Err(WatermarkError::PayloadTooLarge {
            needed_bytes: payload.len(), capacity_bytes: capacity,
        });
    }

    // Build a single bit stream: 4-byte length header + payload.
    let mut bits: Vec<u8> = Vec::with_capacity((payload.len() + 4) * 8);
    let len_bytes = (payload.len() as u32).to_be_bytes();
    for byte in len_bytes.iter().chain(payload.iter()) {
        for i in 0..8 {
            bits.push((byte >> (7 - i)) & 1);
        }
    }

    let mask = !((1u8 << bits_per_channel) - 1);
    let mut new_pixels: Vec<Rgb> = image.pixels.clone();
    let mut bit_idx = 0usize;
    'outer: for pixel in new_pixels.iter_mut() {
        for c in 0..3 {
            let mut chunk: u8 = 0;
            for _ in 0..bits_per_channel {
                if bit_idx >= bits.len() { break; }
                chunk = (chunk << 1) | bits[bit_idx];
                bit_idx += 1;
            }
            // align chunk into low bits_per_channel
            let channel_val = match c {
                0 => &mut pixel.r,
                1 => &mut pixel.g,
                _ => &mut pixel.b,
            };
            *channel_val = (*channel_val & mask) | chunk;
            if bit_idx >= bits.len() { break 'outer; }
        }
    }

    Ok(Image {
        width: image.width, height: image.height, pixels: new_pixels,
    })
}

/// Extract the payload hidden in `image` at `bits_per_channel`.
pub fn extract_lsb(image: &Image, bits_per_channel: u8)
    -> Result<Vec<u8>, WatermarkError>
{
    if bits_per_channel == 0 || bits_per_channel > 4 {
        return Err(WatermarkError::InvalidBitsPerChannel);
    }
    let mask = (1u8 << bits_per_channel) - 1;
    let mut bits: Vec<u8> = Vec::with_capacity(image.pixels.len() * 3 * bits_per_channel as usize);
    for pixel in &image.pixels {
        for c in 0..3 {
            let channel_val = match c {
                0 => pixel.r,
                1 => pixel.g,
                _ => pixel.b,
            };
            let chunk = channel_val & mask;
            for i in 0..bits_per_channel {
                bits.push((chunk >> (bits_per_channel - 1 - i)) & 1);
            }
        }
    }

    if bits.len() < 32 { return Err(WatermarkError::TruncatedData); }
    let mut len_bytes = [0u8; 4];
    for i in 0..32 {
        len_bytes[i / 8] = (len_bytes[i / 8] << 1) | bits[i];
    }
    let payload_len = u32::from_be_bytes(len_bytes) as usize;
    let total_bits_needed = 32 + payload_len * 8;
    if bits.len() < total_bits_needed { return Err(WatermarkError::TruncatedData); }

    let mut payload = Vec::with_capacity(payload_len);
    for byte_i in 0..payload_len {
        let mut b: u8 = 0;
        for bit_j in 0..8 {
            b = (b << 1) | bits[32 + byte_i * 8 + bit_j];
        }
        payload.push(b);
    }
    Ok(payload)
}

/// Resolve a position to (x, y) offset for placing a watermark of given
/// size onto a base image. For Tiled, returns (0, 0) — caller handles tiling.
pub fn resolve_position(base: &Image, wm: &Image, pos: Position) -> (i32, i32) {
    let bw = base.width as i32;
    let bh = base.height as i32;
    let ww = wm.width as i32;
    let wh = wm.height as i32;
    match pos {
        Position::TopLeft => (0, 0),
        Position::TopRight => ((bw - ww).max(0), 0),
        Position::BottomLeft => (0, (bh - wh).max(0)),
        Position::BottomRight => ((bw - ww).max(0), (bh - wh).max(0)),
        Position::Center => ((bw - ww) / 2, (bh - wh) / 2),
        Position::Tiled => (0, 0),
    }
}

/// Apply a visible watermark to `base` at `position` with `opacity` (0..=255).
/// For Tiled, repeats the watermark across the base.
pub fn apply_visible_watermark(base: &Image, watermark: &Image,
                                position: Position, opacity: u8) -> Image {
    let mut out = base.clone();
    match position {
        Position::Tiled => {
            let mut y = 0;
            while y < base.height {
                let mut x = 0;
                while x < base.width {
                    blend_region(&mut out, x as i32, y as i32, watermark, opacity);
                    x += watermark.width;
                }
                y += watermark.height;
            }
        }
        _ => {
            let (x, y) = resolve_position(base, watermark, position);
            blend_region(&mut out, x, y, watermark, opacity);
        }
    }
    out
}

fn blend_region(out: &mut Image, x0: i32, y0: i32, wm: &Image, opacity: u8) {
    for wy in 0..wm.height {
        for wx in 0..wm.width {
            let bx = x0 + wx as i32;
            let by = y0 + wy as i32;
            if bx < 0 || by < 0 { continue; }
            if bx >= out.width as i32 || by >= out.height as i32 { continue; }
            let base_px = out.get(bx as u32, by as u32).unwrap();
            let wm_px = wm.get(wx, wy).unwrap();
            let blended = Rgb::new(
                blend_channel(base_px.r, wm_px.r, opacity),
                blend_channel(base_px.g, wm_px.g, opacity),
                blend_channel(base_px.b, wm_px.b, opacity),
            );
            out.set(bx as u32, by as u32, blended);
        }
    }
}

fn blend_channel(a: u8, b: u8, alpha: u8) -> u8 {
    let a = a as u16;
    let b = b as u16;
    let al = alpha as u16;
    ((a * (255 - al) + b * al) / 255) as u8
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(w: u32, h: u32, c: Rgb) -> Image {
        Image::filled(w, h, c)
    }

    #[test]
    fn capacity_scales_with_bits_per_channel() {
        let img = blank(10, 10, Rgb::new(128, 128, 128));
        let c1 = lsb_capacity(&img, 1);
        let c2 = lsb_capacity(&img, 2);
        let c4 = lsb_capacity(&img, 4);
        assert!(c2 > c1);
        assert!(c4 > c2);
    }

    #[test]
    fn capacity_invalid_bits_returns_zero() {
        let img = blank(10, 10, Rgb::new(0, 0, 0));
        assert_eq!(lsb_capacity(&img, 0), 0);
        assert_eq!(lsb_capacity(&img, 5), 0);
    }

    #[test]
    fn embed_and_extract_roundtrip() {
        let img = blank(32, 32, Rgb::new(128, 128, 128));
        let payload = b"Hello, watermark!".to_vec();
        let embedded = embed_lsb(&img, &payload, 1).unwrap();
        let extracted = extract_lsb(&embedded, 1).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn embed_and_extract_bits_per_channel_2() {
        let img = blank(16, 16, Rgb::new(100, 150, 200));
        let payload: Vec<u8> = (0..64).map(|i| i as u8).collect();
        let embedded = embed_lsb(&img, &payload, 2).unwrap();
        let extracted = extract_lsb(&embedded, 2).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn embed_fails_when_payload_too_large() {
        let img = blank(2, 2, Rgb::new(0, 0, 0));
        let payload = vec![0u8; 1000];
        let r = embed_lsb(&img, &payload, 1);
        assert!(matches!(r, Err(WatermarkError::PayloadTooLarge { .. })));
    }

    #[test]
    fn embed_invalid_bits_errors() {
        let img = blank(10, 10, Rgb::new(0, 0, 0));
        let r = embed_lsb(&img, b"x", 5);
        assert_eq!(r, Err(WatermarkError::InvalidBitsPerChannel));
    }

    #[test]
    fn embed_preserves_high_bits() {
        let img = blank(4, 4, Rgb::new(0xF0, 0xF0, 0xF0)); // high nibble preserved
        let payload = b"A".to_vec();
        let embedded = embed_lsb(&img, &payload, 1).unwrap();
        // Each channel's top 7 bits unchanged
        for p in &embedded.pixels {
            assert_eq!(p.r & 0xFE, 0xF0);
            assert_eq!(p.g & 0xFE, 0xF0);
            assert_eq!(p.b & 0xFE, 0xF0);
        }
    }

    #[test]
    fn resolve_top_left() {
        let base = blank(100, 100, Rgb::BLACK);
        let wm = blank(10, 10, Rgb::WHITE);
        assert_eq!(resolve_position(&base, &wm, Position::TopLeft), (0, 0));
    }

    #[test]
    fn resolve_bottom_right() {
        let base = blank(100, 100, Rgb::BLACK);
        let wm = blank(10, 10, Rgb::WHITE);
        assert_eq!(resolve_position(&base, &wm, Position::BottomRight), (90, 90));
    }

    #[test]
    fn resolve_center() {
        let base = blank(100, 100, Rgb::BLACK);
        let wm = blank(10, 10, Rgb::WHITE);
        assert_eq!(resolve_position(&base, &wm, Position::Center), (45, 45));
    }

    #[test]
    fn visible_watermark_top_left_changes_corner() {
        let base = blank(20, 20, Rgb::new(128, 128, 128));
        let wm = blank(5, 5, Rgb::new(0, 255, 0));
        let out = apply_visible_watermark(&base, &wm, Position::TopLeft, 255);
        // (0, 0) should now be fully green (opacity=255)
        assert_eq!(out.get(0, 0).unwrap(), Rgb::new(0, 255, 0));
        // (10, 10) still gray
        assert_eq!(out.get(10, 10).unwrap(), Rgb::new(128, 128, 128));
    }

    #[test]
    fn visible_watermark_bottom_right_changes_corner() {
        let base = blank(20, 20, Rgb::new(128, 128, 128));
        let wm = blank(5, 5, Rgb::new(255, 0, 0));
        let out = apply_visible_watermark(&base, &wm, Position::BottomRight, 255);
        assert_eq!(out.get(19, 19).unwrap(), Rgb::new(255, 0, 0));
        assert_eq!(out.get(0, 0).unwrap(), Rgb::new(128, 128, 128));
    }

    #[test]
    fn visible_watermark_tiled_covers_whole() {
        let base = blank(20, 20, Rgb::new(128, 128, 128));
        let wm = blank(5, 5, Rgb::new(200, 0, 0));
        let out = apply_visible_watermark(&base, &wm, Position::Tiled, 255);
        // All pixels should be watermark color at full opacity
        for p in &out.pixels {
            assert_eq!(*p, Rgb::new(200, 0, 0));
        }
    }

    #[test]
    fn visible_watermark_opacity_blends() {
        let base = blank(10, 10, Rgb::new(0, 0, 0));
        let wm = blank(10, 10, Rgb::new(200, 200, 200));
        let out = apply_visible_watermark(&base, &wm, Position::TopLeft, 128);
        let p = out.get(0, 0).unwrap();
        // blend(0, 200, 128) ≈ 100
        assert!(p.r > 80 && p.r < 120);
    }

    #[test]
    fn extract_empty_image_errors() {
        let img = Image::from_pixels(1, 1, vec![Rgb::BLACK]).unwrap();
        // Only 3 channels * 1 bit = 3 bits, less than 32-bit header
        let r = extract_lsb(&img, 1);
        assert_eq!(r, Err(WatermarkError::TruncatedData));
    }

    #[test]
    fn tampered_length_header_fails_gracefully() {
        let img = blank(32, 32, Rgb::new(128, 128, 128));
        let embedded = embed_lsb(&img, b"hi", 1).unwrap();
        // Corrupt the first pixel to give a huge length header
        let mut pixels = embedded.pixels.clone();
        for i in 0..11 {
            pixels[i].r |= 1; // force a long length-header value
            pixels[i].g |= 1;
            pixels[i].b |= 1;
        }
        let corrupted = Image::from_pixels(embedded.width, embedded.height, pixels).unwrap();
        let r = extract_lsb(&corrupted, 1);
        // Either errors on truncated data, or the length is now far larger than payload bytes
        assert!(matches!(r, Err(WatermarkError::TruncatedData)) || r.unwrap().len() != 2);
    }
}
