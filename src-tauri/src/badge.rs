//! Tray icon badge: composite red pill + count onto base PNG.

use image::{Rgba, RgbaImage};
use tauri::image::Image;

const BASE_PNG: &[u8] = include_bytes!("../icons/32x32.png");

/// Build a tray `Image` with optional unread badge (0 = plain icon).
pub fn make_tray_image(unread: u32) -> Result<Image<'static>, String> {
    let img = image::load_from_memory(BASE_PNG).map_err(|e| e.to_string())?;
    let mut rgba = img.to_rgba8();

    // Normalize to 32x32
    if rgba.width() != 32 || rgba.height() != 32 {
        rgba = image::imageops::resize(&rgba, 32, 32, image::imageops::FilterType::Lanczos3);
    }

    if unread > 0 {
        draw_badge(&mut rgba, unread);
    }

    let w = rgba.width();
    let h = rgba.height();
    let raw = rgba.into_raw();
    Ok(Image::new_owned(raw, w, h))
}

fn draw_badge(img: &mut RgbaImage, count: u32) {
    let w = img.width() as i32;
    let h = img.height() as i32;

    // Badge circle bottom-right
    let r = 8i32;
    let cx = w - r - 1;
    let cy = h - r - 1;

    let red = Rgba([220, 50, 47, 255]);
    let white = Rgba([255, 255, 255, 255]);
    let dark = Rgba([120, 20, 18, 255]);

    // filled circle + subtle ring
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let dx = x - cx;
            let dy = y - cy;
            let d2 = dx * dx + dy * dy;
            if d2 <= (r - 1) * (r - 1) {
                img.put_pixel(x as u32, y as u32, red);
            } else if d2 <= r * r {
                img.put_pixel(x as u32, y as u32, dark);
            }
        }
    }

    let label = if count > 9 {
        "9+".to_string()
    } else {
        count.to_string()
    };

    // Tiny 3x5 bitmap font
    let glyphs: Vec<[u8; 5]> = label.chars().filter_map(glyph3x5).collect();
    if glyphs.is_empty() {
        return;
    }

    let gw = 3i32;
    let gh = 5i32;
    let gap = 1i32;
    let total_w = glyphs.len() as i32 * gw + (glyphs.len() as i32 - 1) * gap;
    let mut x0 = cx - total_w / 2;
    let y0 = cy - gh / 2;

    for g in glyphs {
        for row in 0..5 {
            let bits = g[row as usize];
            for col in 0..3 {
                if bits & (1 << (2 - col)) != 0 {
                    let px = x0 + col;
                    let py = y0 + row;
                    if px >= 0 && py >= 0 && px < w && py < h {
                        img.put_pixel(px as u32, py as u32, white);
                    }
                }
            }
        }
        x0 += gw + gap;
    }
}

/// 3x5 font rows as bit masks (bit2 = left pixel).
fn glyph3x5(c: char) -> Option<[u8; 5]> {
    Some(match c {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        _ => return None,
    })
}
