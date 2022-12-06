mod system;
mod bitflags;
mod util;

use std::fs;
use anyhow::Result;
use minifb::{WindowOptions, Window, Key};
use std::env;

const WIDTH: usize = 512;
const HEIGHT: usize = 480;
const SCALE: usize = 2;

fn main() -> Result<()> {

    let mut args = env::args();
    let _ = args.next();
    let cart_file_path = args.next().unwrap_or("carts/nestest.nes".to_owned());

    let title = format!("robust - {} - Press ESC to exit", &cart_file_path);
    color_backtrace::install();
    let mut system = system::System::new();

    let cart_file = fs::File::open(cart_file_path)?;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        title.as_str(),
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    system.load_cart(&cart_file)?;

    system.reset();

    eprintln!();
    eprintln!("Starting execution...");

    
    // let a1 = args.next();
    eprintln!("");

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {

        system.run_cycle().or_else(|e| {
            eprintln!("\nStack:");
            system.print_stack()?;
            eprintln!();
            Err(e)
        })?;

        for (y, row) in system.ppu.mono_frame_buffer.iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                let r = (*val as u32) << 0;
                let g = (*val as u32) << 8;
                let b = (*val as u32) << 16;

                for by in 0..SCALE {
                    for bx in 0..SCALE {
                        
                        let buf_y = ((y * SCALE) + by) * WIDTH;
                        let buf_x = (x * SCALE) + bx;

                        // eprintln!("x: {bx} y: {by} => x: {buf_x} y: {buf_y} ({y})");

                        buffer[buf_y + buf_x] = r | g | b;
                    }
                }
  

                
            }
            // panic!("whoa");
            // break;
        }

        // panic!("whoaaa");

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
    eprintln!("\nVRAM:");
    system::dump_mem(system.ppu.vram, None)?;
    eprintln!("\nPalette:");
    system::dump_mem(system.ppu.palette, None)?;


    // let steps: u32 = args.next().map(|s| s.parse().unwrap())
    //     .unwrap_or(2048u32);



    eprintln!("\nDone!");

    Ok(())
}

#[cfg(test)]
mod tests;