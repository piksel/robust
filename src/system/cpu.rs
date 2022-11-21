use super::bus::Bus;

pub struct CPU {
    pc: u16,
    sp: u8,

    x: u8,
    y: u8,
    a: u8,

    // status register
    carry: bool,
    zero: bool,
    interrupt: bool,
    decimal: bool,
    soft_break: bool,
    // reserved
    overflow: bool,
    sign: bool,

    bus: Bus,
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
            bus: Bus::init()
        }
    }

    fn addr_immediate(&self) -> u16 {
        self.pc + 1
    }

    fn addr_absolute(&self) -> u16 {
        self.bus.read_word(self.pc + 1)
    }

}
