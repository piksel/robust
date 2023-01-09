use crate::system::{addr::Addr, cart::Header};

use super::Mapper;


// INES 01
pub struct MMC1 {
    pub(crate) prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    pub(crate) chr_rom: Vec<u8>,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
    control: u8,
    shift: u8,
}
impl MMC1 {
    const PRG_RAM_SIZE: usize = 0x8000;
    pub fn new(_header: &Header, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let prg_ram = vec![0; Self::PRG_RAM_SIZE];


        Self {
            prg_rom, 
            prg_ram,
            chr_rom,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
            control: 0,
            shift: 0,
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0b0011 {
            0 => Mirroring::OneScreenLower,
            1 => Mirroring::OneScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!()
        }
    }

    fn prg_fixed_bank(&self) -> PrgFixedBank {
        match (self.control >> 2) & 0b11 {
            0 | 1 => PrgFixedBank::None,
            2 => PrgFixedBank::First,
            3 => PrgFixedBank::Last,
            _ => unreachable!()
        }
    }

    fn chr_dual_bank(&self) -> bool {
        (self.control & 0b0001_0000) != 0
    }
}

impl Mapper for MMC1 {
    fn ppu_write(&mut self, _addr: Addr, _value: u8) -> anyhow::Result<()> {
        todo!()
    }

    fn cpu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()> {
        // eprintln!("MMC1 write to {addr}, value {value:02x} ({value:08b})");
        if value & 0b1000_0000 != 0 {
            // clear shift reg
            self.shift = 0b0010_0000;
            // set prg mode to 3 (fixed last)
            self.control |= 0b1100;
            eprintln!("Mapper reset!");
            
        } else {
            self.shift = self.shift.rotate_right(1)  |(value << 7);
        }

        // eprintln!("Shift value: {:08b} <= {value:08b} {:08b}", self.shift, value << 7);

        if self.shift & 1 != 0 {
            let reg = 0b1110_0000 & (addr.0 >> 8) as u8;
            let value = (self.shift >> 3) & 0b0001_1111;
            let ri = (reg >> 5) & 0b11;
            // let reg_name = match ri {
            //     0 => "CTRL",
            //     1 => "CHR0",
            //     2 => "CHR1",
            //     3 => "PRG0",
            //     _ => unreachable!()
            // };
            //eprintln!("Writing {value:05b} into register {reg_name} ({reg:02x})");

            match reg {
                0x80 => self.control = value,
                0xa0 => self.chr_bank0 = value,
                0xc0 => self.chr_bank1 = value,
                0xe0 => self.prg_bank = value,
                _ => unreachable!()
            }

            if ri == 0 {
                eprintln!("Control ({:08b}) => DualCHRBank: {:?},  Mirroring: {:?},  FixedPRGBank: {:?}", self.control, self.chr_dual_bank(), self.mirroring(), self.prg_fixed_bank());
            }
            
            self.shift = 0b0010_0000;

        }

        Ok(())
    }

    fn ppu_read(&self, addr: Addr) -> anyhow::Result<u8> {
 
        if self.chr_dual_bank() {
            // eprintln!("DUAL READ!");
            
            let adjusted_addr = match self.mirroring() {
                Mirroring::OneScreenLower => addr.0 % 0x1000,
                Mirroring::OneScreenUpper => todo!(),
                Mirroring::Vertical => todo!(),
                Mirroring::Horizontal => todo!(),
            };
            let mapped_addr = (((self.chr_bank0 & 0b1111) as u16 * 0x1000) + adjusted_addr) as usize;
            return Ok(self.chr_rom[mapped_addr]);
        }

        let bank = if addr < 0x1000 {
            self.chr_bank0
        } else {
            self.chr_bank1
        } as u16;

        // eprintln!("");

        let bank_offset = 0x1000 * (bank & 0b111);
        let mapped_addr = ((addr.0) + bank_offset) as usize;
        let adjusted_addr = match self.mirroring() {
            Mirroring::OneScreenLower => mapped_addr % 0x1000,
            Mirroring::OneScreenUpper => todo!(),
            Mirroring::Vertical => todo!(),
            Mirroring::Horizontal => todo!(),
        };

        // eprint!("{adjusted_addr:04x} ");

        Ok(self.chr_rom[adjusted_addr % self.chr_rom.len()])
        
        // Ok(self.chr_rom[addr.0 as usize])
    }

    fn cpu_read(&self, addr: Addr) -> anyhow::Result<u8> {
        // eprintln!("Reading from {addr} using {:?}, bank {}", self.prg_fixed_bank(), self.prg_bank);
        if addr < 0x6000 {
            anyhow::bail!("read outside pgm range: {addr}")
        } else if addr < 0x8000 {
            // PRG RAM
            anyhow::bail!("read outside pgm range: {addr}")
            // self.prg_ram[addr - 0x6000]
        } else if addr < 0xc000 {

            let bank = match self.prg_fixed_bank() {
                PrgFixedBank::First => 0,
                PrgFixedBank::Last | PrgFixedBank::None => self.prg_bank,
            };

            let bank_offset = bank as usize * 0x4000;
            let rom_addr = bank_offset + (addr.0 as usize - 0x8000);
            // println!("Reading from {addr:08x} => bank offset {last_bank:04x} => {rom_addr:04x} => {rom_addr2:04x}");
            Ok(self.prg_rom[rom_addr % self.prg_rom.len()])
        } else {
            let bank_offset = match self.prg_fixed_bank() {
                PrgFixedBank::Last => self.prg_rom.len() - 0x4000,
                PrgFixedBank::First => self.prg_bank as usize * 0x4000,
                PrgFixedBank::None => (self.prg_bank + 1) as usize * 0x4000,
            };
            let rom_addr = bank_offset + (addr.0 as usize - 0xc000);
            // let rom_addr2 = rom_addr % self.prg_rom.len();
            // println!("Reading from {addr} => bank ({:?}, {}) offset {bank_offset:04x} => {rom_addr:04x} => {rom_addr2:04x}", self.prg_fixed_bank(), self.prg_bank);
            Ok(self.prg_rom[rom_addr % self.prg_rom.len()])
        }
    }
}

impl std::fmt::Display for MMC1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MMC1")
    }
}

/*(0: one-screen, lower bank; 1: one-screen, upper bank;
|||               2: vertical; 3: horizontal) */
#[derive(Debug)]
enum Mirroring {
    OneScreenLower = 0,
    OneScreenUpper = 1,
    Vertical = 2,
    Horizontal = 3,
}

/* |++--- PRG ROM bank mode (
           0, 1: 
|          2: 
    |      3: 
         */
#[derive(Debug)]
enum PrgFixedBank {
    /// switch 32 KB at $8000, ignoring low bit of bank number;
    None,
    /// fix first bank at $8000 and switch 16 KB bank at $C000;
    First,
    /// fix last bank at $C000 and switch 16 KB bank at $8000)
    Last,
}
