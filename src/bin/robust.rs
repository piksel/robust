use clap::{Parser, builder::{PathBufValueParser, TypedValueParser, PossibleValuesParser}};
use robust::{system::{self, apu::ControllerButton, options::Options, execution_state::ExecutionState}, font::Font, clapx::{ensure_existing_file, scale_value_parser, SCALE_VALUES}, screen::Screen};

use std::{env, fs, path::{PathBuf}, ops::Not};
use anyhow::Result;
use minifb::{WindowOptions, Window, Key, KeyRepeat, Scale, ScaleMode, Menu};

const WIDTH: usize = 512;
const HEIGHT: usize = 480;
const SCALE: usize = 2;

#[derive(Parser)]
pub struct Args {

    #[arg(value_parser = PathBufValueParser::new().try_map(ensure_existing_file))]
    cart_file: Option<PathBuf>,

    #[arg(value_parser = PossibleValuesParser::new(SCALE_VALUES).try_map(scale_value_parser), default_value = "x2")]
    scale: Scale,

    #[arg(short = 't', long = "trace", default_value = "false")]
    trace: bool,

    #[arg(short = 'H', long = "history", default_value = "10")]
    history: usize,
}

fn main() -> Result<()> { 
    let args = Args::parse();

    // let cart_file_path = .next().unwrap_or("carts/nestest.nes".to_owned());

    let logo_text = include_str!("../logo.ansi");

    
    color_backtrace::install();
    let mut system = system::System::new(Options{
        dump_ops: args.trace,
        history_len: args.history,
        ..Default::default()
    });



    let mut screen = Screen::new(WIDTH, HEIGHT, 0x00170530); //: Vec<u32> = vec![; WIDTH * HEIGHT];

    let mut window = Window::new(
        "robust",
        WIDTH,
        HEIGHT,
        WindowOptions{
            scale: args.scale,
            scale_mode: ScaleMode::AspectRatioStretch,
            resize: true,
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_background_color(0x17, 0x05, 0x30);



    let mut debug_opts = DebugOps::new();

    debug_opts.update_menu(&mut window);


    let font = Font::from_bytes(*include_bytes!("../../fonts/PixelOperatorMonoHB.bmf"), 10, 18);

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let logo_x = 0;
    let logo_y = 128;
    screen.draw_text(&font, logo_x + 4, logo_y + 4, logo_text, 0)?;
    screen.draw_text(&font, logo_x, logo_y, logo_text, 1)?;


    window.update_with_buffer(&screen.buffer, WIDTH, HEIGHT)?;

    if let Some(cart_file) = args.cart_file {
        let title = format!("robust - {} - Press ESC to exit", cart_file.to_string_lossy());
        window.set_title(&title);

        let cart_file = fs::File::open(cart_file)?;
        system.load_cart(&cart_file)?;
        screen.draw_text(&font, 10, 10, "Loading...", 1)?;
    
        system.reset();
    }


    eprintln!();
    eprintln!("Starting execution...");

    
    // let a1 = args.next();
    eprintln!("");

    // for _ in 0..100 {
    //     window.update();

    // }

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut last_frame = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {

        if let Some(mid) = window.is_menu_pressed() {
            if let Some(id) = debug_opts.match_id(mid) {
                debug_opts.toggle(id)
            } else {
                panic!("Invalid menu ID {mid}")
            }
        }

        if window.is_key_down(Key::T) {
            debug_opts.toggle(DebugOps::SHOW_TEST_REGS);
        }

        debug_opts.update_menu(&mut window);

        let last_state = if system.has_cartridge() {
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

            let last_state = system.run_cycle().or_else(|e| {
                eprintln!("\nStack:");
                system.print_stack()?;
                system.dump_history();
                eprintln!();
                Err(e)
            })?;


            for (y, row) in system.get_frame().iter().enumerate() {
                for (x, val) in row.iter().enumerate() {

                    for by in 0..SCALE {
                        for bx in 0..SCALE {
                            
                            let buf_y = ((y * SCALE) + by) * WIDTH;
                            let buf_x = (x * SCALE) + bx;

                            // eprintln!("x: {bx} y: {by} => x: {buf_x} y: {buf_y} ({y})");

                            screen.buffer[buf_y + buf_x] = *val;
                        }
                    }
    

                    
                }
                // panic!("whoa");
                // break;
            }

            Some(last_state)
        } else {
            None
        };
        // screen.draw_text(&font, 10, 10, &format!("Cycle: {}", system.cycles), 1)?;

        if debug_opts.show_fps() {
            let now = std::time::Instant::now();
            let render_time = now - last_frame;

            let fps = 1.0 / render_time.as_secs_f64();

            screen.draw_text(&font, 10, 30, &format!("FPS: {:.2}", fps), 1)?;
            last_frame = now;
        }

        if debug_opts.show_input_buttons() {
            let btns = "ABESUDLR";
            
            let btns_text = String::from_iter(btns.chars().enumerate().map(|(i, c)| {
                if system.apu.controller1.get(i as u8) {c} else {' '}
            }));
            screen.draw_text(&font, 10, HEIGHT - 16, &btns_text, 1)?;
        }

        if debug_opts.show_last_state() {
            if let Some(last_state) = last_state {
                let state_text = last_state.to_string();
                let (state_a, state_b) = state_text.split_at(46);

                screen.draw_text(&font, 10, HEIGHT - 48, state_a, 1)?;
                screen.draw_text(&font, 10, HEIGHT - 32, state_b, 1)?;

            }
        }

        if debug_opts.show_test_regs() {
            let test_regs_text = format!("{:02x} {:02x}", system.peek_byte(2), system.peek_byte(3));
            screen.draw_text(&font, WIDTH-64, 16, &test_regs_text, 1)?;
        }



        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&screen.buffer, WIDTH, HEIGHT)
            .unwrap();

  
    }
    
    // eprintln!("\nVRAM:"); system::dump_mem(system.ppu.vram, None)?;
    eprintln!("\nPalette:"); system.dump_palette();   
    // eprintln!("\nZero Page:"); system.dump_zero_page();

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

struct DebugOps {
    handles: Option<[minifb::MenuItemHandle; Self::ITEM_COUNT]>,
    labels: [&'static str; Self::ITEM_COUNT],
    values: [bool; Self::ITEM_COUNT],
    has_changes: bool,
    menu_handle: Option<minifb::MenuHandle>
}
impl DebugOps {
    const MENU_ID_OFFSET: usize = 1001;
    const SHOW_TEST_REGS: usize = 0;
    const SHOW_LAST_STATE: usize = 1;
    const SHOW_INPUT_BUTTONS: usize = 2;
    const SHOW_FPS: usize = 3;
    const ITEM_COUNT: usize = 4;

    const items: [usize; Self::ITEM_COUNT] = [
        Self::SHOW_TEST_REGS,
        Self::SHOW_LAST_STATE,
        Self::SHOW_INPUT_BUTTONS,
        Self::SHOW_FPS,
    ];

    pub fn new() -> Self {
        Self {
            values: [
                false /* show_test_regs */,
                true  /* show_last_state */,
                true  /* show_input_buttons */,
                true  /* show_fps */,
            ],
            has_changes: true,
            handles: None,
            menu_handle: None,
            labels: [
                "test regs",
                "last state",
                "input buttons",
                "fps"
            ]
        }
    }

    pub fn update_menu(&mut self, window: &mut Window) {
        if !self.has_changes {return}
        if let Some(menu_handle) = self.menu_handle {
            eprintln!("Removing menu: {:?}", menu_handle.0);
            window.remove_menu(menu_handle);
        }

        let mut menu = Menu::new("Debug").expect("menu should be creatable");
        let mut handles = [minifb::MenuItemHandle(0); Self::ITEM_COUNT];
        for i in 0..Self::ITEM_COUNT {
            handles[i] = menu.add_item(&self.get_item_label(i), Self::MENU_ID_OFFSET + i).build();
        }
        self.handles = Some(handles);
        self.menu_handle = Some(window.add_menu(&menu));
        self.has_changes = false;
        
    }

    pub fn get_item_label(&self, item_id: usize) -> String {
        (if self.values[item_id] {"Hide "} else {"Show "}).to_owned() + self.labels[item_id]
    }

    pub fn show_test_regs(&self) -> bool { self.values[Self::SHOW_TEST_REGS] }
    pub fn show_last_state(&self) -> bool { self.values[Self::SHOW_LAST_STATE] }
    pub fn show_input_buttons(&self) -> bool { self.values[Self::SHOW_INPUT_BUTTONS] }
    pub fn show_fps(&self) -> bool { self.values[Self::SHOW_FPS] }

    pub(crate) fn match_id(&self, mid: usize) -> Option<usize> {
        if mid >= Self::MENU_ID_OFFSET && mid < Self::MENU_ID_OFFSET + Self::ITEM_COUNT {
            Some(mid - Self::MENU_ID_OFFSET)
        } else {
            None
        }
    }

    pub(crate) fn toggle(&mut self, id: usize) {
        self.values[id].toggle();
        self.has_changes = true;
    }

    // pub(crate) fn get_toggle_menu(menu_item: usize, value: bool) -> (&'static str, usize) {
        
    // }
}

pub trait Toggle {
    fn toggle(&mut self);
}

impl<T> Toggle for T where Self: std::ops::Not<Output = Self> + Sized + Copy {
    fn toggle(&mut self) {
        *self = !*self;
    }
}