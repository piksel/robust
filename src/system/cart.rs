

use core::panic;

use anyhow::{Result, format_err, bail};
use std::{io::{Write, Cursor, Read}};
use crate::mappers::{self, Mapper};
use termcolor::{WriteColor, ColorSpec, Color, BufferWriter, Buffer};
use super::addr::Addr;

#[allow(dead_code)]
const HEADER_SIZE: usize = 16;
const HEADER_MAGIC: [u8; 4] = ['N' as u8, 'E' as u8, 'S' as u8, 0x1a];

type IOResult<T> =std::io::Result<T>;
type ByteResult = IOResult<u8>;

pub(crate) struct Cart {
    pub(crate) header: Header,
    pub mapper: Box<dyn Mapper>,
    is_empty: bool,
}

impl Cart {

    const EMPTY_CART: &[u8] = include_bytes!("../../empty/empty.nes");

    pub(crate) fn empty() -> Result<Self> {
        let mut dummy = Buffer::ansi();
        let cart = Self::init(Cursor::new(Self::EMPTY_CART).bytes(), true, &mut dummy)?;
        Ok(cart)
    }

    pub(crate) fn new<B: IntoIterator<Item=ByteResult>>(bytes: B) -> Result<Self> {
        
        let stderr = BufferWriter::stderr(termcolor::ColorChoice::Auto);
        let mut buffer = stderr.buffer();
        let cart = Self::init(bytes, false, &mut buffer)?;
        buffer.flush()?;
        stderr.print(&buffer)?;
        Ok(cart)
    }

    pub(crate) fn init<B: IntoIterator<Item=ByteResult>>(bytes: B, is_empty: bool, log: &mut Buffer) -> Result<Self> {
        let mut head = bytes.into_iter();
        let header = Header::from_bytes(head.by_ref().take(16), log)?;

        let prg_rom = head.by_ref().take(header.prg_rom_size).collect::<IOResult<Vec<u8>>>()?;
        let chr_rom = if header.chr_rom_size == 0 {
            vec![0; 0x2000]
        } else {
            head.by_ref().take(header.chr_rom_size).collect::<IOResult<Vec<u8>>>()?
        };

        let mapper = mappers::new(&header, prg_rom, chr_rom)?;

        log_var(log, "Mapper", &mapper, ColorSpec::new().set_bold(true).set_fg(Some(Color::Magenta)));


        Ok(Cart{
            header,
            mapper,
            is_empty,
        })
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

    pub(crate) fn is_empty(&self) -> bool {
        self.is_empty
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

fn log_var(stream:&mut Buffer, key: &str, val: impl std::fmt::Display, spec: &ColorSpec){
    let reset = ColorSpec::new();
    let _ = stream.set_color(&reset);
    let _ = write!(stream, "{key}: ");
    let _ = stream.set_color(spec);
    let _ = writeln!(stream, "{val}");
    let _ = stream.set_color(&reset);
}

fn log_bool(stream: &mut Buffer, key: &str, val: bool, spec_true: &ColorSpec, spec_false: &ColorSpec){
    if val {
        log_var(stream, key, "yes", spec_true);
    } else {
        log_var(stream, key, "no", spec_false);
    }
}

fn log_flags(stream:&mut Buffer, key: &str, val: u8, shorts: &str, spec_true: &ColorSpec, spec_false: &ColorSpec){
    let reset = ColorSpec::new();
    let _ = stream.set_color(&reset);
    let _ = write!(stream, "{key}: ");
    for (i, c) in shorts.chars().enumerate() {
        let _ = stream.set_color(if val >> i & 1 != 0 {spec_true}else{spec_false});
        let _ = write!(stream, "{c}");
    }
    let _ = writeln!(stream);
    let _ = stream.set_color(&reset);
}

impl Header {
    pub fn from_bytes<B>(mut bytes: B, log: &mut Buffer) -> Result<Self> where
        B: Iterator<Item = ByteResult>
    {
        let col_enum = ColorSpec::new().set_bold(true).set_fg(Some(Color::Magenta)).to_owned();
        let col_true = ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)).to_owned();
        let col_false = ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)).to_owned();
        let col_number = ColorSpec::new().set_bold(true).set_fg(Some(Color::Blue)).to_owned();
        

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

