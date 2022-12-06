
/*
$0000-$07FF	$0800	2KB internal RAM
$0800-$0FFF	$0800	Mirrors of $0000-$07FF
$1000-$17FF	$0800
$1800-$1FFF	$0800
$2000-$2007	$0008	NES PPU registers
$2008-$3FFF	$1FF8	Mirrors of $2000-2007 (repeats every 8 bytes)
$4000-$4017	$0018	NES APU and I/O registers
$4018-$401F	$0008	APU and I/O functionality that is normally disabled. See CPU Test Mode.
$4020-$FFFF	$BFE0	Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note
*/

use super::ppu;

impl super::System {

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match self.map_addr(addr) {
            BusTarget::RAM(ra) => self.ram[ra],
            BusTarget::PPU(ra) => ppu::read(self, ra as u8),
            BusTarget::APU(ra) => self.apu[ra],
            BusTarget::PRG => {
                match &self.cart {
                    None => panic!("tried to read from cart when not loaded"),
                    Some(cart) => cart.read_prg_byte(addr as usize)
                }
            },
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match self.map_addr(addr) {
            BusTarget::RAM(ra) => self.ram[ra] = value,
            BusTarget::PPU(ra) => ppu::write(self, ra as u8, value),
            BusTarget::APU(ra) => self.apu[ra] = value,
            BusTarget::PRG => {
                match &self.cart {
                    None => panic!("tried to read from cart when not loaded"),
                    Some(_cart) => eprintln!("tried to write to 0x{value:02x} to cart ({addr:04x}) which is not implemented")
                    // Some(_cart) => panic!("writing to cart is not implemented (tried writing {value:02x} to {addr:04x})")
                }
            },
        }
    }



    pub fn read_word(&mut self, addr: u16) -> u16 {
        let high = self.read_byte(addr + 1);
        let low = self.read_byte(addr);
        ((high as u16) << 8) | low as u16
    }

    pub fn read_zero_word(&mut self, addr: u8) -> u16 {
        let high = self.read_byte(addr.wrapping_add(1) as u16);
        let low = self.read_byte(addr as u16);
        ((high as u16) << 8) | low as u16
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, (value & 0xff) as u8);
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    pub fn map_addr(&self, addr: u16) -> BusTarget {
        if addr <= 0x07ff {
            // $0000-$07FF	$0800	2KB internal RAM
            BusTarget::RAM(addr as usize)
        } else if addr <= 0x0fff {
            // $0800-$0FFF	$0800	Mirrors of $0000-$07FF
            BusTarget::RAM((addr - 0x800) as usize)
        } else if addr <= 0x1fff {
            // $1000-$17FF	$0800
            // $1800-$1FFF	$0800
            panic!("invalid address")
        } else if addr <= 0x2007 {
            // $2000-$2007	$0008	NES PPU registers
            BusTarget::PPU( (addr - 0x2000) as usize )
        } else if addr <= 0x3fff {
            // $2008-$3FFF	$1FF8	Mirrors of $2000-2007 (repeats every 8 bytes)
            BusTarget::PPU( ((addr - 0x2000) % 8) as usize )
        } else if addr <= 0x4017 {
            // $4000-$4017	$0018	NES APU and I/O registers
            BusTarget::APU((addr - 0x4000) as usize )
        } else if addr <= 0x401f {
            // $4018-$401F	$0008	APU and I/O functionality that is normally disabled. See CPU Test Mode.
            panic!("invalid address")
        } else {
            // $4020-$FFFF	$BFE0	Cartridge space: PRG ROM, PRG RAM, and mapper registers
            BusTarget::PRG
        }
    }
}

pub enum BusTarget {
    RAM(usize),
    PPU(usize),
    APU(usize),
    PRG
}
