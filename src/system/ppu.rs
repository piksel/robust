use std::hint::unreachable_unchecked;

use super::System;

const PRE_RENDER_LINE: u16 = 245;
const POST_RENDER_LINE: u16 = 240;

pub struct PPU {
    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: [u8; 2],
    addr: u16,
    scroll_y: bool,
    addr_lsb: bool,

    pub scan_line: u16, // column
    pub scan_row: u16,
    // status flags
    // sprite_0_hit: bool,
    // vblank_started: bool,
    pub mono_frame_buffer: [[u8; 256]; 240],

    pub vram: [u8; 2048],
    pub palette: [u8; 31],
    pub bg_patterns: [u16; 2],
    pub bg_palettes: [u8; 2],
}

impl PPU {
    pub fn init() -> Self {
        Self {
            control: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: [0; 2],
            addr: 0,
            scroll_y: false,
            addr_lsb: false,
            scan_line: 0,
            scan_row: 0,
            mono_frame_buffer: [[0; 256]; 240],
            vram: [0; 2048],
            palette: [0; 31],
            bg_patterns: [0; 2],
            bg_palettes: [0; 2],
        }
    }

    pub fn nametable_address(&self) -> u16 {
        match self.control & 0b0000_0011 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!()
        }
    }

    pub fn vram_incr(&self) -> u16 {
        if self.control & 0b0000_0100 == 0 {
            1
        } else {
            32
        }
    }

}

pub(crate) fn tick(sys: &mut System) {

    // increase the scan pos
    if sys.ppu.scan_line >= 341 {
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
            // clear sprite 0 hit 
            sys.ppu.status &= 0b0100_0000;
            // clear vblank
            sys.ppu.status &= 0b1000_0000;
        }

        if row == POST_RENDER_LINE + 1{
            // set vblank
            sys.ppu.status |= 0b1000_0000;
        }
    }



    if row <= 239 {
        if col >= 4 && col < 260 {
            let nm_base = (sys.ppu.nametable_address() - 0x2000) as usize;
            let [scroll_x, scroll_y] = [0, 0];//sys.ppu.scroll;
            let y = (row + scroll_y as u16) as usize;
            let x = ((col - 4) + scroll_x as u16) as usize;

            let nm_y = (y / 8) * 32;
            let nm_x = x / 8;
            let nm_addr = (nm_base + nm_y + nm_x) as usize;

            // eprintln!("{nm_y} ({y}) x {nm_x} ({x}) => {nm_addr}");
            let tile_index = sys.ppu.vram[nm_addr];

            let base_addr = (tile_index as u16) << 4;
            let upper_addr = base_addr + ((y as u16) % 8);
            let lower_addr = upper_addr + 8;


            let cart = sys.cart.as_ref().unwrap();
            let upper_sliver = cart.chr_rom[upper_addr as usize];
            let lower_sliver = cart.chr_rom[lower_addr as usize];

            let mask = 0b1000_0000u8 >> (x % 8);
            let color = if (upper_sliver & mask) != 0 {128u8} else {0u8}
                + if (lower_sliver & mask) != 0 {64} else {0};

            

            if tile_index == 0x2d && mask == 1 {
                // eprintln!("0x{upper_addr:04x}: {upper_sliver:08b} 0x{lower_addr:04x}: {lower_sliver:08b}");

                if (y as u16 % 8) == 7 {
                   
                }
            }


            if (y % 8) == 0 && (x % 8) == 0 {
                // eprintln!("Name Addr for {nm_x:4},{:4} {base_addr:04x} ", nm_y / 32);
            } else {
                // eprintln!("{x} {} {y} {}", x % 8, y % 8);
            }


            sys.ppu.mono_frame_buffer[y][x] = color;
        } else if col > 260 {
            // eprintln!();
        }
    }

    
}

pub(crate) fn write(sys: &mut System, address: u8, value: u8) {
    match address {
        0 => {
            sys.ppu.control = value;
        }
        1 => {
            sys.ppu.mask = value;
        }
        // 2 => {
        //     sys.ppu.status = value;
        // }
        // 3 => {
        //     sys.ppu.oam_addr = value;
        // }
        // 4 => {
        //     sys.ppu.oam_data = value;
        // }
        5 => {
            if sys.ppu.scroll_y {
                sys.ppu.scroll[1] = value;
            } else {
                sys.ppu.scroll[0] = value;
                sys.ppu.scroll[1] = 0;
            }
            sys.ppu.scroll_y = !sys.ppu.scroll_y;
            eprintln!("PPU scroll set to {} by {}!", sys.ppu.scroll[0], sys.ppu.scroll[1]);
        }
        6 => {
            if sys.ppu.addr_lsb {
                sys.ppu.addr |= value as u16;
                
            } else {
                sys.ppu.addr = (value as u16) << 8;
            }
            sys.ppu.addr_lsb = !sys.ppu.addr_lsb;
            eprintln!("PPU address set to 0x{:04x}!", sys.ppu.addr);
        }
        7 => {
            if sys.ppu.addr < 0x2000 {
                // Writing to CHR-RAM. Let's just hope it's RAM...
                sys.ppu.vram[sys.ppu.addr as usize] = value;
            } else if sys.ppu.addr < 0x3000 {
                let addr = mirrored_addr(sys, sys.ppu.addr - 0x2000);

                sys.ppu.vram[addr as usize] = value;

            } else if sys.ppu.addr < 0x3f00 {
                panic!("tried to write {value:02x} to PPU address {:04x}", sys.ppu.addr);
            } else if sys.ppu.addr < 0x3fff {
                let addr = (sys.ppu.addr - 0x3f00) % 0x1f;
                sys.ppu.palette[addr as usize] = value;
            } else {
                panic!("tried to write {value:02x} to PPU address {:04x}", sys.ppu.addr);
            }
            bump_addr(sys);
        }
        _ => panic!("invalid PPU address {address}"),
    }

    eprintln!("Wrote to PPU ${address}: 0x{value:02x}!");
}

fn bump_addr(sys: &mut System) {
    sys.ppu.addr += sys.ppu.vram_incr()
}

fn mirrored_addr(sys: &System, base: u16) -> u16 {
    let vertical_mirroring = sys.cart.as_ref().unwrap().header.vertical_mirroring;

    if vertical_mirroring {
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

pub(crate) fn read(sys: &mut System, address: u8) -> u8 {
    let value = match address {
        0 => sys.ppu.control,
        1 => sys.ppu.mask,
        2 => {
            let value = sys.ppu.status;
             // clear vblank (??)
            sys.ppu.status &= 0b1000_0000;

            // clear address latch
            sys.ppu.addr_lsb = false;
            sys.ppu.scroll_y = false;
            value
        }
        3 => sys.ppu.oam_addr,
        4 => sys.ppu.oam_data,
        5 => panic!("tried to read from PPU SCROLL"), //sys.ppu.scroll,
        6 => panic!("tried to read from PPU ADDRESS"), //sys.ppu.addr,
        7 => panic!("tried to read from PPU DATA"), //sys.ppu.addr,sys.ppu.data,
        _ => panic!("invalid PPU address {address}"),
    };
    if value != 0 {
        eprintln!("Read from PPU ${address}: 0x{value:02x}!");
    }
    value
}