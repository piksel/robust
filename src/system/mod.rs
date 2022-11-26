use std::{io::{self, Read, BufRead, Write as IOWrite}, fs, fmt::{self, Write}};

use crate::system::cpu::opcode_map::format_op_byte;

use self::cpu::{CPU, AddressMode};

use self::cart::Cart;
use anyhow::{Result, Ok, bail};

use tc::{WriteColor, ColorSpec, Color};
use termcolor as tc;

pub mod cart;
pub mod bus;
pub mod cpu;

type ParseIntResult<T> = std::result::Result<T, std::num::ParseIntError>;

pub struct System {
    ram: Vec<u8>,
    ppu: Vec<u8>,
    apu: Vec<u8>,
    pub(crate) cpu: CPU,
    pub(crate) cart: Option<Cart>,
    expect_log: Option<Vec<String>>
}

impl System {
    pub fn new() -> Self {
        let cpu = CPU::init();
        
        System {
            ram: vec![0; 2048],
            ppu: vec![0; 8],
            apu: vec![0; 0x18],
            cpu,
            cart: None,
            expect_log: None
        }
    }

    pub(crate) fn load_cart(&mut self, cart_file: &fs::File) -> Result<()> {
        let reader = io::BufReader::new(cart_file);
        let cart = Cart::new(reader.bytes())?;

        // for b in cart.prg_rom.iter() {
        //     print!("{b:02x} ")
        // }
        // println!();

        self.cart = Some(cart);

        eprintln!("Cart loaded!");


        Ok(())
    }

    pub(crate) fn run(&mut self, steps: u32) -> Result<()> {

        let mut stderr = tc::BufferWriter::stderr(tc::ColorChoice::AlwaysAnsi);
        let mut bad_colors = tc::ColorSpec::new();
        bad_colors.set_fg(Some(tc::Color::Red));

        let mut good_colors = tc::ColorSpec::new();
        good_colors.set_fg(Some(tc::Color::Green));

        let mut dark_colors = tc::ColorSpec::new();
        dark_colors.set_fg(Some(tc::Color::Black)).set_intense(true);

        let norm_colors = tc::ColorSpec::new();

        // init program counter (for use with test cart)
        self.cpu.pc = 0xc000;

        let mut cycles = 0u64;

        for step in 0..steps {

            // print!("{step:03}:  PC: {pc:04x}  A:{a:02x} X:{x:02x} Y:{y:02x}  SP: {sp:02x}  Flags: ");
            // print!("{pc:04X}  A:{a:02X} X:{x:02X} Y:{y:02X} SP: {sp:02x}  ");
            //print!("");

            //println!("-- [ Step {step} ] ---------------------------------------------------------");

            if let Some(xlog) = &self.expect_log {
                println!("\nRW {}", xlog[step as usize]);
            }


            let expected = self.get_expected_state(step as usize).unwrap();

            
            

            let (op, am, bc) = self.cpu.load(&self);

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
                 cycles,
                 ppu: (0, 0)
            };

            let actual_log = actual.to_string();
            let expected_log = expected.to_string();
            
            

            if actual_log != expected_log {

                let mut buff_expected = stderr.buffer();
                let mut buff_actual = stderr.buffer();

                buff_actual.set_color(&bad_colors)?;
                write!(&mut buff_actual, "!! ")?;

                for (a, e) in actual_log.chars().zip(expected_log.chars()) {
                    if a != e {
                        buff_actual.set_color(&bad_colors)?;
                        buff_expected.set_color(&good_colors)?;
                    } else {
                        buff_actual.set_color(&norm_colors)?;
                        buff_expected.set_color(&dark_colors)?;
                    }
                    write!(&mut buff_actual, "{a}")?;
                    write!(&mut buff_expected, "{e}")?;
                    // print!("{}", if a == e {' '} else {e});

                }

                buff_actual.set_color(&tc::ColorSpec::new().set_fg(Some(tc::Color::White)).set_intense(true))?;
                write!(&mut buff_actual, "")?;
                buff_expected.set_color(&norm_colors)?;
                writeln!(&mut buff_expected)?;
                // println!();
                stderr.print(&buff_actual)?;
                stderr.print(&buff_expected)?;

                // if let Some(xlog) = &self.expect_log {
                //     println!("RW {}", xlog[step as usize]);
                // }
                

                //panic!("Actual log did not match expected!")
            } else {
                let mut buff = stderr.buffer();
                buff.set_color(&good_colors)?;
                write!(&mut buff, "OK ")?;
                buff.set_color(&norm_colors)?;
                writeln!(&mut buff, "{actual_log} {step}")?;
                stderr.print(&buff)?;
            }

            if false {

         
            if expected.cpu.a  != actual.cpu.a  { bail!("Accumulator desync")};
            if expected.cpu.x  != actual.cpu.x  { bail!("X register desync")};
            if expected.cpu.y  != actual.cpu.y  { bail!("Y register desync")};
            if expected.cpu.pc != actual.cpu.pc { bail!("Program Counter desync")};
            if expected.cpu.status() != actual.cpu.status() {bail!("CPU status flags desync")};
            if expected.am != actual.am {bail!("Address Mode desync")};
        }
            

            cycles += CPU::execute(self, op, am) as u64;
            // let cpu = &mut self.cpu;
            // cpu.execute(self, op, am);
        }

