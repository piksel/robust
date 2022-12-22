use robust::{system::{self, apu::ControllerButton}, font::Font};

use std::fs;
use anyhow::Result;
use minifb::{WindowOptions, Window, Key, KeyRepeat};
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

    let mut buffer: Vec<u32> = vec![0x00808080; WIDTH * HEIGHT];

    let mut window = Window::new(
        title.as_str(),
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let font = Font::from_bytes(*include_bytes!("../../fonts/PixelOperatorMonoHB.bmf"));

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    font.draw_text(&mut buffer, 10, 10, "Loading...")?;

    window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;

    system.load_cart(&cart_file)?;

    system.reset();

    eprintln!();
    eprintln!("Starting execution...");

    
    // let a1 = args.next();
    eprintln!("");

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut last_frame = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {


            for key in window.get_keys_pressed(KeyRepeat::No) {
                if let Some((cid, btn)) = map_key_to_button(key) {
                    system.apu.set_controller_button(cid, btn, true);
                }
            }

            for key in window.get_keys_released() {
                if let Some((cid, btn)) = map_key_to_button(key) {
                    system.apu.set_controller_button(cid, btn, false);
                }
            }


            system.run_cycle().or_else(|e| {
                eprintln!("\nStack:");
                system.print_stack()?;
                eprintln!();
                Err(e)
            })?;


            for (y, row) in system.get_frame().iter().enumerate() {
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
        
        font.draw_text(&mut buffer, 10, 10, &format!("Cycle: {}", system.cycles))?;

        let now = std::time::Instant::now();
        let render_time = now - last_frame;

        let fps = 1.0 / render_time.as_secs_f64();

        font.draw_text(&mut buffer, 10, 30, &format!("FPS: {:.2}", fps))?;

        last_frame = now;

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();

  
    }
    
    // eprintln!("\nVRAM:"); system::dump_mem(system.ppu.vram, None)?;
    // eprintln!("\nPalette:"); system.dump_palette();   
    eprintln!("\nZero Page:"); system.dump_zero_page();

    eprintln!("\nDone!");

    Ok(())
}

fn map_key_to_button(key: Key) -> Option<(usize, ControllerButton)> {
    match key {
        Key::Up        => Some((0, ControllerButton::Up)),
        Key::Down      => Some((0, ControllerButton::Down)),
        Key::Left      => Some((0, ControllerButton::Left)),
        Key::Right     => Some((0, ControllerButton::Right)),
        Key::Z         => Some((0, ControllerButton::A)),
        Key::X         => Some((0, ControllerButton::B)),
        Key::Enter     => Some((0, ControllerButton::Start)),
        Key::Backspace => Some((0, ControllerButton::Select)),
        _ => None,
    }
}