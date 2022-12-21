use crate::system::cpu;
use std::fmt::{self, Write};


pub(crate) struct ExecutionState {
    pub cpu: cpu::CPU,
    pub am: cpu::AddressMode,
    pub pc_bytes: Vec<u8>,
    pub ppu: (u16, u16),
    pub cycles: u64,
}

impl fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        
        let bc = self.pc_bytes[0];
        let op_byte_name = cpu::opcode_map::format_op_byte(bc);

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
        // let cycles = 0;
        let cycles = self.cycles;


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