use crate::system::System;

use super::{mirrored_addr, PPU};



fn draw_bg(sys: &mut System, nx: usize, ny: usize) -> anyhow::Result<u8> {
    let nm_base = (sys.ppu.control.nametable_address - 0x2000) as usize;
    let at_base = nm_base + 0x3c0  as usize;
    let [scroll_x, scroll_y] = sys.ppu.scroll;
    let y = (ny + scroll_y as usize) as usize;
    let x = (nx + scroll_x as usize) as usize;
    let nm_y = ((y / 8) * 32);
    let nm_x = x / 8;
    let nm_addr = (nm_base + nm_y + nm_x + if x>255 {992}else{0}) as u16;
    let at_addr = (at_base + (8*(y/32)) + (x/32) + if x>255 {sys.offset}else{0}) as u16;

    let enable_bg = if x <= 8 {sys.ppu.mask.enable_start_bg} else {sys.ppu.mask.enable_bg};

    // eprintln!("{nm_y} ({y}) x {nm_x} ({x}) => {nm_addr}");
    let tile_index = sys.ppu.vram[mirrored_addr(sys, nm_addr) as usize];
    let attributes = sys.ppu.vram[mirrored_addr(sys, at_addr) as usize];

    let cbits = match ((x / 16) % 2, (y / 16) % 2) {
        (0,0) => attributes & 0x3,      // top left
        (1,0) => attributes >> 2 & 0x3, // top right
        (0,1) => attributes >> 4 & 0x3, // bottom left
        (1,1) => attributes >> 6 & 0x3, // bottom right
        (y,x) => panic!("invalid attribute pos {x},{y}"),
    };

    let ctrl = &sys.ppu.control;
    let base_addr = ((tile_index as u16) << 4) + ctrl.pattern_base_bg;
    let cart = sys.cart.as_ref().unwrap();
    let (upper_sliver, lower_sliver) = cart.get_tile(base_addr + ((y as u16) % 8))?;




    let mask = 0b1000_0000u8 >> (x % 8);
    let mut color_index = (
        if upper_sliver & mask != 0 {1 << 0} else {0} |
        if lower_sliver & mask != 0 {1 << 1} else {0} |
        cbits << 2
    );

    if color_index % 4 == 0 {

        color_index &= 0x10;
    }

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
        Ok(sys.ppu.palette[color_index as usize])
        // sys.ppu.frame_buffer[y][nx] = PPU::palette_colors[pal_index as usize];
    } else {Ok(0)}
}

fn eval_fg(sys: &mut System, nx: usize, ny: usize) -> anyhow::Result<()> {

    let [scroll_x, scroll_y] = sys.ppu.scroll;
    let y = (ny + scroll_y as usize) as usize;
    let x = (nx + scroll_x as usize) as usize;

    // eprint!("")

    //let mut behind = false;

    // TODO: Sprites!
    for i in 0..64 {
        let sprite_base = i * 4;
        let y_pos = sys.oam[sprite_base + 0] as usize;
        let tile_index = sys.oam[sprite_base + 1];
        let attrs = sys.oam[sprite_base + 2];
        let x_pos = sys.oam[sprite_base + 3] as usize;
        if y >= y_pos && y < (y_pos+8) && nx >= x_pos && nx < (x_pos+8) {
            sys.ppu.sprite_outputs.push([y_pos as u8, tile_index, attrs, x_pos as u8, i as u8]);
        }
    }

    Ok(())
}
fn draw_fg(sys: &mut System, nx: usize, ny: usize) -> anyhow::Result<Option<(u8, bool)>> {
    // if ny == 0 {
    //     return Ok(None);
    // }
    
    let [scroll_x, scroll_y] = sys.ppu.scroll;
    let y = (ny + scroll_y as usize) as usize;
    let x = (nx + scroll_x as usize) as usize;

    let enable_fg = if x <= 8 {sys.ppu.mask.enable_start_fg} else {sys.ppu.mask.enable_fg};
    let enable_bg = if x <= 8 {sys.ppu.mask.enable_start_bg} else {sys.ppu.mask.enable_bg};

    // eprint!("")

    let mut color = None;

    for [y_pos, tile_index, attrs, x_pos, i] in sys.ppu.sprite_outputs.iter() {
        
        let y_pos = *y_pos as usize;
        let x_pos = *x_pos as usize;
       
        let base_addr = ((*tile_index as u16) << 4) + sys.ppu.control.pattern_base_fg;
        let cbits = attrs & 0b0000_0011;
        let flip_h = attrs & 0b0100_0000 != 0;
        let flip_v = attrs & 0b1000_0000 != 0;

        let nsy = ny - y_pos as usize;
        let nsx = nx - x_pos as usize;
        let tile_y = if flip_v {8 - ((nsy as u16) % 8)} else {(nsy as u16) % 8};
        let (upper_sliver, lower_sliver) = sys.cart.as_ref().unwrap().get_tile(base_addr + tile_y)?;

    
        
        let mask = if flip_h {0b1u8 << (nsx % 8)} else {0b1000_0000u8 >> (nsx % 8)};
        let color_index = (
            if upper_sliver & mask != 0 {1 << 0} else {0} |
            if lower_sliver & mask != 0 {1 << 1} else {0} |
            cbits << 2
        );

        let behind =   attrs & 0b0010_0000 != 0;
        
        if enable_fg && color_index % 4 != 0 /* && !behind */ {
            color = Some((sys.ppu.palette[color_index as usize + 0x10], behind));
            // eprint!("{i:02x} ");
            // sys.ppu.frame_buffer[y][nx] = if color_index == 0 {
            //     sys.ppu.frame_buffer[y][nx]
            // } else {
            //     PPU::palette_colors[sys.ppu.palette[color_index as usize] as usize]
            // };
            // sys.ppu.frame_buffer[y][x] = PPU::palette_colors[sys.ppu.palette[color_index as usize] as usize];
        };
        
        if *i == 0 && (color_index % 4 != 0) && enable_bg && enable_fg {

            /*
            TODO: Sprite 0 hit does not happen:
                If background or sprite rendering is disabled in PPUMASK ($2001)
                At x=0 to x=7 if the left-side clipping window is enabled (if bit 2 or bit 1 of PPUMASK is 0).
                At x=255, for an obscure reason related to the pixel pipeline.
                x At any pixel where the background or sprite pixel is transparent (2-bit color index from the CHR pattern is %00).
                If sprite 0 hit has already occurred this frame. Bit 6 of PPUSTATUS ($2002) is cleared to 0 at dot 1 of the pre-render line. This means only the first sprite 0 hit in a frame can be detected.
            */
            
            // Sprite zero hit!
            sys.ppu.status.sprite_zero_hit = true;
            // eprintln!("SPRITE0 HIT")
        }
        
    }

    sys.ppu.sprite_outputs.clear();

    Ok(color)
}

pub fn draw(sys: &mut System, nx: usize, ny: usize) -> anyhow::Result<()> {


    let bg_col = draw_bg(sys, nx, ny + 1)?;
    let fg_col = draw_fg(sys, nx - 1, ny)?;
    let pixel_col = match fg_col {
        None => bg_col,
        // (0, _fg, _) => Some(fg_col),
        Some(( _, true)) => bg_col,
        Some((fg, false)) => fg,
    };


    sys.ppu.frame_buffer[ny][nx] = PPU::palette_colors[pixel_col as usize];
    
    eval_fg(sys, nx, ny)?;
    
    Ok(())
}