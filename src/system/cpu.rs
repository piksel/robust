use std::{rc::{Weak, Rc}, ops::Add};

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

    // pub(crate) bus: Weak<Bus>,
}

impl CPU {
    pub fn init() -> Self {
        CPU {
            pc: 0,
            sp: 0,
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

    fn addr_immediate(&self) -> u16 {
        self.pc + 1
    }

    fn addr_absolute(&self, sys: &System) -> u16 {
        sys.read_word(self.pc + 1)
    }

    pub(crate) fn load(&self, sys: &System) -> (OpCode, AddressMode) {
        match sys.read_byte(self.pc) {
            0x4c => (OpCode::Jump, AddressMode::Absolute),
            0x6c => (OpCode::Jump, AddressMode::Indirect),
            instr => panic!("instruction not implemented: 0x{instr:02x}")
        }
    }

    pub(crate) fn execute(&self, _sys: &System, op: OpCode, am: AddressMode) {
        match (op, am) {
            (op, am) => panic!("opcode {op:?} is not implemented with address mode {am:?}")
        }
        
    }

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
    Dec(Register), // DEC works on addresses and X/Y
    Inc(Register),
    // TODO: ASL  {adr}:={adr}*2
    // TODO: ROL  {adr}:={adr}*2+C
    // TODO: LSR  {adr}:={adr}/2
    // TODO: ROR  {adr}:={adr}/2+C*128


    // Move
    Load(Register),
    Store(Register),
    Transfer(Register, Register),
    // TODO: PLA, PHA, PLP, PHP

    Jump
}

#[derive(Debug)]
pub(crate) enum AddressMode {
    Absolute,
    Indirect,
    Relative,
    Immediate
}