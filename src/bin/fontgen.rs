
use image::{RgbaImage, Rgba};
use imageproc::{drawing::{self, text_size, draw_text_mut, Canvas}, rect::Rect};
use rusttype::{Font, Scale};
use robust::{clapx::ensure_existing_file, font::{BitmapFontHeader, ColorMode}};
use std::{path::PathBuf};
use std::fs;

use clap::{Parser, builder::{PathBufValueParser, TypedValueParser}};




#[derive(Parser)]
struct Args {
    #[arg(value_parser = PathBufValueParser::new().try_map(ensure_existing_file))]
    source_font: PathBuf,

    #[arg(short = 'o')]
    output_bitmap: Option<PathBuf>,
    
    #[arg(short = 'p')]
    output_preview: Option<PathBuf>,

    #[arg(short = 's', long = "size")]
    font_size: Option<f32>,
}

fn main() -> anyhow::Result<()> {

    let args = Args::parse();

    let font = fs::read(&args.source_font)?;
    let Some(font) = Font::try_from_vec(font) else {
        #[allow(unreachable_code)] // 🥴
        return anyhow::bail!("failed to load font from {}", args.source_font.as_os_str().to_string_lossy())
    };

    

    let char_size = args.font_size.unwrap_or(16.0);
    let scale = Scale::uniform(char_size);
    // let scale_outline = Scale::uniform(char_height+2f32);
    // let scale_outline = Scale{ y: char_height+4f32, x: char_height + 4f32 };
    let char_height = char_size as u32 + 2;
    let char_width = font.glyph('w').scaled(scale).h_metrics().advance_width as u32 + 2;

    let img_height = char_height as u32 * 4;
    let img_width = char_width as u32 * 0x20;
    let mut image = RgbaImage::new(img_width, img_height);
    let mut preview = RgbaImage::new(img_width, img_height);
    println!("Image size: {img_width}x{img_height}");

    let white = Rgba([0xff, 0xff, 0xff, 0xff]);
    let black = Rgba([0u8, 0u8, 0u8, 255u8]);
    fn gray(l: u8) -> Rgba<u8> { Rgba([l, 0, l, 0xff]) }
    let light = gray(0xd0);
    let dark = gray(0xb0);


    for row in 0u8..4 {
        let offset = 0x20*row;
        let text = if row == 0 {
            // Use 0..1f for special characters
            "\x7f¥ TODO: Special Chars ---\x1a\x1b\x1c\x1d\x1e\x1f".to_owned()
        } else {
            String::from_iter((0u8..0x20u8).map(|i| (i + offset) as char))
        };
        let y = 1 + (char_height as i32 * row as i32);
        let row_even = (row & 1) == 0;
        for (i, (ci, c)) in text.char_indices().enumerate() {
            let col_even = (i & 1) == 0;
            let bg = if col_even ^ row_even {light} else {dark};
            let x = 1 + (i as i32 * char_width as i32);
            let cox = (x - 1) as u32;
            let coy = (y - 1) as u32;
            let rect = Rect::at(x-1, y-1).of_size(char_width, char_height as u32);
            drawing::draw_filled_rect_mut(&mut preview, rect, bg);

            match c {
                // Draw sprites
                '\x1a' => {
                    let half_h = char_height / 2;
                    let rect = Rect::at(x - 1, y - 1).of_size(char_width, half_h);
                    drawing::draw_filled_rect_mut(&mut image, rect, black);
                    drawing::draw_filled_rect_mut(&mut preview, rect, black);
                }
                '\x1b' => {
                    let half_h = char_height / 2;
                    let rect = Rect::at(x - 1, (y - 1) + half_h as i32).of_size(char_width, half_h);
                    drawing::draw_filled_rect_mut(&mut image, rect, black);
                    drawing::draw_filled_rect_mut(&mut preview, rect, black);
                }
                '\x1c' => {
                    // drawing::draw_filled_rect_mut(&mut image, rect, white);
                    // drawing::draw_filled_rect_mut(&mut preview, rect, white);

                    for cy in 0..char_height {
                        if cy & 1 != 0 {continue}
                        for cx in 0..char_width {
                            if cx & 1 != 0 {continue}
                            image.draw_pixel(cox + cx, coy + cy, black);
                            preview.draw_pixel(cox + cx, coy + cy, black);
                        }
                    }
                }
                '\x1d' => {
                    // drawing::draw_filled_rect_mut(&mut image, rect, white);
                    // drawing::draw_filled_rect_mut(&mut preview, rect, white);

                    for cy in 0..char_height {
                        for cx in 0..char_width {
                            if (cx & 1) ^ (cy & 1) != 0 {continue}
                            image.draw_pixel(cox + cx, coy + cy, black);
                            preview.draw_pixel(cox + cx, coy + cy, black);
                        }
                    }
                }
                '\x1e' => {
                    // drawing::draw_filled_rect_mut(&mut image, rect, white);
                    // drawing::draw_filled_rect_mut(&mut preview, rect, white);

                    for cy in 0..char_height {
                        for cx in 0..char_width {
                            if ((cy & 1) == 0) && (cx & 1) != ((cy & 2) >> 1) {continue}
                            image.draw_pixel(cox + cx, coy + cy, black);
                            preview.draw_pixel(cox + cx, coy + cy, black);
                        }
                    }
                }
                '\x1f' => {
                    drawing::draw_filled_rect_mut(&mut image, rect, black);
                    drawing::draw_filled_rect_mut(&mut preview, rect, black);
                }
                _ => {
                    // Draw character using font
                    // let text_char = String::new()
                    let Some(text_char) = text.get(ci..(ci + c.len_utf8())) else {
                        panic!("Could not index into text using index {x}");
                    };
                    
                    for sx in -1..2 {
                        for sy in -1..2 {
                            draw_text_mut(&mut image, white, x + sx, y + sy, scale, &font, text_char);
                            draw_text_mut(&mut preview, white, x + sx, y + sy, scale, &font, text_char);
                        }
                    }

                    draw_text_mut(&mut image, black, x, y, scale, &font, text_char);
                    draw_text_mut(&mut preview, black, x, y, scale, &font, text_char);
                }
            }
        }

        

        let (w, h) = text_size(scale, &font, &text);
        println!("Text size: {}x{}", w, h);


    }

    let output_path = args.output_bitmap.unwrap_or_else(|| args.source_font.with_extension("bmf"));

    let header = BitmapFontHeader{
        version: 1,
        width: char_width as u8,
        height: char_height as u8,
        outline: 1,
        color_mode: ColorMode::Outlined,
    };

    // let output_file = File::create(output_path)?;
    let output_buf = image.pixels().map(|p| {
        // If alpha is less than 50%, pixel is transparent
        if p.0[3] < 0x80 {0} else {
            let mono = (p.0[0] as u16 + p.0[1] as u16 + p.0[2] as u16) / 3;
            if mono >= 0xff {0xffu8} else {(mono + 1)  as u8}
        }
    });
    fs::write(output_path, header.into_iter().chain(output_buf).collect::<Vec<u8>>()).unwrap();

    let preview_path = args.output_preview.unwrap_or_else(|| args.source_font.with_extension("png"));

    let _ = preview.save_with_format(preview_path, image::ImageFormat::Png).unwrap();

    Ok(())
}

