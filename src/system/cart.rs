

use anyhow::{Result, format_err, bail};

#[allow(dead_code)]
const HEADER_SIZE: usize = 16;
const HEADER_MAGIC: [u8; 4] = ['N' as u8, 'E' as u8, 'S' as u8, 0x1a];

type IOResult<T> =std::io::Result<T>;
type ByteResult = IOResult<u8>;

pub(crate) struct Cart {
    header: Header,
    pub(crate) prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    #[allow(dead_code)]
    chr_rom: Vec<u8>,
}

impl Cart {


    pub(crate) fn new<B: IntoIterator<Item=ByteResult>>(bytes: B) -> Result<Self> {
        let mut head = bytes.into_iter();
        let header = Header::from_bytes(head.by_ref().take(16))?;

        let prg_rom = head.by_ref().take(header.prg_rom_size).collect::<IOResult<Vec<u8>>>()?;
        let chr_rom = head.by_ref().take(header.chr_rom_size).collect::<IOResult<Vec<u8>>>()?;

        let prg_ram_size = match header.mapper {
            // 0 => 4096,
            0 => 0x2000,
            mapper => panic!("mapper {mapper} is not implemented")
        };
        let prg_ram = vec![0; prg_ram_size];

        Ok(Cart{
            header,
            prg_rom,
            chr_rom,
            prg_ram,
        })
    }

    pub(crate) fn read_prg_byte(&self, addr: usize) -> u8 {
        match self.header.mapper {
            0 => {
                if addr < 0x6000 {
                    panic!("read outside pgm range: {addr}")
                } else if addr < 0x8000 {
                    self.prg_ram[addr - 0x6000]
                } else {
                    self.prg_rom[(addr - 0x8000) % self.header.prg_rom_size]
                }
            }
            mapper => panic!("reading is not implemented for mapper {mapper}")
        }
    }
}

#[allow(dead_code)]
struct Header {
    header_type: HeaderType,
    vertical_mirroring: bool, 
    battery_ram: bool, 
    trainer: bool, 
    no_mirror: bool,
    prg_rom_size: usize,
    chr_rom_size: usize,
    mapper: u16
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
                bail!("invalid cart header magic byte: {actual:?} (expected {expected:02x})");
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

            #[allow(unused_variables)]
            {
                let flags8 = bytes.next().ok_or_else(too_short_err)??;
                let flags9 = bytes.next().ok_or_else(too_short_err)??;
                let flags10 = bytes.next().ok_or_else(too_short_err)??;
            }

            let mapper = (flags7 as u16 & 0xf0) >> 8 | (flags6 as u16 & 0xf0);

            eprintln!("Mapper: {mapper} ({mapper:016b}), f6lo: {flags6:08b}, f7hi: {flags7:08b}");
    
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
                mapper,
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

