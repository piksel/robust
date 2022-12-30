use std::{io::{self, Read, Write as IOWrite}, fs};

use self::{cpu::{CPU}, execution_state::ExecutionState, addr::Addr, options::Options};

use self::cart::Cart;
use anyhow::{Result, Ok};

use tc::{WriteColor, ColorSpec, Color};
use termcolor as tc;

pub mod cart;
pub mod bus;
pub mod cpu;
pub mod execution_state;
pub mod apu;
pub mod ppu;
pub mod addr;
pub mod options;

pub struct System {
    pub(crate) ram: Vec<u8>,
    pub(crate) ppu: ppu::PPU,
    pub apu: apu::APU,
    pub(crate) cpu: CPU,
    pub(crate) cart: Option<Cart>,
    pub cycles: u64,
    pub(crate) oam: [u8; 256],
    pub opts: Options,
    pub(crate) nmi: bool,
    history: Vec<ExecutionState>,
    history_pos: usize,
    pub offset: usize,
}

impl System {
    pub fn new(opts: Options) -> Self {
        let cpu = CPU::init();
        let history_pos = opts.history_len-1;
        System {
            ram: vec![0; 2048],
            ppu: ppu::PPU::init(),
            apu: apu::APU::init(),
            cpu,
            cart: None,
            cycles: 7,
            oam: [0u8; 256],
            opts,
            nmi: false,
            history: Vec::new(),
            history_pos,
            offset: 1016,
        }
    }

    pub fn has_cartridge(&self) -> bool {
        self.cart.is_some()
    }

    pub fn get_frame(&self) -> [[u32; 256]; 240] {
        self.ppu.frame_buffer
    }

    pub fn dump_palette(&self) {
        dump_mem(&self.ppu.palette, None).expect("failed to dump palette");
    }

    pub fn dump_vram(&self) {
        dump_mem(&self.ppu.vram, None).expect("failed to dump vram");
    }

    pub fn dump_zero_page(&self) {
        dump_mem(self.ram.iter().take(0xff), None).expect("failed to dump vram");
    }

    pub fn dump_oam(&self) {
        dump_mem(&self.oam, Some(Addr::from_zero(self.ppu.oam_addr))).expect("failed to dump oam");
    }

    pub fn dump_stack(&self) {
        dump_mem(self.ram.iter().skip(CPU::STACK_BOT.0 as usize).take(0xff), Some(Addr::from_zero(self.cpu.sp))).expect("failed to dump stack");
    }

    pub fn dump_history(&self) {
        for (i, hi) in (self.history_pos..self.opts.history_len).chain(0..self.history_pos).enumerate() {
            if hi >= self.history.len() {break}
            let state = &self.history[hi];
            eprintln!("[{:3}] {state}", 1isize - (self.opts.history_len - i) as isize);
        }
    }

    pub fn load_cart(&mut self, cart_file: &fs::File) -> Result<()> {
        let reader = io::BufReader::new(cart_file);


        // for (ix,mb) in reader.bytes().enumerate() {
        //     match mb {
        //         Result::Ok(b) => eprint!("{b:02x} "),
        //         Err(_) => eprint!("!! ")
        //     }
        //     if ix > 100 { panic!("TOO FAR!")}
        // }
        // println!("");

        // panic!();
        let cart = Cart::new(reader.bytes())?;

        // for b in cart.prg_rom.iter() {
        //     print!("{b:02x} ")
        // }
        // println!();

        self.cart = Some(cart);

        eprintln!("Cart loaded!");


        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        // Read reset vector
        let rv = self.read_addr(0xfffc)?;
        self.cpu.pc = rv;
        eprintln!("Resetting to {}", self.cpu.pc);
        Ok(())
    }

