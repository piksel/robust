use crate::system::{System, addr::Addr};

use anyhow::Result;

pub(crate) mod opcode;
pub(crate) mod opcode_map;

pub(crate) use opcode_map::load;

#[derive(Clone)]
pub struct CPU {
    pub(crate) pc: Addr,
    pub(crate) sp: u8,

    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) a: u8,

    // status register
    pub(crate) carry: bool,
    pub(crate) zero: bool,
    pub(crate) interrupt: bool,
    pub(crate) decimal: bool,
    pub(crate) soft_break: bool,
    // reserved
    pub(crate) overflow: bool,
    pub(crate) sign: bool,
}

impl CPU {
    pub fn init() -> Self {
        CPU {
            pc: Addr(0),
            sp: ((CPU::STACK_TOP - CPU::STACK_BOT) - 2u16).lsb(),
            x: 0,
            y: 0,
            a: 0,
            carry: false,
            zero: false,
            interrupt: true,
            decimal: false,
            soft_break: false,
            overflow: false,
            sign: false,
        }
    }

    fn index_reg(&self, base: Addr, reg: &Option<Register>) -> Addr {
        match reg {
            None => base,
            Some(Register::X) => base + (self.x as u16),
            Some(Register::Y) => base + (self.y as u16), // + (self.y as u16),
            Some(r) => panic!("indexing on register {r:?} is not implemented")
        }
    }

    fn set_reg(&mut self, register: &Register, value: u8) {
        match register {
            Register::X => self.x = value,
            Register::Y => self.y = value,
            Register::A => self.a = value,
            Register::SP => self.sp = value,
        }
    }

    fn get_reg(&self, register: &Register) -> u8 {
        match register {
            Register::X => self.x,
            Register::Y => self.y,
            Register::A => self.a,
            Register::SP => self.sp,
        }
    }

    pub fn stack_push_word(sys: &mut System, value: u16) -> anyhow::Result<()> {
        // eprintln!("Pushing word {value:04x} to stack at {:02x}", sys.cpu.sp);
        Self::stack_push_byte(sys, (value >> 8) as u8)?;
        Self::stack_push_byte(sys, value as u8)?;
        Ok(())
    }

    pub fn stack_push_byte(sys: &mut System, value: u8) -> anyhow::Result<()> {
        // eprintln!("Pushing byte {value:02x} to stack at {:02x}", sys.cpu.sp);
        let addr = CPU::addr_stack(sys.cpu.sp);
        sys.write_byte(addr, value);
        if let Some(sp) = sys.cpu.sp.checked_sub(1) {
            sys.cpu.sp = sp;
            Ok(())
        } else {
            anyhow::bail!("stack overflow!")
        }
    }

    fn stack_pull_word(sys: &mut System) -> u16 {
        sys.cpu.sp = sys.cpu.sp.checked_add(1).expect("stack overflow!");
        let addr = CPU::addr_stack(sys.cpu.sp);
        let value = sys.read_word(addr);
        sys.cpu.sp = sys.cpu.sp.checked_add(1).expect("stack overflow!");

        // println!("{value:04x} read from stack at {addr:04x}");
        value
    }

    
    fn stack_pull_byte(sys: &mut System) -> u8 {
        sys.cpu.sp = sys.cpu.sp.checked_add(1).expect("stack overflow!");
        let addr = CPU::addr_stack(sys.cpu.sp);
        let value = sys.read_byte(addr);
        // println!("{value:04x} read from stack at {addr:04x}");
        value
    }

    pub const STACK_BOT: Addr = Addr(0x0100);
    const STACK_TOP: Addr = Addr(0x01ff);
    const ADDR_BREAK: Addr = Addr(0xfffe);

    pub(crate) fn addr_stack(sp: u8) -> Addr {
        CPU::STACK_BOT + (sp as u16)
    }

    fn get_flag(&self, flag: &Flag) -> bool {
        match flag {
            Flag::Carry => self.carry,
            Flag::Zero => self.zero,
            Flag::Interrupt => self.interrupt,
            Flag::Decimal => self.decimal,
            Flag::Break => self.soft_break,
            Flag::Overflow => self.overflow,
            Flag::Negative => self.sign,
        }
    }

    pub(crate) fn status(&self) -> u8 {
        0
        | if self.carry      {1 << 0} else {0} 
        | if self.zero       {1 << 1} else {0} 
        | if self.interrupt  {1 << 2} else {0} 
        | if self.decimal    {1 << 3} else {0} 
        | if self.soft_break {1 << 4} else {0} 
        | 1 << 5 // always set
        | if self.overflow   {1 << 6} else {0} 
        | if self.sign       {1 << 7} else {0}
    }