        Ok(())
    }

    pub(crate) fn print_stack(&self) -> Result<()> {
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

    pub(crate) fn load_expected_log(&mut self, file: &fs::File) -> Result<()> {

        let lines = io::BufReader::new(file).lines();
        self.expect_log = Some(lines.collect::<io::Result<Vec<String>>>()?);

        Ok(())
    }

    fn get_expected_state(&self, step: usize) -> Result<ExecutionState> {
        let xlog = self.expect_log.as_ref().unwrap();
        let row = &xlog[step];

        let chars = &mut row.chars();

        let mut pc_bytes = [0, 0, 0];
        let mut ppu = (0, 0);
        let mut cpu = CPU::init();

        cpu.pc = u16::from_str_radix(&chars.take(4).collect::<String>(), 16)?;

        let pc_bytes: Vec<u8> = [
            chars.skip(2).take(2).collect::<String>().trim(),
            chars.skip(1).take(2).collect::<String>().trim(),
            chars.skip(1).take(2).collect::<String>().trim(),
        ].iter()
            .take_while(|s| !s.is_empty())
            .map(|s| u8::from_str_radix(s, 16))
            .collect::<ParseIntResult<Vec<u8>>>()?;


        let byte_count = pc_bytes.len();
        
        
        let op_name: String = chars.skip_while(|c| c.is_whitespace() || *c == '*')
            .take(3).collect();

        assert_eq!(format_op_byte(pc_bytes[0]), op_name);

        let am = AddressMode::from_str_with_bc(&chars.skip(1).take(26).collect::<String>(), byte_count)?;

        assert_eq!(
            &chars.skip_while(|c| c.is_whitespace()).take(2).collect::<String>(),
            "A:"
        );
        cpu.a = u8::from_str_radix(&chars.take(2).collect::<String>(), 16).unwrap();

    
        assert_eq!(&chars.skip(1).take(2).collect::<String>(), "X:");
        cpu.x = u8::from_str_radix(&chars.take(2).collect::<String>(), 16).unwrap();
        
        assert_eq!(&chars.skip(1).take(2).collect::<String>(), "Y:");
        cpu.y = u8::from_str_radix(&chars.take(2).collect::<String>(), 16).unwrap();
        
        assert_eq!(&chars.skip(1).take(2).collect::<String>(), "P:");
        cpu.set_status(u8::from_str_radix(&chars.take(2).collect::<String>(), 16).unwrap());

        assert_eq!(&chars.skip(1).take(3).collect::<String>(), "SP:");
        cpu.sp = u8::from_str_radix(&chars.take(2).collect::<String>(), 16).unwrap();

        assert_eq!(&chars.skip(1).take(4).collect::<String>(), "PPU:");
        ppu.0 = u16::from_str_radix(&chars
            .take(3).collect::<String>().trim(), 10).unwrap();
        assert_eq!(chars.next().unwrap(), ',');
        ppu.1 = u16::from_str_radix(&chars
                .take(3).collect::<String>().trim(), 10).unwrap();

        assert_eq!(&chars.skip_while(|c| c.is_whitespace()).take(4).collect::<String>(), "CYC:");
        
        let cycles = u64::from_str_radix(&chars.take_while(char::is_ascii_digit).collect::<String>(), 10).unwrap();

        Ok(ExecutionState { cpu, pc_bytes, am, cycles, ppu })
    }

}

struct ExecutionState {
    cpu: CPU,
    am: AddressMode,
    pc_bytes: Vec<u8>,
    ppu: (u16, u16),
    cycles: u64,
}

impl fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        
        let bc = self.pc_bytes[0];
        let op_byte_name = format_op_byte(bc);

        let byte_count = self.am.bytes();
        let b1 = if byte_count > 0 {
            format!("{:02X}", self.pc_bytes[1])
        } else {"  ".to_owned()};
        let b2 = if byte_count > 1 {
            format!("{:02X}", self.pc_bytes[2])
        } else {"  ".to_owned()};
    
        let op_addr = "";
    
        // Use dummy PPU for now
        let ppu = format!("   ,   ");
        // let ppu = format!("{:>3},{:>3}", self.ppu.0, self.ppu.1);
    
        let p = self.cpu.status();
        let pc = self.cpu.pc;
        let a = self.cpu.a;
        let x = self.cpu.x;
        let y = self.cpu.y;
        let sp = self.cpu.sp;

        // Use dummy cycles for now
        let cycles = 0;
        // let cycles = self.cycles;


        // op_addr should be 27 long if used
        f.write_fmt(format_args!("{pc:04X}  {bc:02X} {b1} {b2}  {op_byte_name} {op_addr} "))?;
        f.write_fmt(format_args!("A:{a:02X} X:{x:02X} Y:{y:02X} P:{p:02X} SP:{sp:02X} PPU:{ppu} CYC:{cycles:<6}"))?;
    
        // print!("Flags:");
        f.write_str("  ")?;
    
        f.write_char(if self.cpu.sign {'-'} else {'+'})?;
    
        f.write_char(if self.cpu.overflow {'O'} else {'-'})?;
    
        f.write_char(' ')?;
 
        f.write_char(if self.cpu.soft_break {'B'} else {'-'})?;
    
        f.write_char(if self.cpu.decimal {'D'} else {'-'})?;
    
        f.write_char(if self.cpu.interrupt {'I'} else {'-'})?;
    
        f.write_char(if self.cpu.zero {'Z'} else {'-'})?;
    
        f.write_char(if self.cpu.carry {'C'} else {'-'})?;
    
    
    
    
    
    
    
    
    
    
    
        
        // print!(" P:{p:08b}");
    
        // print!("  {bc:02x}  {op:26} {:16}", am.format(bytes));
        f.write_fmt(format_args!("  {:18}", format!("{:?}", self.am)))?;
        // f.write_fmt(format_args!("  {op:26} {:16}", format!("{am:?}")));
    
        // let debug_02 = self.read_byte(0x02);
        // let debug_03 = self.read_byte(0x03);
    

        // println!("  {debug_02:02x}  {debug_03:02x}");
        fmt::Result::Ok(())
    }
}