    pub fn run_cycle(&mut self) -> Result<ExecutionState> {

        // let mut stderr = tc::BufferWriter::stderr(tc::ColorChoice::AlwaysAnsi);
        // let mut bad_colors = tc::ColorSpec::new();
        // bad_colors.set_fg(Some(tc::Color::Red));

        // let mut good_colors = tc::ColorSpec::new();
        // good_colors.set_fg(Some(tc::Color::Green));

        // let mut dark_colors = tc::ColorSpec::new();
        // dark_colors.set_fg(Some(tc::Color::Black)).set_intense(true);

        // let norm_colors = tc::ColorSpec::new();

        // init program counter (for use with test cart)


        // loop {

        loop {

            if self.nmi {
                    // panic!("First NMI at {}", self.cycles);
                CPU::stack_push_word(self, self.cpu.pc.into())?;
                CPU::stack_push_byte(self, self.cpu.status())?;

                let nmi_handler_addr = self.read_addr(0xfffa)?;
                self.cpu.pc = nmi_handler_addr;

                // eprintln!("NMI! => {nmi_handler_addr}");
                
                self.nmi = false;
            }


            let cpu = self.cpu.clone();
            let (op, am) = cpu::load(self)?;

            let byte_count = am.bytes() + 1;
            let pc_bytes = (0..byte_count).map(|i| self.peek_byte(cpu.pc + (i as i8))).collect();
            
            let actual = ExecutionState {
                    cpu, 
                    pc_bytes, 
                    am: am.clone(), 
                    cycles: self.cycles,
                    ppu: (self.ppu.scan_row, self.ppu.scan_line)
            };

            if self.opts.dump_ops {
                // let actual_log = actual.to_string();
                eprintln!("{actual}");
            }

            if self.opts.history_len > 0 {
                if self.history.len() < self.opts.history_len {
                    self.history.push(actual.clone());
                    self.history_pos = self.history.len() - 1;
                } else {
                    self.history[self.history_pos] = actual.clone();
                    self.history_pos += 1;
                    if self.history_pos == self.opts.history_len {
                        self.history_pos = 0;
                    }   
                    
                }
            }

      

            let cpu_cycles = op.execute(self, &am)?;
            let ppu_cycles = cpu_cycles * 3;

            let scan_row_before = self.ppu.scan_row;

            for _ in 0..ppu_cycles {
                ppu::tick(self)?;
            }

            if scan_row_before < 241 && self.ppu.scan_row >= 241 {
                // We have a new frame to draw!
                return Ok(actual);
            }

            //  eprintln!();
           
            // let cpu = &mut self.cpu;
            // cpu.execute(self, op, am);
        // }
        }
    }

    pub fn print_stack(&mut self) -> Result<()> {
        let mut stderr = tc::StandardStream::stderr(tc::ColorChoice::AlwaysAnsi);
        
        write!(&mut stderr, "   ")?;
        for x in 0..0x10 {
            if self.cpu.sp & 0x0f == x {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
            } else {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
            }
            write!(&mut stderr, "{x:02x} ")?;
        }
        writeln!(&mut stderr)?;

        for y in 0..0x10_u8 {
            
            let base = y << 4;
            if self.cpu.sp & 0xf0 == base {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
            } else {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
            }
            write!(&mut stderr, "{base:02x} ").unwrap();
            stderr.set_color(&ColorSpec::new())?;
            for x in 0..0x10 {
                //if y == 0
                let addr = base + x;
                if self.cpu.sp == addr {
                    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_intense(true))?;
                }
                write!(&mut stderr, "{:02x} ", self.read_byte(CPU::addr_stack(addr))?)?;

                if self.cpu.sp == addr {
                    stderr.set_color(&ColorSpec::new())?;
                }
            }
            writeln!(&mut stderr)?;
        }
        Ok(())
    }

    fn trigger_nmi(&mut self) {
        self.nmi = true;
    }





}

pub fn dump_mem<'a, T>(source: T, curr_address: Option<addr::Addr>) -> Result<()> 
    where T: IntoIterator<Item = &'a u8>
{
    let mut stderr = tc::StandardStream::stderr(tc::ColorChoice::AlwaysAnsi);
    
    write!(&mut stderr, "    ")?;
    for x in 0..0x10 {
        // if curr_address.is_some_and(|ca| ca & 0xf == x) {
        if curr_address.is_some() && curr_address.unwrap().0 & 0xf == x {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
        } else {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
        }
        write!(&mut stderr, " {x:01x} ")?;
    }
    writeln!(&mut stderr)?;

    let mut chars = ['.'; 16];

    for (pos, value) in source.into_iter().copied().enumerate() {
        let x = pos & 0xf;
        let y = (pos & 0xfff0) as u16;

        if x == 0 {
            if curr_address.is_some() && curr_address.unwrap().0 & 0xfff0 == y  {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
            } else {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
            }
            write!(&mut stderr, " {:02x} ", y >> 4).unwrap();
        }
        
        if value == 0x00 {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
        } else if value == 0xff {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_intense(true))?;
        } else {
            stderr.set_color(&ColorSpec::new())?;
        }
        write!(&mut stderr, "{:02x} ", value)?;

        let val_char = value as char;

        chars[x] = val_char;

        if x == 0xf {
            for char in chars {
                if char.is_ascii_graphic() {
                    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_intense(true))?;
                    write!(&mut stderr, "{char}")?;
                } else {

                    match char as u8 {
                        0 => {
                            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
                        },
                        255 => {
                            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_intense(true))?;
                        }
                        _ => {
                            stderr.set_color(&ColorSpec::new())?;
                            
                        }
                    }
                    write!(&mut stderr, ".")?;
                }
                // ▮
            }
            writeln!(&mut stderr)?;
            chars = ['.'; 16];
        }
    }
    stderr.set_color(&ColorSpec::new())?;
    Ok(())
}