        log_var(log, "PRG ROM size", format!("{prg_rom_size_raw}"), &col_number);
        log_var(log, "CHR ROM size", format!("{chr_rom_size_raw}"), &col_number);

        let flags6 = bytes.next().ok_or_else(too_short_err)??;

        log_flags(log, "Byte 6 Flags",  flags6, "VBTDLLLL", &col_true, &col_false);

        let vertical_mirroring = flags6 & 0b0001 != 0;
        let battery_ram        = flags6 & 0b0010 != 0;
        let trainer            = flags6 & 0b0100 != 0;
        let no_mirror          = flags6 & 0b1000 != 0;

        log_bool(log, " => Vertical mirroring", vertical_mirroring, &col_true, &col_false);
        log_bool(log, " => Battery RAM", battery_ram, &col_true, &col_false);
        log_bool(log, " => Trainer", trainer, &col_true, &col_false);
        log_bool(log, " => Disable mirroring", no_mirror, &col_true, &col_false);

        let flags7 = bytes.next().ok_or_else(too_short_err)??;

        log_flags(log, "Byte 7 Flags",  flags7, "VPNNUUUU", &col_true, &col_false);

        let vs_unisystem = flags7 & 0b0001 != 0;
        let playchoice10 = flags7 & 0b0010 != 0;
        let nes2_format  = flags7 & 0b1100 == 0b1100;

        log_bool(log, " => VS Unisystem", vs_unisystem, &col_true, &col_false);
        log_bool(log, " => PlayChoice 10", playchoice10, &col_true, &col_false);
        log_bool(log, " => NES 2.0 format", nes2_format, &col_true, &col_false);
        
        eprintln!();

        if nes2_format {
            eprintln!("NES2.0 format, (flags7: {flags7:08b})");

            panic!("not implemented")

        } else {
            log_var(log, "Header format",  "iNES", &col_enum);

            let prg_rom_size = (prg_rom_size_raw as usize) * 16384;
            let chr_rom_size = (chr_rom_size_raw as usize) * 8192;

            let flags8 = bytes.next().ok_or_else(too_short_err)??;
            let prg_ram_size = (if flags8 == 0 {1u16} else {flags8 as u16}) * 8192;
            

            log_var(log, "PRG ROM bytes", prg_rom_size, &col_number);
            log_var(log, "CHR ROM bytes", chr_rom_size, &col_number);
            log_var(log, "PRG RAM bytes", prg_ram_size, &col_number);
            // eprintln!("PRG_ROM: {prg_rom_size} byte(s) ({prg_rom_size:08x})");
            // eprintln!("CHR_ROM: {chr_rom_size} byte(s) ({chr_rom_size:08x})");


            #[allow(unused_variables)]
            {
                let flags9 = bytes.next().ok_or_else(too_short_err)??;
                log_flags(log, "Byte 9 Flags",  flags9, "T???????", &col_true, &col_false);
                log_var(log, " => TV System", if flags9&1!=0 {"PAL"}else{"NTSC"}, &col_enum);
                let flags10 = bytes.next().ok_or_else(too_short_err)??;
                log_flags(log, "Byte 10 Flags",  flags10, "SS??PB??", &col_true, &col_false);
                log_var(log, " => TV System", match flags10&0b11 {
                    0 => "NTSC",
                    2 => "PAL",
                    _ => "Dual",
                }, &col_enum);
                log_bool(log, " => PRG RAM", flags10&0b0001_0000!=0, &col_true, &col_false);
                log_bool(log, " => Bus conflicts", flags10&0b0010_0000!=0, &col_true, &col_false);
            }

            let mapper_id = (flags6 as u16 & 0xf0) >> 4 | (flags7 as u16 & 0xf0);

            log_var(log, "Mapper ID",  format!("{mapper_id}"), &col_number);
            log_var(log, "Mapper bits",  format!("{mapper_id:08b}"), &col_number);
    
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




