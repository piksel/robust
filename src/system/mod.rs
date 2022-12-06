use std::{io::{self, Read, BufRead, Write as IOWrite}, fs, fmt::{self, Write}};

use crate::system::cpu::opcode_map::format_op_byte;

use self::{cpu::{CPU, AddressMode}, execution_state::ExecutionState};

use self::cart::Cart;
use anyhow::{Result, Ok, bail};

use tc::{WriteColor, ColorSpec, Color};
use termcolor as tc;

pub mod cart;
pub mod bus;
pub mod cpu;
pub mod execution_state;
pub mod ppu;

pub struct System {
    ram: Vec<u8>,
    pub(crate) ppu: ppu::PPU,
    apu: Vec<u8>,
    pub(crate) cpu: CPU,
    pub(crate) cart: Option<Cart>,
    pub(crate) cycles: u64,
}

impl System {
    pub fn new() -> Self {
        let cpu = CPU::init();
        
        System {
            ram: vec![0; 2048],
            ppu: ppu::PPU::init(),
            apu: vec![0; 0x18],
            cpu,
            cart: None,
            cycles: 0,
        }
    }

    pub(crate) fn load_cart(&mut self, cart_file: &fs::File) -> Result<()> {
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

    pub fn reset(&mut self) {
        // Read reset vector
        let rv = self.read_word(0xfffc);
        self.cpu.pc = rv;
    }

    pub(crate) fn run_cycle(&mut self) -> Result<()> {

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

            let (op, am, bc) = cpu::load(self);

            let byte_count = am.bytes();
            let pc_bytes = vec![
                bc,
                if byte_count > 0 {self.read_byte(self.cpu.pc.wrapping_add(1))} else {0},
                if byte_count > 1 {self.read_byte(self.cpu.pc.wrapping_add(2))} else {0},
            ];

            let actual = ExecutionState {
                 cpu: self.cpu.clone(), 
                 pc_bytes, 
                 am: am.clone(), 
                 ppu: (self.ppu.scan_row as u16, self.ppu.scan_line),
                 cycles: self.cycles,
            };
            // if self.ppu.scan_row > 240 {

                // let mut buff = stderr.buffer();
                // buff.set_color(&good_colors)?;
                // eprint!(&mut buff, "OK ")?;
                // buff.set_color(&norm_colors)?;
                
                // stderr.print(&buff)?;

                let actual_log = actual.to_string();
                eprintln!("{actual_log}");
            // }


            
      

            let cpu_cycles = op.execute(self, &am);
            let ppu_cycles = cpu_cycles * 3;

            let scan_row_before = self.ppu.scan_row;

            for _ in 0..ppu_cycles {
                ppu::tick(self);
            }

            if scan_row_before != 0 && self.ppu.scan_row == 0 {
                // We have a new frame to draw!
                break;
            }

            //  eprintln!();
           
            // let cpu = &mut self.cpu;
            // cpu.execute(self, op, am);
        // }
        }

        Ok(())
    }

    pub(crate) fn print_stack(&mut self) -> Result<()> {
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
                write!(&mut stderr, "{:02x} ", self.read_byte(CPU::addr_stack(addr)))?;

                if self.cpu.sp == addr {
                    stderr.set_color(&ColorSpec::new())?;
                }
            }
            writeln!(&mut stderr)?;
        }
        Ok(())
    }

}

pub(crate) fn dump_mem<T>(source: T, curr_address: Option<u16>) -> Result<()> 
    where T: IntoIterator<Item = u8>
{
    let mut stderr = tc::StandardStream::stderr(tc::ColorChoice::AlwaysAnsi);
    
    write!(&mut stderr, "    ")?;
    for x in 0..0x10 {
        // if curr_address.is_some_and(|ca| ca & 0xf == x) {
        if curr_address.is_some() && curr_address.unwrap() & 0xf == x {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
        } else {
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
        }
        write!(&mut stderr, "{x:02x} ")?;
    }
    writeln!(&mut stderr)?;

    let mut chars = ['.'; 16];

    for (pos, value) in source.into_iter().enumerate() {
        let x = pos & 0xf;
        let y = (pos & 0xfff0) as u16;

        if x == 0 {
            if curr_address.is_some() && curr_address.unwrap() & 0xfff0 == y  {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
            } else {
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_intense(true))?;
            }
            write!(&mut stderr, "{y:03x} ").unwrap();
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

        chars[x] = if val_char.is_ascii_graphic() {val_char} else {'.'};

        if x == 0xf {

            writeln!(&mut stderr, "{}", String::from_iter(chars))?;
            chars = ['.'; 16];
        }
    }

    // for y in 0..0x10_u8 {
        
        

        
    //     ;
    //     for x in 0..0x10 {
    //         //if y == 0
    //         let addr = base + x;
    //         if self.cpu.sp == addr {
    //             stderr.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_intense(true))?;
    //         }
    //         write!(&mut stderr, "{:02x} ", self.read_byte(CPU::addr_stack(addr)))?;

    //         if self.cpu.sp == addr {
    //             stderr.set_color(&ColorSpec::new())?;
    //         }
    //     }
    //     writeln!(&mut stderr)?;
    // }
    Ok(())
}