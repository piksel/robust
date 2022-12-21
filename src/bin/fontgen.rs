
use image::{RgbaImage, Rgba};
use imageproc::{drawing::{self, text_size, draw_text_mut}, rect::Rect};
use rusttype::{Font, Scale};

use std::{path::PathBuf, fs::File, io::Write};
use std::fs;

use clap::{Parser, builder::{ValueParser, PathBufValueParser, TypedValueParser}, value_parser};


fn ensure_existing_file(p: PathBuf) -> anyhow::Result<PathBuf> {
    if !p.exists() {anyhow::bail!("Path does not exist")} else
    if !p.is_file() {anyhow::bail!("Path is not a file")} else
    {Ok(p)}
}

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
            "\x7f¥ TODO: Special Chars ❤️♥️\x7f\x7f\x7f\x7f\x7f\x7f\x7f\x7f\x7f".to_owned()
        } else {
            String::from_iter((0u8..0x20u8).map(|i| (i + offset) as char))
        };
        let y = 1 + (char_height as i32 * row as i32);
        let row_even = (row & 1) == 0;
        for (i, (ci, c)) in text.char_indices().enumerate() {
            let col_even = (i & 1) == 0;
            let bg = if col_even ^ row_even {light} else {dark};
            let x = 1 + (i as i32 * char_width as i32);
            let rect = Rect::at(x-1, y-1).of_size(char_width, char_height as u32);
            drawing::draw_filled_rect_mut(&mut preview, rect, bg);

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

        

        let (w, h) = text_size(scale, &font, &text);
        println!("Text size: {}x{}", w, h);


    }

    let output_path = args.output_bitmap.unwrap_or_else(|| args.source_font.with_extension("bmf"));

    
    // let output_file = File::create(output_path)?;
    let output_buf: Vec<u8> = image.pixels().map(|p| {
        // If alpha is less than 50%, pixel is transparent
        if p.0[3] < 0x80 {0} else {
            let mono = (p.0[0] as u16 + p.0[1] as u16 + p.0[2] as u16) / 3;
            if mono >= 0xff {0xffu8} else {(mono + 1)  as u8}
        }
    }).collect();
    fs::write(output_path, output_buf).unwrap();

    let preview_path = args.output_preview.unwrap_or_else(|| args.source_font.with_extension("png"));

    let _ = preview.save_with_format(preview_path, image::ImageFormat::Png).unwrap();

    Ok(())
}