    pub(crate) fn set_status(&mut self, flags: u8) {
        self.carry      = ((flags >> 0) & 1) != 0;
        self.zero       = ((flags >> 1) & 1) != 0;
        self.interrupt  = ((flags >> 2) & 1) != 0;
        self.decimal    = ((flags >> 3) & 1) != 0;
        // BREAK should never be manually set
        // self.soft_break = ((flags >> 4) & 1) != 0;
        self.overflow   = ((flags >> 6) & 1) != 0;
        self.sign       = ((flags >> 7) & 1) != 0;
    }

    fn update_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.sign = (value & 0b10000000) != 0;
    }

}

#[cfg(test)]
mod tests {
    use crate::system::cpu::CPU;

    #[test]
    fn cpu_flags_roundtrip() {
        
        
        fn test_roundtrip(sent: u8) {
            let mut cpu = CPU::init();
            cpu.set_status(sent);
            let actual = cpu.status();

            let expected = sent & 0b1110_1111;

            assert_eq!(expected, actual, "{expected:08b} != {actual:08b}");
        }

        test_roundtrip(0b1111_1111);
        test_roundtrip(0b0010_0000);
        test_roundtrip(0b0010_1111);
        test_roundtrip(0b1010_1010);
    }
}

pub(crate) fn get_addr_ro(sys: &mut System, address_mode: &AddressMode) -> AddrLookup {
    match address_mode {
        AddressMode::Zero(reg) => addr_zero(sys, reg),
        AddressMode::Absolute(reg) => addr_absolute(sys, reg),
        AddressMode::Immediate => addr_immediate(sys),
        AddressMode::Indirect(reg) => addr_indirect(sys, reg),
        address_mode => panic!("getting address mode {address_mode:?} is not implemented")
    } 
}

pub(crate) fn resolve_addr(sys: &mut System, address_mode: &AddressMode) -> Addr {
    let AddrLookup {addr, crossed_page} = get_addr_ro(sys, address_mode);
    sys.cycles += default_cycles(address_mode, crossed_page);
    addr
}

pub(crate) fn resolve_addr_with_xp(sys: &mut System, address_mode: &AddressMode, xp: bool) -> Addr {
    let addr = get_addr_ro(sys, address_mode).addr;
    sys.cycles += default_cycles(address_mode, xp);
    addr
}

pub(crate) fn default_cycles(address_mode: &AddressMode, xp: bool) -> u64 {
    match address_mode { 
        AddressMode::Register(_) => 0,
        AddressMode::Immediate => 2,
        AddressMode::Zero(None) => 3,
        AddressMode::Absolute(Some(_)) if xp => 5,
        AddressMode::Absolute(Some(_)) => 4,
        AddressMode::Indirect(Some(Register::X)) => 6,
        AddressMode::Indirect(Some(Register::Y)) if xp => 6,
        AddressMode::Indirect(Some(Register::Y)) => 5,
         _ => 4
    }
}

pub(crate) fn addr_immediate(sys: &mut System) -> AddrLookup {
    let addr = sys.cpu.pc;
    sys.cpu.pc += 1i8;
    AddrLookup { addr, crossed_page: false }

}

pub(crate) fn addr_zero(sys: &mut System, reg: &Option<Register>) -> AddrLookup {
    let offset = match reg {
        None => 0,
        Some(r) => sys.cpu.get_reg(r)
    };

    // Zero-extended address
    let base = shift_pc(sys);

    let sum = base.wrapping_add(offset);
    // eprintln!("Base: {base:04x} offset: {offset:02x} sum:{sum:02x}");
    let addr = Addr::from_zero(sum);

    // TODO
    let crossed_page = false;

    AddrLookup { addr, crossed_page }
    
    // sys.cpu.index_reg(base, reg)
}

pub(crate) fn addr_absolute(sys: &mut System, reg: &Option<Register>) -> AddrLookup {
    let base = shift_pc_addr(sys);
    let addr = sys.cpu.index_reg(base, reg);

    AddrLookup { addr, crossed_page: !addr.same_page_as(base) }
}

pub(crate) fn addr_indirect(sys: &mut System, reg: &Option<Register>) -> AddrLookup {
    match reg {
        None => {
            let lsb_addr = shift_pc_addr(sys);
            let addr_lsb = sys.read_byte(lsb_addr);

            // This is a bug, but we will have to implement it as well
            let addr_msb = sys.read_byte(if lsb_addr.lsb() == 0xff{
                lsb_addr - 0xff
            } else {
                lsb_addr + 1
            });
            let addr = Addr::from_bytes(addr_msb, addr_lsb);
            // let next_c = sys.read_byte(addr - 0xff);
            // eprintln!("addr1: {addr:04x} a: {next_a:02x} b: {next_b:02x} next: {next:04x}");
            AddrLookup { addr, crossed_page: !addr.same_page_as(lsb_addr) }
        },
        Some(Register::X) => {
            let lsb_addr = shift_pc(sys);
            let msb_addr = sys.cpu.x;
            let meta_addr = lsb_addr.wrapping_add(msb_addr);
            let addr = sys.read_zero_word(meta_addr).into();
            AddrLookup{addr, crossed_page: false}
        }
        Some(Register::Y) => {
            let lsb_addr = shift_pc(sys);
            let addr_lsb = sys.read_zero_addr(lsb_addr); // FFFF
            
            let msb_addr = sys.cpu.y;
            
            let addr = addr_lsb + (msb_addr as u16);
            // eprintln!("addr1: {addr_lsb:04x} lsb: {lsb_addr:02x} msb: {msb_addr:02x} addr: {addr:04x}");
            let crossed_page = !addr.same_page_as(addr_lsb);
  
            AddrLookup{addr, crossed_page}
        }
        Some(r) => panic!("cannot read indirect from register {r:?}")
    }
}

