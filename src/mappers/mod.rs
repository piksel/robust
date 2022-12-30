use std::{fmt::Display};

use crate::system::{addr::Addr, cart::Header};


pub mod mmc1;
pub mod nrom;
pub mod uxrom;

pub use mmc1::MMC1;
pub use nrom::NROM;
pub use uxrom::UxROM;

pub trait Mapper where Self: Display {
    fn ppu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()>;
    fn cpu_write(&mut self, addr: Addr, value: u8) -> anyhow::Result<()>;

    fn ppu_read(&self, addr: Addr) -> anyhow::Result<u8>;
    fn cpu_read(&self, addr: Addr) -> anyhow::Result<u8>;
}

pub fn new(header: &Header, prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> anyhow::Result<Box<dyn Mapper>> {
     match header.mapper_id {
        0 => Ok(Box::new(NROM::new(&header, prg_rom, chr_rom))),
        1 => Ok(Box::new(MMC1::new(&header, prg_rom, chr_rom))),
        2 => Ok(Box::new(UxROM::new(&header, prg_rom, chr_rom))),
        mid => anyhow::bail!("non-supported mapper id {mid:02x}"),
    }
}