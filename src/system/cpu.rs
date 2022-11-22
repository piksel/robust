use std::ops::Add;

use super::{System};

pub struct CPU {
    pub(crate) pc: u16,
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
            pc: 0,
            sp: (CPU::STACK_TOP - CPU::STACK_BOT - 2) as u8,
            x: 0,
            y: 0,
            a: 0,
            carry: false,
            zero: false,
            interrupt: false,
            decimal: false,
            soft_break: false,
            overflow: false,
            sign: false,
        }
    }

    fn addr_immediate(sys: &mut System) -> u16 {
        sys.cpu.pc += 1;
        sys.cpu.pc
    }

    fn addr_zero(sys: &mut System, reg: &Option<Register>) -> u16 {
        sys.cpu.pc += 1;
        // Zero-extended address
        let base = sys.read_byte(sys.cpu.pc) as u16;
        sys.cpu.index_reg(base, reg)
    }

    fn addr_absolute(sys: &mut System, reg: &Option<Register>) -> u16 {
        let base = sys.read_word(sys.cpu.pc + 1);
        sys.cpu.pc += 2;
        sys.cpu.index_reg(base, reg)
    }

    fn addr_indirect(sys: &mut System, reg: &Option<Register>) -> u16 {
        let addr_ref = sys.read_word(sys.cpu.pc + 1);
        sys.cpu.pc += 2;
        let base = sys.read_word(addr_ref);
        sys.cpu.index_reg(base, reg)
    }

    fn addr_relative(sys: &mut System) -> i8 {
        sys.cpu.pc += 1;
        sys.read_byte(sys.cpu.pc) as i8
    }

    fn index_reg(&self, base: u16, reg: &Option<Register>) -> u16 {
        match reg {
            None => base,
            Some(Register::X) => base + (self.x as u16),
            Some(Register::Y) => base + (self.y as u16),
            Some(r) => panic!("indexing on register {r:?} is not implemented")
        }
    }

    pub(crate) fn load(&self, sys: &System) -> (OpCode, AddressMode, u8) {
        let byte_code = sys.read_byte(self.pc);
        let (op, am) = match byte_code {
            
            // JMP
            0x4c => (OpCode::Jump, AddressMode::Absolute(None)),
            0x6c => (OpCode::Jump, AddressMode::Indirect(None)),

            // BCS, BEQ, BMI, BVS, Branch if flag is set
            0xb0 => (OpCode::BranchIf(Flag::Carry,    true), AddressMode::Relative),
            0xf0 => (OpCode::BranchIf(Flag::Zero,     true), AddressMode::Relative),
            0x30 => (OpCode::BranchIf(Flag::Negative, true), AddressMode::Relative),
            0x70 => (OpCode::BranchIf(Flag::Overflow, true), AddressMode::Relative),

            // BCC, BNE, BPL, Branch if flag is clear
            0x90 => (OpCode::BranchIf(Flag::Carry,    false), AddressMode::Relative),
            0xd0 => (OpCode::BranchIf(Flag::Zero,     false), AddressMode::Relative),
            0x10 => (OpCode::BranchIf(Flag::Negative, false), AddressMode::Relative),
            0x50 => (OpCode::BranchIf(Flag::Overflow, false), AddressMode::Relative),
            
            // JSR
            0x20 => (OpCode::JumpSub, AddressMode::Absolute(None)),

            // LDA
            0xa9 => (OpCode::Load(Register::A), AddressMode::Immediate),
            0xa5 => (OpCode::Load(Register::A), AddressMode::Zero(None)),
            0xb5 => (OpCode::Load(Register::A), AddressMode::Zero(Some(Register::X))),
            0xad => (OpCode::Load(Register::A), AddressMode::Absolute(None)),
            0xbd => (OpCode::Load(Register::A), AddressMode::Absolute(Some(Register::X))),
            0xb9 => (OpCode::Load(Register::A), AddressMode::Absolute(Some(Register::Y))),
            0xa1 => (OpCode::Load(Register::A), AddressMode::Indirect(Some(Register::X))),
            0xb1 => (OpCode::Load(Register::A), AddressMode::Indirect(Some(Register::Y))),

            // LDX
            0xa2 => (OpCode::Load(Register::X), AddressMode::Immediate),
            0xa6 => (OpCode::Load(Register::X), AddressMode::Zero(None)),
            0xb6 => (OpCode::Load(Register::X), AddressMode::Zero(Some(Register::Y))),
            0xae => (OpCode::Load(Register::X), AddressMode::Absolute(None)),
            0xbe => (OpCode::Load(Register::X), AddressMode::Absolute(Some(Register::Y))),

            // LDY
            0xa0 => (OpCode::Load(Register::Y), AddressMode::Immediate),
            0xa4 => (OpCode::Load(Register::Y), AddressMode::Zero(None)),
            0xb4 => (OpCode::Load(Register::Y), AddressMode::Zero(Some(Register::X))),
            0xac => (OpCode::Load(Register::Y), AddressMode::Absolute(None)),
            0xbc => (OpCode::Load(Register::Y), AddressMode::Absolute(Some(Register::X))),

            // LSR
            0x4a => (OpCode::ShiftRight, AddressMode::Register(Register::A)),
            0x46 => (OpCode::ShiftRight, AddressMode::Zero(None)),
            0x56 => (OpCode::ShiftRight, AddressMode::Zero(Some(Register::X))),
            0x4e => (OpCode::ShiftRight, AddressMode::Absolute(None)),
            0x5e => (OpCode::ShiftRight, AddressMode::Absolute(Some(Register::X))),

            // NOP, BRK
            0xea => (OpCode::NoOp, AddressMode::Implied),
            0x00 => (OpCode::Break, AddressMode::Implied),

            // ORA
            0x09 => (OpCode::Or, AddressMode::Immediate),
            0x05 => (OpCode::Or, AddressMode::Zero(None)),
            0x15 => (OpCode::Or, AddressMode::Zero(Some(Register::X))),
            0x0d => (OpCode::Or, AddressMode::Absolute(None)),
            0x1d => (OpCode::Or, AddressMode::Absolute(Some(Register::X))),
            0x19 => (OpCode::Or, AddressMode::Absolute(Some(Register::Y))),
            0x01 => (OpCode::Or, AddressMode::Indirect(Some(Register::X))),
            0x11 => (OpCode::Or, AddressMode::Indirect(Some(Register::Y))),

            
            // PHA, PHP, PLA, PLP
            0x48 => (OpCode::PushAcc,   AddressMode::Implied),
            0x08 => (OpCode::PushFlags, AddressMode::Implied),
            0x68 => (OpCode::PullAcc,   AddressMode::Implied),
            0x28 => (OpCode::PullFlags, AddressMode::Implied),

            // ROL
            0x2a => (OpCode::RotateLeft, AddressMode::Register(Register::A)),
            0x26 => (OpCode::RotateLeft, AddressMode::Zero(None)),
            0x36 => (OpCode::RotateLeft, AddressMode::Zero(Some(Register::X))),
            0x2e => (OpCode::RotateLeft, AddressMode::Absolute(None)),
            0x3e => (OpCode::RotateLeft, AddressMode::Absolute(Some(Register::X))),

            // ROR
            0x6a => (OpCode::RotateRight, AddressMode::Register(Register::A)),
            0x66 => (OpCode::RotateRight, AddressMode::Zero(None)),
            0x76 => (OpCode::RotateRight, AddressMode::Zero(Some(Register::X))),
            0x6e => (OpCode::RotateRight, AddressMode::Absolute(None)),
            0x7e => (OpCode::RotateRight, AddressMode::Absolute(Some(Register::X))),

            // RTI, RTS
            0x40 => (OpCode::ReturnInt, AddressMode::Implied),
            0x60 => (OpCode::ReturnSub, AddressMode::Implied),

            // SEC, SED, SEI, Set processor flags
            0x38 => (OpCode::SetFlag(Flag::Carry, true), AddressMode::Implied),
            0xf8 => (OpCode::SetFlag(Flag::Decimal, true), AddressMode::Implied),
            0x78 => (OpCode::SetFlag(Flag::Interrupt, true), AddressMode::Implied),

            // CLC, CLD, CLI, CLO, Clear processor flags
            0x18 => (OpCode::SetFlag(Flag::Carry, false), AddressMode::Implied),
            0xd8 => (OpCode::SetFlag(Flag::Decimal, false), AddressMode::Implied),
            0x58 => (OpCode::SetFlag(Flag::Interrupt, false), AddressMode::Implied),
            0xB8 => (OpCode::SetFlag(Flag::Overflow, false), AddressMode::Implied),


            // STA
            0x85 => (OpCode::Store(Register::A), AddressMode::Zero(None)),
            0x95 => (OpCode::Store(Register::A), AddressMode::Zero(Some(Register::X))),
            0x8d => (OpCode::Store(Register::A), AddressMode::Absolute(None)),
            0x9d => (OpCode::Store(Register::A), AddressMode::Absolute(Some(Register::X))),
            0x99 => (OpCode::Store(Register::A), AddressMode::Absolute(Some(Register::Y))),
            0x81 => (OpCode::Store(Register::A), AddressMode::Indirect(Some(Register::X))),
            0x91 => (OpCode::Store(Register::A), AddressMode::Indirect(Some(Register::Y))),

            // STX
            // 0x => (OpCode::Store(Register::X), AddressMode::Immediate),
            0x86 => (OpCode::Store(Register::X), AddressMode::Zero(None)),
            0x96 => (OpCode::Store(Register::X), AddressMode::Zero(Some(Register::Y))),
            0x8e => (OpCode::Store(Register::X), AddressMode::Absolute(None)),
            // 0x9e => (OpCode::Store(Register::X), AddressMode::Absolute(Some(Register::Y))),

            // STY
            // 0x => (OpCode::Store(Register::Y), AddressMode::Immediate),
            0x84 => (OpCode::Store(Register::Y), AddressMode::Zero(None)),
            0x94 => (OpCode::Store(Register::Y), AddressMode::Zero(Some(Register::X))),
            0x8c => (OpCode::Store(Register::Y), AddressMode::Absolute(None)),
            // 0x9c => (OpCode::Store(Register::Y), AddressMode::Absolute(Some(Register::X))),

            // BIT
            0x24 => (OpCode::Bit, AddressMode::Zero(None)),
            0x2c => (OpCode::Bit, AddressMode::Absolute(None)),

            // AND
            0x29 => (OpCode::And, AddressMode::Immediate),
            0x25 => (OpCode::And, AddressMode::Zero(None)),
            0x35 => (OpCode::And, AddressMode::Zero(Some(Register::X))),
            0x2d => (OpCode::And, AddressMode::Absolute(None)),
            0x3d => (OpCode::And, AddressMode::Absolute(Some(Register::X))),
            0x39 => (OpCode::And, AddressMode::Absolute(Some(Register::Y))),
            0x21 => (OpCode::And, AddressMode::Indirect(Some(Register::X))),
            0x31 => (OpCode::And, AddressMode::Indirect(Some(Register::Y))),

            0xc9 => (OpCode::Compare(Register::A), AddressMode::Immediate),
            // 0x25 => (OpCode::And, AddressMode::Zero(None)),
            // 0x35 => (OpCode::And, AddressMode::Zero(Some(Register::X))),
            // 0x2d => (OpCode::And, AddressMode::Absolute(None)),
            // 0x3d => (OpCode::And, AddressMode::Absolute(Some(Register::X))),
            // 0x39 => (OpCode::And, AddressMode::Absolute(Some(Register::Y))),
            // 0x21 => (OpCode::And, AddressMode::Indirect(Some(Register::X))),
            // 0x31 => (OpCode::And, AddressMode::Indirect(Some(Register::Y))),
            
            instr => panic!("instruction not implemented: 0x{instr:02x}")
        };

        (op, am, byte_code)
    }

    pub(crate) fn execute(sys: &mut System, op: OpCode, am: AddressMode) -> u8 {
        // let cpu = &mut sys.cpu;
        let mut pc_set = false;

        let cycles = match (op, am) {
            (OpCode::Jump, AddressMode::Absolute(None)) => {
                let addr = CPU::addr_absolute(sys, &None);
                sys.cpu.pc = addr; 
                pc_set = true;
                3
            }

            (OpCode::Jump, AddressMode::Indirect(r)) => {
                let addr = CPU::addr_indirect(sys, &r);
                sys.cpu.pc = addr; 
                pc_set = true;
                5
            }

            (OpCode::JumpSub, AddressMode::Absolute(None)) => {
                let addr = CPU::addr_absolute(sys, &None);
                let pc = sys.cpu.pc;
                CPU::stack_push_word(sys, sys.cpu.pc);
                sys.cpu.pc = addr; 
                pc_set = true;
                3
            }

            (OpCode::BranchIf(flag, value), AddressMode::Relative) => {
                let addr = CPU::addr_relative(sys);
                if sys.cpu.get_flag(flag) == value {
                    sys.cpu.pc = sys.cpu.pc.checked_add_signed(addr as i16).expect("program counter overflow!");
                    // pc_set = true;
                    3
                } else {2}
            }

            (OpCode::Load(Register::A), AddressMode::Indirect(source_reg)) => {
                let addr = CPU::addr_indirect(sys, &source_reg);
                let value = sys.read_byte(addr);
                sys.cpu.pc += 2;
                sys.cpu.set_reg(Register::A, value);
                6 // TODO: should be 5 for Y if no page is crossed
            }

            (OpCode::Load(target_reg), AddressMode::Immediate) => {
                let addr = CPU::addr_immediate(sys);
                sys.cpu.set_reg(target_reg, sys.read_byte(addr));
                6 // TODO: should be 5 for Y if no page is crossed
            }

            (OpCode::Store(reg), AddressMode::Zero(addr_reg)) => {
                let addr = CPU::addr_zero(sys, &addr_reg);
                let value = sys.cpu.get_reg(reg);
                sys.write_byte(addr, value);
                match addr_reg { None => 3, _ => 4 }
            }

            (OpCode::NoOp, _) => 2,

            (OpCode::SetFlag(flag, value), _) => {
                match flag {
                    Flag::Carry => sys.cpu.carry = value,
                    Flag::Decimal => sys.cpu.decimal = value,
                    Flag::Interrupt => sys.cpu.interrupt = value,
                    Flag::Overflow => sys.cpu.overflow = value,
                    flag => panic!("setting flag {flag:?} is not implemented")
                };
                2
            }

            (OpCode::Bit, address_mode) => {
                let addr = CPU::get_addr(sys, &address_mode);
                let value = sys.read_byte(addr);
                sys.cpu.zero = (sys.cpu.a & value) != 0;
                sys.cpu.overflow = (value & 0b00100000) != 0;
                sys.cpu.sign = (value & 0b01000000) != 0;
                match address_mode { AddressMode::Zero(_) => 3, _ => 4 }
            }

            (OpCode::ReturnSub, _) => {
                let pc = CPU::stack_pull_word(sys);
                sys.cpu.pc = pc;
                6
            }

            (OpCode::PushFlags, _) => {
                let flags = 
                    if sys.cpu.carry      {1 << 7} else {0} |
                    if sys.cpu.zero       {1 << 6} else {0} |
                    if sys.cpu.interrupt  {1 << 5} else {0} |
                    if sys.cpu.decimal    {1 << 4} else {0} |
                    if sys.cpu.soft_break {1 << 3} else {0} |
                    if sys.cpu.overflow   {1 << 2} else {0} |
                    if sys.cpu.sign       {1 << 1} else {0} ;
                // println!("Pushed flags: {flags:08b}");
                CPU::stack_push_byte(sys, flags);
                3
            }

            
            (OpCode::PullFlags, _) => {
                let flags = CPU::stack_pull_byte(sys);
                sys.cpu.carry      = ((flags >> 7) & 1) != 0;
                sys.cpu.zero       = ((flags >> 6) & 1) != 0;
                sys.cpu.interrupt  = ((flags >> 5) & 1) != 0;
                sys.cpu.decimal    = ((flags >> 4) & 1) != 0;
                sys.cpu.soft_break = ((flags >> 3) & 1) != 0;
                sys.cpu.overflow   = ((flags >> 2) & 1) != 0;
                sys.cpu.sign       = ((flags >> 1) & 1) != 0;
                3
            }


            (OpCode::PullAcc, _) => {
                let value = CPU::stack_pull_byte(sys);
                sys.cpu.set_reg(Register::A, value);
                4
            }

            (OpCode::PushAcc, _) => {
                CPU::stack_push_byte(sys, sys.cpu.a);
                4
            }
            
            (OpCode::And, address_mode) => {
                let addr = CPU::get_addr(sys, &address_mode);
                sys.cpu.a &=  sys.read_byte(addr);
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

                        
            (OpCode::Or, address_mode) => {
                let addr = CPU::get_addr(sys, &address_mode);
                sys.cpu.a |=  sys.read_byte(addr);
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

            (OpCode::Compare(reg), address_mode) => {
                let addr = CPU::get_addr(sys, &address_mode);
                let value_m = sys.read_byte(addr);
                let value_r = sys.cpu.get_reg(reg);
                sys.cpu.carry = value_r >= value_m;
                sys.cpu.zero = value_r == value_m;
                4 // TODO: Should take 2 - 6 cycles depending on addressing mode!
            }

            (op, am) => panic!("opcode {op:?} is not implemented with address mode {am:?}")
        };

        if !pc_set {
            sys.cpu.pc += 1;
        }

        cycles
        
    }

    fn set_reg(&mut self, register: Register, value: u8) {
        match register {
            Register::X => self.x = value,
            Register::Y => self.y = value,
            Register::A => self.a = value,
        }
    }

    fn get_reg(&self, register: Register) -> u8 {
        match register {
            Register::X => self.x,
            Register::Y => self.y,
            Register::A => self.a,
        }
    }

    fn stack_push_word(sys: &mut System, value: u16) {
        sys.cpu.sp = sys.cpu.sp.checked_sub(2).expect("stack overflow!");
        let addr = CPU::addr_stack(sys.cpu.sp);
        sys.write_word(addr, value);
        // println!("{value:04x} written to stack at {addr:04x}");
    }

    fn stack_push_byte(sys: &mut System, value: u8) {
        sys.cpu.sp = sys.cpu.sp.checked_sub(1).expect("stack overflow!");
        let addr = CPU::addr_stack(sys.cpu.sp);
        sys.write_byte(addr, value);
    }

    fn stack_pull_word(sys: &mut System) -> u16 {
        let addr = CPU::addr_stack(sys.cpu.sp);
        sys.cpu.sp = sys.cpu.sp.checked_add(2).expect("stack overflow!");
        let value = sys.read_word(addr);
        // println!("{value:04x} read from stack at {addr:04x}");
        value
    }

    
    fn stack_pull_byte(sys: &mut System) -> u8 {
        let addr = CPU::addr_stack(sys.cpu.sp);
        sys.cpu.sp = sys.cpu.sp.checked_add(1).expect("stack overflow!");
        let value = sys.read_byte(addr);
        // println!("{value:04x} read from stack at {addr:04x}");
        value
    }

    const STACK_BOT: u16 = 0x0100;
    const STACK_TOP: u16 = 0x01ff;

    fn addr_stack(sp: u8) -> u16 {
        CPU::STACK_BOT + (sp as u16)
    }

    fn get_flag(&self, flag: Flag) -> bool {
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

    fn get_addr(sys: &mut System, address_mode: &AddressMode) -> u16 {
        match address_mode {
            AddressMode::Zero(reg) => CPU::addr_zero(sys, reg),
            AddressMode::Absolute(reg) => CPU::addr_absolute(sys, reg),
            AddressMode::Immediate => CPU::addr_immediate(sys),
            address_mode => panic!("getting address mode {address_mode:?} is not implemented")
        } 
    }

    // pub(crate) fn execute(&mut self, sys: &mut System, op: OpCode, am: AddressMode) -> u8 {
    //     // let cpu = &mut sys.cpu;
    //     match (op, am) {
    //         (OpCode::Jump, AddressMode::Absolute(None)) => {
    //             self.pc = self.addr_absolute(sys, None); 3
    //         }
    //         (OpCode::Jump, AddressMode::Indirect(r)) => {
    //             self.pc = self.addr_indirect(sys, r); 5
    //         }
    //         (op, am) => panic!("opcode {op:?} is not implemented with address mode {am:?}")
    //     }
    // }

}

#[derive(Debug)]
pub(crate) enum Flag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Overflow,
    Negative,
}

#[derive(Debug)]
pub(crate) enum Register {
    A,
    X,
    Y
}

#[derive(Debug)]
pub(crate) enum OpCode {
    Break,
    Kill,
    NoOp,

    // Logical operators
    // Only possible on accumulator register
    Or,
    And,
    ExOr,
    Add,
    Sub,
    Compare(Register), // CMP, CPX, CPY
    Dec(Option<Register>), // DEC works on addresses and X/Y
    Inc(Option<Register>),
    ShiftLeft,  // TODO: ASL  {adr}:={adr}*2
    RotateLeft, // TODO: ROL  {adr}:={adr}*2+C
    ShiftRight, // TODO: LSR  {adr}:={adr}/2
    RotateRight, // TODO: ROR  {adr}:={adr}/2+C*128


    // Move
    Load(Register),
    Store(Register),
    Transfer(Register, Register),
    PushAcc,   // PHA
    PushFlags, // PHP, Push Processor
    PullAcc,   // PLA
    PullFlags, // PLP, Pull Processor

    Jump,
    JumpSub,
    BranchIf(Flag, bool),

    ReturnInt,
    ReturnSub,

    SetFlag(Flag, bool),
    Bit,
}

#[derive(Debug)]
pub(crate) enum AddressMode {
    Absolute(Option<Register>),
    Zero(Option<Register>),
    Indirect(Option<Register>),
    Relative,
    Immediate,
    Implied,
    Register(Register)
}