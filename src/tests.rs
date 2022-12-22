use anyhow::{bail, Result};
use termcolor::{BufferWriter, ColorSpec, Color, ColorChoice, WriteColor};
use std::{fs, io::{self, Write as IOWrite, BufRead}, fmt::{self, Write}};

type ParseIntResult<T> = std::result::Result<T, std::num::ParseIntError>;

use crate::system::{self, cpu, execution_state::ExecutionState, addr::Addr};

#[test]
fn matches_nestest() {
    assert!(run_with_expect_log(
        "carts/nestest.nes", 
        "carts/nestest.log", 
        0xc000, 
        8991).is_ok());
    

}

fn run_with_expect_log(cart_file: &str, log_file: &str, start_pc: u16, steps: usize) -> Result<()> {
    let mut system = system::System::new();

    let cart_file = fs::File::open(cart_file)?;
    let log_file = fs::File::open(log_file)?;

    let state_log = StateLog::from_file(&log_file)?;

    
    system.load_cart(&cart_file)?;



    let stderr = BufferWriter::stderr(ColorChoice::AlwaysAnsi);
    let mut bad_colors = ColorSpec::new();
    bad_colors.set_fg(Some(Color::Red));

    let mut good_colors = ColorSpec::new();
    good_colors.set_fg(Some(Color::Green));

    let mut dark_colors = ColorSpec::new();
    dark_colors.set_fg(Some(Color::Black)).set_intense(true);

    let norm_colors = ColorSpec::new();

    // init program counter (for use with test cart)
    system.cpu.pc = Addr(start_pc);

    let mut cycles = 0u64;

    for step in 0..steps {

        // print!("{step:03}:  PC: {pc:04x}  A:{a:02x} X:{x:02x} Y:{y:02x}  SP: {sp:02x}  Flags: ");
        // print!("{pc:04X}  A:{a:02X} X:{x:02X} Y:{y:02X} SP: {sp:02x}  ");
        //print!("");

        //println!("-- [ Step {step} ] ---------------------------------------------------------");

        // if let Some(xlog) = &state_log.expect_log {
        //     eprintln!("\nRW {}", xlog[step as usize]);
        // }


        let expected = state_log.get_expected_state(step as usize).unwrap();

        
        

        let cpu = system.cpu.clone();
        let (op, am, bc) = cpu::load(&mut system);

        let byte_count = am.bytes() + 1;
        let pc_bytes = (0..byte_count).map(|i| system.peek_byte(cpu.pc + (i as i8))).collect();
        
        let actual = ExecutionState {
                cpu, 
                pc_bytes, 
                am: am.clone(), 
                cycles: system.cycles,
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

            buff_actual.set_color(&ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;
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

        if true {

            compare_states(expected, actual).map_err(|err| {
                let ram = &system.ram;
                let pc = system.cpu.pc;
                // let _ = crate::system::dump_mem(ram, Some(pc));
                // let _ = system.print_stack();
                err
            })?;

        }
        

        op.execute(&mut system, &am);
        // let cpu = &mut self.cpu;
        // cpu.execute(self, op, am);
    }

    assert_eq!(system.peek_byte(02), 0);
    assert_eq!(system.peek_byte(03), 0);

    Ok(())

}

fn compare_states(expected: ExecutionState, actual: ExecutionState) -> Result<(), anyhow::Error> {
    if expected.cycles != actual.cycles  { bail!("Cycle desync")};
    if expected.cpu.a  != actual.cpu.a   { bail!("Accumulator desync")};
    if expected.cpu.x  != actual.cpu.x   { bail!("X register desync")};
    if expected.cpu.y  != actual.cpu.y   { bail!("Y register desync")};
    if expected.cpu.pc != actual.cpu.pc  { bail!("Program Counter desync")};
    if expected.cpu.status() != actual.cpu.status() {bail!("CPU status flags desync")};
    if expected.am != actual.am {bail!("Address Mode desync")};
    return Ok(())
}



struct StateLog{
    expect_log: Option<Vec<String>>
}

impl StateLog {
    fn from_file(file: &fs::File) ->  Result<Self> {
        let lines = io::BufReader::new(file).lines();
        let expect_log = Some(lines.collect::<io::Result<Vec<String>>>()?);
    
        Ok(StateLog{expect_log})
    }

    fn get_expected_state(&self, step: usize) -> Result<ExecutionState> {
        let xlog = self.expect_log.as_ref().unwrap();
        let row = &xlog[step];
    
        let chars = &mut row.chars();
    
        let mut pc_bytes = [0, 0, 0];
        let mut ppu = (0, 0);
        let mut cpu = cpu::CPU::init();
    
        cpu.pc = u16::from_str_radix(&chars.take(4).collect::<String>(), 16)?.into();
    
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
    
        assert_eq!(cpu::opcode_map::format_op_byte(pc_bytes[0]), op_name);
    
        let am = cpu::AddressMode::from_str_with_bc(&chars.skip(1).take(26).collect::<String>(), byte_count)?;
    
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

