use std::fmt::Display;

use crate::system::{cart::Header, addr::Addr};

use super::Mapper;



// INES 02
pub struct UxROM {
    pub(crate) prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    #[allow(dead_code)]
    pub(crate) chr_rom: Vec<u8>,
    pub(crate) chr_ram: Vec<u8>,
    chr_bank: u8,
}
impl UxROM {
    pub fn new(header: &Header, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {


        let chr_ram = vec![0u8; 8192];

        let prg_ram_size = match header.mapper_id {
            // 0 => 4096,
            0 => 0x2000,
            1 => 0,
            2 => 0,
            mapper => panic!("mapper {mapper} is not implemented")
        };
        let prg_ram = vec![0; prg_ram_size];


        Self {
            prg_rom, 
            prg_ram,
            chr_rom,
            chr_ram,
            chr_bank: 0,
        }
    }
}

impl Mapper for UxROM {
    fn ppu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()> {
        self.chr_ram[addr.0 as usize] = value;
        Ok(())
    }

    fn cpu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()> {
        if addr > 0x8000 {
            let banks = (self.prg_rom.len() / 0x4000) as u8;
            // assert!(value < banks, "Value {value} is larger than banks ({banks})");
            if value < banks {
                // eprintln!("CHR switched to bank {value:02x} ({value:08b})");
                self.chr_bank = value;
            } else {eprintln!("Value {value} is larger than banks ({banks})");}
            
        }
        Ok(())
    }

    fn ppu_read(&self, addr: Addr) -> anyhow::Result<u8> {
        Ok(self.chr_ram[addr.0 as usize])
    }

    fn cpu_read(&self, addr: Addr) -> anyhow::Result<u8> {
        if addr < 0x6000 {
            anyhow::bail!("read outside pgm range: {addr}")
        } else if addr < 0x8000 {
            anyhow::bail!("read outside pgm range: {addr}")
            // self.prg_ram[addr - 0x6000]
        } else if addr < 0xc000 {
            // switchable bank
            let bank_offset = self.chr_bank as usize * 0x4000;
            let rom_addr = bank_offset + (addr.0 as usize - 0x8000);
            // let rom_addr2 = rom_addr % self.prg_rom.len();
            // println!("Reading from {addr:08x} => bank offset {last_bank:04x} => {rom_addr:04x} => {rom_addr2:04x}");
            Ok(self.prg_rom[rom_addr % self.prg_rom.len()])
        } else {
            // static last bank
            
            let last_bank = self.prg_rom.len() - 0x4000;
            let rom_addr = last_bank + (addr.0 as usize - 0xc000);
            // let rom_addr2 = rom_addr % self.prg_rom.len();
            // println!("Reading from {addr:08x} => bank offset {last_bank:04x} => {rom_addr:04x} => {rom_addr2:04x}");
            Ok(self.prg_rom[rom_addr % self.prg_rom.len()])
        }
    }
}

impl Display for UxROM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UxROM")
    }
}