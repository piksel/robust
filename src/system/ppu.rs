use super::System;

const PRE_RENDER_LINE: u16 = 245;
const POST_RENDER_LINE: u16 = 240;

pub struct PPU {
    control: u8,
    /*
    7  bit  0
    ---- ----
    BGRs bMmG
    |||| ||||
    |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    |||| +---- 1: Show background
    |||+------ 1: Show sprites
    ||+------- Emphasize red (green on PAL/Dendy)
    |+-------- Emphasize green (red on PAL/Dendy)
    +--------- Emphasize blue
    */
    mask: u8,
    pub status: u8,
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

    pub vram: [u8; 0x4000],
    pub palette: [u8; 32],
    pub bg_patterns: [u16; 2],
    pub bg_palettes: [u8; 2],
}

impl PPU {

    pub const palette_colors: [u32; 64] = [
        0x545454, 0x001e74, 0x081090, 0x300088, 0x440064, 0x5c0030, 0x540400, 0x3c1800, 0x202a00, 0x083a00, 0x004000, 0x003c00, 0x00323c, 0x000000, 0x000000, 0x000000,
        0x989698, 0x084cc4, 0x3032ec, 0x5c1ee4, 0x8814b0, 0xa01464, 0x982220, 0x783c00, 0x545a00, 0x287200, 0x087c00, 0x007628, 0x006678, 0x000000, 0x000000, 0x000000,
        0xeceeec, 0x4c9aec, 0x787cec, 0xb062ec, 0xe454ec, 0xec58b4, 0xec6a64, 0xd48820, 0xa0aa00, 0x74c400, 0x4cd020, 0x38cc6c, 0x38b4cc, 0x3c3c3c, 0x000000, 0x000000,
        0xeceeec, 0xa8ccec, 0xbcbcec, 0xd4b2ec, 0xecaeec, 0xecaed4, 0xecb4b0, 0xe4c490, 0xccd278, 0xb4de78, 0xa8e290, 0x98e2b4, 0xa0d6e4, 0xa0a2a0, 0x000000, 0x000000,
    ];

    pub fn init() -> Self {
        Self {
            control: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            data: 0,
            scroll: [0; 2],
            addr: 0,
            scroll_y: false,
            addr_lsb: false,
            scan_line: 21,
            scan_row: 0,
            frame_buffer: [[0u32; 256]; 240],
            vram: [0; 0x4000],
            palette: [0; 32],
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
            // clear sprite 0 hit 
            sys.ppu.status &= 0b0100_0000;
            // clear vblank
            sys.ppu.status &= 0b1000_0000;
        }

        if row == POST_RENDER_LINE + 1{
            // set vblank
            sys.ppu.status |= 0b1000_0000;

            if sys.ppu.control & 0b1000_0000 != 0 {
                sys.trigger_nmi()
            }
        }
    }



