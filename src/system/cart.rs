

use core::panic;

use anyhow::{Result, format_err, bail};

use crate::mappers::{self, Mapper};

use super::addr::Addr;

#[allow(dead_code)]
const HEADER_SIZE: usize = 16;
const HEADER_MAGIC: [u8; 4] = ['N' as u8, 'E' as u8, 'S' as u8, 0x1a];

type IOResult<T> =std::io::Result<T>;
type ByteResult = IOResult<u8>;

pub(crate) struct Cart {
    pub(crate) header: Header,
    pub mapper: Box<dyn Mapper>,
}

impl Cart {


    pub(crate) fn new<B: IntoIterator<Item=ByteResult>>(bytes: B) -> Result<Self> {
        let mut head = bytes.into_iter();
        let header = Header::from_bytes(head.by_ref().take(16))?;

        let prg_rom = head.by_ref().take(header.prg_rom_size).collect::<IOResult<Vec<u8>>>()?;
        let chr_rom = head.by_ref().take(header.chr_rom_size).collect::<IOResult<Vec<u8>>>()?;

        let mapper = mappers::new(&header, prg_rom, chr_rom)?;



        Ok(Cart{
            header,
            mapper,
        })
    }

    pub(crate) fn read_prg_byte(&self, addr: usize) -> u8 {
        self.mapper.cpu_read(Addr(addr as u16)).unwrap()
        
    }

    pub fn read_chr_byte(&self, addr: u16) -> u8 {
        self.mapper.ppu_read(addr.into()).unwrap()
    }

    pub(crate) fn write_byte(&mut self, addr: Addr, value: u8) {
        self.mapper.cpu_write(addr, value).unwrap();
    }

    pub(crate) fn get_tile(&self, addr: u16) -> anyhow::Result<(u8, u8)> {
        let upper_addr = Addr(addr);
        let lower_addr = upper_addr + 8;
        let upper_sliver = self.mapper.ppu_read(upper_addr)?;// .chr_rom[upper_addr as usize];
        let lower_sliver = self.mapper.ppu_read(lower_addr)?;// as usize];
        Ok((upper_sliver, lower_sliver))
    }
}

#[allow(dead_code)]
pub struct Header {
    header_type: HeaderType,
    pub(crate) vertical_mirroring: bool, 
    pub(crate) battery_ram: bool, 
    pub(crate) trainer: bool, 
    pub(crate) no_mirror: bool,
    pub(crate) prg_rom_size: usize,
    pub(crate) chr_rom_size: usize,
    pub(crate) mapper_id: u16
}

enum HeaderType {
    INES(INESHeader), 
    #[allow(dead_code)]
    NES2(NES2Header)
}

struct INESHeader {

}

// bitflags! {
//     pub MapperFlags [vertical_mirroring, battery_ram, trainer, no_mirror]
// }

struct NES2Header {

}

impl Header {
    pub fn from_bytes<B>(mut bytes: B) -> Result<Self> where
        B: Iterator<Item = ByteResult>
    {

        let too_short_err = || format_err!("not enough bytes for a cart header");

        // let magic_bytes = bytes.take(HEADER_MAGIC.len());
        for expected in HEADER_MAGIC.iter() {
            let actual = bytes.next().ok_or_else(too_short_err)??;
            
            // let actual = actual?;
            if actual != *expected {
                bail!("invalid cart header magic byte: {actual:02x} (expected {expected:02x})");
            }
        }


        let prg_rom_size_raw = bytes.next().ok_or_else(too_short_err)??;
        let chr_rom_size_raw = bytes.next().ok_or_else(too_short_err)??;

        eprintln!("Program ROM size: {prg_rom_size_raw}");
        eprintln!("Character ROM size: {chr_rom_size_raw}");

        let flags6 = bytes.next().ok_or_else(too_short_err)??;
        let trainer = false;
        let vertical_mirroring = false;
        let no_mirror = false;
        let battery_ram = false;

        eprintln!("Flags6: {flags6:08b}");

        let flags7 = bytes.next().ok_or_else(too_short_err)??;

        let nes2_format = flags7 & 0b00001100 == 0b00001100;

        if nes2_format {
            eprintln!("NES2.0 format, (flags7: {flags7:08b})");

            panic!("not implemented")

        } else {
            eprintln!("iNES format, (flags7: {flags7:08b})");

            let prg_rom_size = (prg_rom_size_raw as usize) * 16384;
            let chr_rom_size = (chr_rom_size_raw as usize) * 8192;

            eprintln!("PRG_ROM: {prg_rom_size} byte(s) ({prg_rom_size:08x})");
            eprintln!("CHR_ROM: {chr_rom_size} byte(s) ({chr_rom_size:08x})");


            #[allow(unused_variables)]
            {
                let flags8 = bytes.next().ok_or_else(too_short_err)??;
                let flags9 = bytes.next().ok_or_else(too_short_err)??;
                let flags10 = bytes.next().ok_or_else(too_short_err)??;
            }

            let mapper_id = (flags6 as u16 & 0xf0) >> 4 | (flags7 as u16 & 0xf0);

            eprintln!("Mapper: {mapper_id} ({mapper_id:016b}), f6lo: {flags6:08b}, f7hi: {flags7:08b}");
    
            // Unused
            let _ = bytes.by_ref().take(5).collect::<IOResult<Vec<u8>>>()?;

            if trainer {
                //for _ in 0..512 {
                    // skip trainer
                 //   let _ = bytes.by_ref().take(512);
                //}
            }

            let header_type = HeaderType::INES(INESHeader{});

            Ok(Self{
                mapper_id,
                header_type,
                prg_rom_size,
                chr_rom_size,
                trainer,
                no_mirror,
                vertical_mirroring,
                battery_ram
            })

            
        }

        // MapperFlags(flags6).BatteryRam()

    }
}