pub fn addr_relative(sys: &mut System) -> i8 {
    shift_pc(sys) as i8
}

pub fn shift_pc(sys: &mut System) -> u8 {
    let val = sys.read_byte(sys.cpu.pc);
    sys.cpu.pc += 1i8;
    val
}

pub fn shift_pc_addr(sys: &mut System) -> Addr {
    let lsb = shift_pc(sys);
    let msb = shift_pc(sys);
    Addr::from_bytes(msb, lsb)
}

pub(crate) struct AddrLookup {
    addr: Addr,
    crossed_page: bool
}

#[derive(Debug)]
pub(crate) enum Flag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    #[allow(dead_code)]
    Break,
    Overflow,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Register {
    A,
    X,
    Y,
    SP,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressMode {
    Absolute(Option<Register>),
    Zero(Option<Register>),
    Indirect(Option<Register>),
    Relative,
    Immediate,
    Implied,
    Register(Register)
}

impl AddressMode {
    pub fn bytes(&self) -> usize {
        match self {
            Self::Implied => 0,
            Self::Zero(_) => 1,
            Self::Indirect(Some(_)) => 1,
            Self::Indirect(None) => 2,
            Self::Absolute(_) => 2,
            Self::Immediate => 1,
            Self::Relative => 1,
            Self::Register(_) => 0,
        }
    }

    pub fn from_str_with_bc(s: &str, bc: usize) -> Result<Self> {
        let chars = &mut s.chars();
        match (chars.nth(0), chars.nth(2)) {
            (Some(' '), _) => Ok(Self::Implied),
            (Some('A'), _) => Ok(Self::Register(Register::A)),
            (Some('#'), _) => Ok(AddressMode::Immediate),
            (Some('('), _) => match (chars.nth(1), chars.nth(0)) {
                (Some('X'), _) => Ok(AddressMode::Indirect(Some(Register::X))),
                (_, Some('Y')) => Ok(AddressMode::Indirect(Some(Register::Y))),
                (_, Some(')')) => Ok(AddressMode::Indirect(None)),
                (a, b) => anyhow::bail!("unknown asm syntax: {s} ( {a:?} {b:?}")
            },
            (Some('$'), Some(c)) => match c {
                ' ' => Ok(AddressMode::Zero(None)),
                '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|
                '8'|'9'|'A'|'B'|'C'|'D'|'E'|'F' => {
                    match bc {
                        3 => match (chars.nth(1), chars.nth(0)) {
                            (Some(' '), _) => Ok(AddressMode::Absolute(None)),
                            (Some(','), Some('X')) => Ok(AddressMode::Absolute(Some(Register::X))),
                            (Some(','), Some('Y')) => Ok(AddressMode::Absolute(Some(Register::Y))),
                            // Some(c) => anyhow::bail!("unknown byte count: {c}"),
                            (a, b) => anyhow::bail!("unknown asm syntax: {a:?} {b:?}")
                        },
                        2 => Ok(AddressMode::Relative),
                        _ => anyhow::bail!("unknown byte count: {bc}")
                    }
                }
                ',' => match chars.nth(0) {
                    Some('X') => Ok(AddressMode::Zero(Some(Register::X))),
                    Some('Y') => Ok(AddressMode::Zero(Some(Register::Y))),
                    c => anyhow::bail!("unknown asm syntax: {s} ({c:?})")
                }
                c => anyhow::bail!("unknown asm syntax: {s} ({c:?})")
            },
            (a, b) => anyhow::bail!("unknown asm syntax: {s} {a:?} {b:?}")
        }
    }
}

impl std::str::FromStr for AddressMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = &mut s.chars();
        match (chars.nth(0), chars.nth(2)) {
            (Some(' '), _) => Ok(AddressMode::Implied),
            (Some('A'), _) => Ok(AddressMode::Register(Register::A)),
            (Some('#'), _) => Ok(AddressMode::Immediate),
            (Some('$'), Some(c)) => match c {
                ' ' => Ok(AddressMode::Zero(None)),
                '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|
                '8'|'9'|'A'|'B'|'C'|'D'|'E'|'F' => {
                    Ok(AddressMode::Absolute(None))
                }
                _ => anyhow::bail!("unknown asm syntax")
            },
            _ => anyhow::bail!("unknown asm syntax")
        }
    }
}