    if row <= 239 {
        if col >= 4 && col < 260 {
            let nm_base = (sys.ppu.nametable_address() - 0x2000) as usize;
            let at_base = nm_base + 0x3c0  as usize;
            let [scroll_x, scroll_y] = sys.ppu.scroll;
            let y = (row as u16 + scroll_y as u16) as usize;
            let nx = (col - 4) as usize;
            let x = (nx + scroll_x as usize) as usize;

            let enable_mask = if x <= 8 {sys.ppu.mask << 2} else {sys.ppu.mask};
            let enable_bg = enable_mask & 0b0000_1000 != 0;
            let enable_fg = enable_mask & 0b0001_0000 != 0;

            let nm_y = (y / 8) * 32;
            let nm_x = x / 8;
            let nm_addr = (nm_base + nm_y + nm_x) as u16;
            let at_addr = (at_base + (8*(y/32)) + (x/32)) as u16;

            // eprintln!("{nm_y} ({y}) x {nm_x} ({x}) => {nm_addr}");
            let tile_index = sys.ppu.vram[mirrored_addr(sys, nm_addr) as usize];
            let attributes = sys.ppu.vram[mirrored_addr(sys, at_addr) as usize];

            let cbits = match ((x / 16) % 2, (y / 16) % 2) {
                (0,0) => attributes & 0x3, // topleft
                (1,0) => attributes >> 2 & 0x3, // topright
                (0,1) => attributes >> 4 & 0x3, // bottomleft
                (1,1) => attributes >> 6 & 0x3, // bottomright
                (y,x) => panic!("invalid attribute pos {x},{y}"),
            };

            let ctrl = sys.ppu.control;
            let base_addr = ((tile_index as u16) << 4) + if ctrl&0b1_0000!=0{0x1000}else{0};
            let cart = sys.cart.as_ref().unwrap();
            let (upper_sliver, lower_sliver) = cart.get_tile(base_addr + ((y as u16) % 8))?;




            let mask = 0b1000_0000u8 >> (x % 8);
            let color_index = (
                if upper_sliver & mask != 0 {1 << 0} else {0} |
                if lower_sliver & mask != 0 {1 << 1} else {0} |
                cbits << 2
            );

            // if nm_x == 20 && y/8 == 6 && color_index != 0 {
            //     eprint!("X: {:3} Y: {:3} => ", x/8, y / 8);
            //     eprint!("X: {:3} Y: {:3} => ", x/32, y / 32);
            //     eprintln!("Cbits: {cbits:08b} atPart: {}, {}", nm_x % 2, nm_y % 2);
            //     eprintln!("CI: {color_index:02x} {color_index:04b} A: {attributes:02x} {attributes:08b}");
            //     eprintln!("NTAddr: {nm_addr:04x} ATAddr: {at_addr:04x}");
            //     eprintln!("Tile: {tile_index:02x} {tile_index:08b}");
            // }

            // if (color_index != 0) {
            //     eprintln!("Color index: {color_index} {color_index:02x} {color_index:04b}");
            //     let pal_col = sys.ppu.palette[color_index as usize];
            //     panic!("PalCol: {pal_col} {pal_col:02x} {pal_col:04b}");
            // }
            /*
                if (upper_sliver & mask) != 0 {128u8} else {0u8}
                + if (lower_sliver & mask) != 0 {64} else {0}
                + if (cbits & 0x1) != 0 {32} else {0}
                + if (cbits & 0x2) != 0 {16} else {0};
            */
            

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

            if enable_bg {
                let pal_index = sys.ppu.palette[color_index as usize] % 64;
                sys.ppu.frame_buffer[y][nx] = PPU::palette_colors[pal_index as usize];
            }

            // eprint!("")

            // TODO: Sprites!
            for i in 0..64 {
                let sprite_base = i * 4;
                let y_pos = sys.oam[sprite_base + 0] as usize;
                let tile_index = sys.oam[sprite_base + 1];
                let attrs = sys.oam[sprite_base + 2];
                let x_pos = sys.oam[sprite_base + 3] as usize;
                if y >= y_pos && y < (y_pos+8) && nx >= x_pos && nx < (x_pos+8) {
                    let base_addr = (tile_index as u16) << 4 + if ctrl&0b1000!=0{0x1000}else{0};
                    let cbits = attrs & 0b0000_0011;
                    let flip_h = attrs & 0b0100_0000 != 0;
                    let flip_v = attrs & 0b1000_0000 != 0;

                    let nsy = y - y_pos as usize;
                    let nsx = nx - x_pos as usize;
                    let tile_y = if flip_v {8 - ((nsy as u16) % 8)} else {(nsy as u16) % 8};
                    let (upper_sliver, lower_sliver) = cart.get_tile(base_addr + tile_y)?;

             
                    
                    let mask = if flip_h {0b1u8 << (nsx % 8)} else {0b1000_0000u8 >> (nsx % 8)};
                    let color_index = (
                        if lower_sliver & mask != 0 {1 << 0} else {0} |
                        if upper_sliver & mask != 0 {1 << 1} else {0} |
                        cbits << 2
                    );

                    let behind =   attrs & 0b0010_0000 != 0;
                    
                    if enable_fg /* && !behind */ {
                        // eprint!("{i:02x} ");
                        sys.ppu.frame_buffer[y][nx] = if color_index == 0 {
                            sys.ppu.frame_buffer[y][nx]
                        } else {
                            PPU::palette_colors[sys.ppu.palette[color_index as usize] as usize]
                        };
                        // sys.ppu.frame_buffer[y][x] = PPU::palette_colors[sys.ppu.palette[color_index as usize] as usize];
                    }
                    
                    if i == 0 && cbits != 00 && enable_bg && enable_fg {

                        /*
                        TODO: Sprite 0 hit does not happen:
                            If background or sprite rendering is disabled in PPUMASK ($2001)
                            At x=0 to x=7 if the left-side clipping window is enabled (if bit 2 or bit 1 of PPUMASK is 0).
                            At x=255, for an obscure reason related to the pixel pipeline.
                          x At any pixel where the background or sprite pixel is transparent (2-bit color index from the CHR pattern is %00).
                            If sprite 0 hit has already occurred this frame. Bit 6 of PPUSTATUS ($2002) is cleared to 0 at dot 1 of the pre-render line. This means only the first sprite 0 hit in a frame can be detected.
                        */
                        
                        // Sprite zero hit!
                        sys.ppu.status |= 0b0100_0000;
                        // eprintln!("SPRITE0 HIT")
                    }

                }

            }

        } else if col > 260 {
            // eprintln!();
        }
    }
    Ok(())
    
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
                sys.ppu.vram[sys.ppu.addr as usize] = value;
            } else if sys.ppu.addr < 0x3000 {
                let addr = mirrored_addr(sys, sys.ppu.addr - 0x2000);

                sys.ppu.vram[addr as usize] = value;

            } else if sys.ppu.addr < 0x3f00 {
                panic!("tried to write {value:02x} to PPU address {:04x}", sys.ppu.addr);
            } else if sys.ppu.addr < 0x3fff {
                let mut addr = (sys.ppu.addr - 0x3f00) % 0x20;
                if addr % 4 == 0 {
                    addr &= 0x0f
                }
                eprintln!("Wrote palette entry {value:02x} to {:04x}", addr);
                sys.ppu.palette[addr as usize] = value;
            } else {
                panic!("tried to write {value:02x} to PPU address {:04x}", sys.ppu.addr);
            }
            bump_addr(sys);
        }
        _ => panic!("invalid PPU address {address}"),
     }

    // eprintln!("Wrote to PPU ${address}: 0x{value:02x}!");
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
                sys.ppu.vram[sys.ppu.addr as usize]
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
    value
}