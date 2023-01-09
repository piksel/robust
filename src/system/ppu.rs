use crate::system::addr::Addr;

use super::System;
use draw::draw;
use registers::{Control, Status, Mask};

mod draw;
mod registers;


const PRE_RENDER_LINE: u16 = 245;
const POST_RENDER_LINE: u16 = 240;



pub struct PPU {
    control: Control,
    mask: Mask,

    pub status: Status,
    pub(crate) oam_addr: u8,
    data: u8,
    scroll: [u8; 2],
    addr: u16,
    scroll_y: bool,
    addr_lsb: bool,

    pub scan_line: u16, // column
    pub scan_row: u16,
    // status flags
    // sprite_0_hit: bool,
    // vblank_started: bool,
    // pub mono_frame_buffer: [[u8; 256]; 240],
    pub frame_buffer: [[u32; 256]; 240],

    pub vram: Vec<u8>,
    pub palette: [u8; 32],
    pub bg_patterns: [u16; 2],
    pub bg_palettes: [u8; 2],

    pub sprite_outputs: Vec<[u8; 7]>,
}

impl PPU {

    pub const PALETTE_COLORS: [u32; 64] = [
        0x545454, 0x001e74, 0x081090, 0x300088, 0x440064, 0x5c0030, 0x540400, 0x3c1800, 0x202a00, 0x083a00, 0x004000, 0x003c00, 0x00323c, 0x000000, 0x000000, 0x000000,
        0x989698, 0x084cc4, 0x3032ec, 0x5c1ee4, 0x8814b0, 0xa01464, 0x982220, 0x783c00, 0x545a00, 0x287200, 0x087c00, 0x007628, 0x006678, 0x000000, 0x000000, 0x000000,
        0xeceeec, 0x4c9aec, 0x787cec, 0xb062ec, 0xe454ec, 0xec58b4, 0xec6a64, 0xd48820, 0xa0aa00, 0x74c400, 0x4cd020, 0x38cc6c, 0x38b4cc, 0x3c3c3c, 0x000000, 0x000000,
        0xeceeec, 0xa8ccec, 0xbcbcec, 0xd4b2ec, 0xecaeec, 0xecaed4, 0xecb4b0, 0xe4c490, 0xccd278, 0xb4de78, 0xa8e290, 0x98e2b4, 0xa0d6e4, 0xa0a2a0, 0x000000, 0x000000,
    ];

    pub fn init() -> Self {
        Self {
            control: Control::default(),


            mask: Mask::default(),

            status: Status::default(),
            oam_addr: 0,
            data: 0,
            scroll: [0; 2],
            addr: 0,
            scroll_y: false,
            addr_lsb: false,
            scan_line: 21,
            scan_row: 0,
            frame_buffer: [[0u32; 256]; 240],
            vram: vec![0; 0x4000],
            palette: [0; 32],
            bg_patterns: [0; 2],
            bg_palettes: [0; 2],

            sprite_outputs: Vec::with_capacity(8)
        }
    }

    // pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
    //     Self::palette_colors[self.mono_frame_buffer[y][x] as usize]
    // }

}

pub(crate) fn tick(sys: &mut System) -> anyhow::Result<()> {

    // increase the scan pos
    if sys.ppu.scan_line >= 340 {
        sys.ppu.scan_line = 0;
        sys.ppu.scan_row += 1;
        if sys.ppu.scan_row > 261 {
            sys.ppu.scan_row = 0;
        }
    } else {
        sys.ppu.scan_line += 1;
    }

    let col = sys.ppu.scan_line;
    let row = sys.ppu.scan_row;

    // dot 1
    if col == 0 {
        if row == PRE_RENDER_LINE {
            sys.ppu.status.sprite_zero_hit = false;
            sys.ppu.status.vertical_blank = false;
        }

        if row == POST_RENDER_LINE + 1{
            // set vblank
            sys.ppu.status.vertical_blank = true;

            if sys.ppu.control.enable_nmi {
                sys.trigger_nmi()
            }
        }
    }



    if row <= 239 {
        if col >= 1 && col < 257 {
            draw(sys, col as usize - 1, row as usize)?;
        } else if col == 257 {
            // eprintln!();
        }
    }
    Ok(())
    
}

