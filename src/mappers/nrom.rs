use std::fmt::Display;

use crate::system::{addr::Addr, cart::Header};

use super::Mapper;

// INES 00
pub struct NROM {
    pub(crate) prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    #[allow(dead_code)]
    pub(crate) chr_rom: Vec<u8>,
    pub(crate) chr_ram: Vec<u8>,
    chr_bank: u8,
}
impl NROM {
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

impl Mapper for NROM {
    fn ppu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()> {
        todo!()
    }

    fn cpu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()> {
        anyhow::bail!("tried to write to 0x{value:02x} to cart ({addr}) which is not implemented for mapper {self}");
    }

    fn ppu_read(&self, addr: Addr) -> anyhow::Result<u8> {
        if self.chr_rom.len() == 0 {
            Ok(self.chr_ram[addr.0 as usize])
        } else {
            Ok(self.chr_rom[addr.0 as usize])
        }
    }

    fn cpu_read(&self, addr: Addr) -> anyhow::Result<u8> {
        if addr < 0x6000 {
            panic!("read outside pgm range: {addr}")
        } else if addr < 0x8000 {
            Ok(self.prg_ram[addr.0 as usize - 0x6000])
        } else {
            Ok(self.prg_rom[(addr.0 as usize - 0x8000) % self.prg_rom.len()])
        }
    }
}

impl Display for NROM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NROM")
    }
}