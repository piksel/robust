pub struct Bus {
    ram: Vec<u8>,
    ppu: Vec<u8>,
    apu: Vec<u8>,
    prg: Vec<u8>,
}

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

impl Bus {
    pub fn init() -> Self {
        Bus { 
            ram: vec!(),
            ppu: vec!(),
            apu: vec!(),
            prg: vec!(),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match self.map_addr(addr) {
            BusTarget::RAM(ra) => self.ram[ra],
            BusTarget::PPU(ra) => self.ppu[ra],
            BusTarget::APU(ra) => self.apu[ra],
            BusTarget::PRG(ra) => self.prg[ra],
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        let high = self.read_byte(addr + 1);
        let low = self.read_byte(addr);
        ((high as u16) << 8) | low as u16
        // match self.map_addr(addr) {
        //     RAM(ra) => self.ram[ra] << 8 | self.ram[ra + 1],
        //     PPU(ra) => self.ppu[ra] << 8 | self.ppu[ra + 1],
        //     APU(ra) => self.apu[ra] << 8 | self.apu[ra + 1],
        //     PRG(ra) => self.prg[ra] << 8 | self.prg[ra + 1],
        // }
    }

    pub fn map_addr(&self, addr: u16) -> BusTarget {
        if addr < 0x07ff {
            // $0000-$07FF	$0800	2KB internal RAM
            BusTarget::RAM(addr as usize)
        } else if addr < 0x0fff {
            // $0800-$0FFF	$0800	Mirrors of $0000-$07FF
            BusTarget::RAM((addr - 0x800) as usize)
        } else if addr < 0x1fff {
            // $1000-$17FF	$0800
            // $1800-$1FFF	$0800
            panic!("invalid address")
        } else if addr < 0x2007 {
            // $2000-$2007	$0008	NES PPU registers
            BusTarget::PPU( (addr - 0x2000) as usize )
        } else if addr < 0x3fff {
            // $2008-$3FFF	$1FF8	Mirrors of $2000-2007 (repeats every 8 bytes)
            BusTarget::PPU( ((addr - 0x2000) % 8) as usize )
        } else if addr < 0x4017 {
            // $4000-$4017	$0018	NES APU and I/O registers
            BusTarget::APU((addr - 0x4000) as usize )
        } else if addr < 0x1fff {
            // $4018-$401F	$0008	APU and I/O functionality that is normally disabled. See CPU Test Mode.
            panic!("invalid address")
        } else {
            // $4020-$FFFF	$BFE0	Cartridge space: PRG ROM, PRG RAM, and mapper registers
            BusTarget::PRG((addr - 0x4020) as usize)
        }
    }
}

pub enum BusTarget {
    RAM(usize),
    PPU(usize),
    APU(usize),
    PRG(usize)
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

}