pub(crate) fn write(sys: &mut System, address: u8, value: u8) -> anyhow::Result<()> {
    match address {
        0 => {
            sys.ppu.control = value.into();
            // eprintln!("PPU Control set from {value:02x} ({value:08b}) to {:?}", sys.ppu.control);
            
        }
        1 => {
            sys.ppu.mask = value.into();
            
        }
        // 2 => {
        //     sys.ppu.status = value;
        // }
        3 => {
            sys.ppu.oam_addr = value;
            // eprintln!("Wrote OAM Addr: {value:04x}");
        }
        4 => {
            sys.oam[sys.ppu.oam_addr as usize] = value;
            eprintln!("Wrote OAM Data: {value:04x}");
            // sys.ppu.oam_data = value;
            // panic!()
            let _ = sys.ppu.oam_addr.wrapping_add(1);
        }
        5 => {
            if sys.ppu.scroll_y {
                sys.ppu.scroll[1] = value;
            } else {
                sys.ppu.scroll[0] = value;
                sys.ppu.scroll[1] = 0;
            }
            sys.ppu.scroll_y = !sys.ppu.scroll_y;
            // eprintln!("PPU scroll set to {} by {}!", sys.ppu.scroll[0], sys.ppu.scroll[1]);
        }
        6 => {
            if sys.ppu.addr_lsb {
                sys.ppu.addr |= value as u16;
                
            } else {
                sys.ppu.addr = (value as u16) << 8;
            }
            sys.ppu.addr_lsb = !sys.ppu.addr_lsb;
            // eprintln!("PPU address set to 0x{:04x}!", sys.ppu.addr);
        }
        7 => {
            if sys.ppu.addr < 0x2000 {
                // Writing to CHR-RAM. Let's just hope it's RAM...
                sys.cart.mapper.ppu_write(sys.ppu.addr.into(), value)?;
                // sys.ppu.vram[sys.ppu.addr as usize] = value;
            } else if sys.ppu.addr < 0x3000 {
                if sys.ppu.addr == 0x27a0 {
                    println!("Writing {value:02x} to {}!", Addr(sys.ppu.addr));
                }
                let addr = mirrored_addr(sys, sys.ppu.addr - 0x2000);

                sys.ppu.vram[addr as usize] = value;

            } else if sys.ppu.addr < 0x3f00 {
                panic!("tried to write {value:02x} to PPU address {:04x}", sys.ppu.addr);
            } else if sys.ppu.addr < 0x3fff {
                let mut addr = (sys.ppu.addr - 0x3f00) % 0x20;
                if addr % 4 == 0 {
                    addr &= 0x0f
                }
                // eprintln!("Wrote palette entry {value:02x} to {:04x}", addr);
                sys.ppu.palette[addr as usize] = value;
            } else {
                panic!("tried to write {value:02x} to PPU address {:04x}", sys.ppu.addr);
            }
            bump_addr(sys);
        }
        _ => anyhow::bail!("invalid PPU address {address}"),
     }
     Ok(())

    // eprintln!("Wrote to PPU ${address}: 0x{value:02x}!");
}

fn bump_addr(sys: &mut System) {
    sys.ppu.addr += sys.ppu.control.vram_incr
}

fn mirrored_addr(sys: &System, base: u16) -> u16 {
    let vertical_mirroring = sys.cart.header.vertical_mirroring;
    if vertical_mirroring {
        // eprintln!("Writing to {:04x} wrapped from {:04x}", base % 0x800, base);
        base % 0x800
    } else {
        if base < 0x800 {
            // All addresses are within table A
            base % 0x400
        } else {
            // All addresses are within table B
            0x400 + (base % 0x400)
        }
    }
}

pub(crate) fn read(sys: &mut System, address: u8) -> anyhow::Result<u8> {
    let value = match address {
        0 => panic!("tried to read from PPU CONTROL"),
        1 => panic!("tried to read from PPU MASK"),
        2 => {
            let value = (&sys.ppu.status).into();
             // clear vblank (??)
            sys.ppu.status.vertical_blank = false;

            // clear address latch
            sys.ppu.addr_lsb = false;
            sys.ppu.scroll_y = false;

            // eprintln!("Read from PPUSTATUS: {value:08b}");

            value
        }
        3 => panic!("tried to read from OAM ADDR"),
        4 => {

            panic!("tried to read from OAM DATA")
        },
        5 => panic!("tried to read from PPU SCROLL"), //sys.ppu.scroll,
        6 => panic!("tried to read from PPU ADDRESS"), //sys.ppu.addr,
        7 => {
            let value = sys.ppu.data;

            sys.ppu.data = if sys.ppu.addr < 0x2000 {
                // Reading from to CHR-RAM
                //sys.ppu.vram[sys.ppu.addr as usize]
                sys.cart.mapper.ppu_read(sys.ppu.addr.into())?
            } else if sys.ppu.addr < 0x3000 {
                let addr = mirrored_addr(sys, sys.ppu.addr - 0x2000);

                sys.ppu.vram[addr as usize]
            } else if sys.ppu.addr < 0x3fff {
                sys.ppu.vram[sys.ppu.addr as usize]
            } else {
                panic!("Tried to read from PPUDATA {:04x}", sys.ppu.addr)
            };

            bump_addr(sys);

            value
        }
        _ => panic!("invalid PPU address {address}"),
    };
    if value != 0 {
        // eprintln!("Read from PPU ${address}: 0x{value:02x}!");
    }
    Ok(value)
}