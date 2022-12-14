use crate::system::System;

use super::{mirrored_addr, PPU};



fn draw_bg(sys: &mut System, nx: usize, ny: usize) -> anyhow::Result<Option<u8>> {
    let nm_base = (sys.ppu.control.nametable_address - 0x2000) as usize;
    let at_base = nm_base + 0x3c0  as usize;
    let [scroll_x, scroll_y] = sys.ppu.scroll;
    let y = (ny + scroll_y as usize) as usize;
    let x = (nx + scroll_x as usize) as usize;
    let second_table = x > 255;
    let nm_y = ((y / 8) + if second_table {31} else {0} ) * 32;
    let nm_x = x / 8;
    let nm_addr = (nm_base + nm_y + nm_x) as u16;
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
    let (upper_sliver, lower_sliver) = sys.cart.get_tile(base_addr + ((y as u16) % 8))?;




    let mask = 0b1000_0000u8 >> (x % 8);
    let color_index = (
        if upper_sliver & mask != 0 {1 << 0} else {0} |
        if lower_sliver & mask != 0 {1 << 1} else {0} |
        cbits << 2
    );

    if color_index % 4 == 0 {
        // color_index &= 0x10;
        return Ok(None)
    }

    if enable_bg {
        Ok(Some(sys.ppu.palette[color_index as usize]))
    } else {Ok(None)}
}

fn eval_fg(sys: &mut System, _nx: usize, ny: usize) -> anyhow::Result<()> {

    for i in 0..64 {
        let sprite_base = i * 4;
        let y_pos = sys.oam[sprite_base + 0] as usize;
        let tile_index = sys.oam[sprite_base + 1];
        let attrs = sys.oam[sprite_base + 2];
        let x_pos = sys.oam[sprite_base + 3] as usize;
        if ny >= y_pos && ny < (y_pos+8) {
            let flip_v = attrs & 0b1000_0000 != 0;
            let base_addr = ((tile_index as u16) << 4) + sys.ppu.control.pattern_base_fg;
            let nsy = ny - y_pos as usize;
            let tile_y = if flip_v {8 - ((nsy as u16) % 8)} else {(nsy as u16) % 8};
            let (upper_sliver, lower_sliver) = sys.cart.get_tile(base_addr + tile_y)?;
    

            sys.ppu.sprite_outputs.push([y_pos as u8, tile_index, attrs, x_pos as u8, i as u8, upper_sliver, lower_sliver]);
        }
    }
    // && nx >= x_pos && nx < (x_pos+8)

    Ok(())
}

fn draw_fg(sys: &mut System, nx: usize, _ny: usize) -> anyhow::Result<Option<(u8, bool)>> {

    let [scroll_x, _] = sys.ppu.scroll;
    let x = (nx + scroll_x as usize) as usize;

    let enable_fg = if x <= 8 {sys.ppu.mask.enable_start_fg} else {sys.ppu.mask.enable_fg};
    let enable_bg = if x <= 8 {sys.ppu.mask.enable_start_bg} else {sys.ppu.mask.enable_bg};

    let mut color = None;

    for (out_i, [_y_pos, _tile_index, attrs, x_pos, i, tile_high, tile_low]) in sys.ppu.sprite_outputs.iter().enumerate() {
        
        // let y_pos = *y_pos as usize;
        let x_pos = *x_pos as usize;

        if nx < x_pos || nx >= (x_pos+8) || x_pos == 0 {continue}
       
        let cbits = attrs & 0b0000_0011;
        let flip_h = attrs & 0b0100_0000 != 0;
        
        
        
        let nsx = nx - x_pos as usize;

        let mask = if flip_h {0b1u8 << (nsx % 8)} else {0b1000_0000u8 >> (nsx % 8)};
        let color_index = (
            if tile_high & mask != 0 {1 << 0} else {0} |
            if tile_low & mask != 0 {1 << 1} else {0} |
            cbits << 2
        );

        let behind =   attrs & 0b0010_0000 != 0;
        
        if enable_fg && color_index % 4 != 0 /* && !behind */ {
            color = if sys.opts.sprite_order_overlay {
                Some((0x40 + out_i as u8, behind))
            } else {
                Some((sys.ppu.palette[color_index as usize + 0x10], behind))
            }
            // color = ;
            
            // eprintln!("Drawing sprite {out_i} or {}", sys.ppu.sprite_outputs.len());
        };
        
        if *i == 0 && 
            // At any pixel where the sprite pixel is transparent (2-bit color index from the CHR pattern is %00).
            (color_index % 4 != 0) && 
            // If background or sprite rendering is disabled in PPUMASK ($2001)
            enable_bg && enable_fg && 
            // At x=255, for an obscure reason related to the pixel pipeline.
            nx < 255
            {

            /*
            TODO: Sprite 0 hit does not happen:
                ?????? If background or sprite rendering is disabled in PPUMASK ($2001)
                ?????? At x=0 to x=7 if the left-side clipping window is enabled (if bit 2 or bit 1 of PPUMASK is 0).
                ?????? At x=255, for an obscure reason related to the pixel pipeline.
                ?????? At any pixel where the sprite pixel is transparent (2-bit color index from the CHR pattern is %00).
                ??? At any pixel where the background pixel is transparent (2-bit color index from the CHR pattern is %00).
                (implicit) If sprite 0 hit has already occurred this frame. Bit 6 of PPUSTATUS ($2002) is cleared to 0 at dot 1 of the pre-render line. This means only the first sprite 0 hit in a frame can be detected.
            */
            sys.ppu.status.sprite_zero_hit = true;
            // color = Some((255, behind))
        }
        
    }

    // sys.ppu.sprite_outputs.clear();

    Ok(color)
}

pub fn draw(sys: &mut System, nx: usize, ny: usize) -> anyhow::Result<()> {



    // let mono = sys.ppu.sprite_outputs.len() as u32 * 32;
    // let debug_col = (mono & 0xff) |
    //     (mono & 0xff) << 8 |
    //     if mono > 255 {0xff} else {mono & 0xff} << 16;

    let bg_col = draw_bg(sys, nx, ny + 1)?;
    let fg_col = draw_fg(sys, nx, ny)?;
    let palette_col = match (bg_col, fg_col) {
        (None, None) => sys.ppu.palette[0],
        (Some(bg), None) => bg,
        // (0, _fg, _) => Some(fg_col),
        (None, Some((fg , true))) => fg,
        (Some(bg), Some(( _, true))) => bg,
        (_, Some((fg, false))) => fg,
    };


    
    let pixel_col = if palette_col >= 64 {
        // debug color
        match palette_col {
            0x40 => 0xff0000,
            0x41 => 0xffa500,
            0x42 => 0xffff00,
            0x43 => 0x008000,
            0x44 => 0x00ffff,
            0x45 => 0x0000ff,
            0x46 => 0x4b0082,
            0x47 => 0xee82ee,
            _ => 0x00ff00,
        }
    } else {
        PPU::PALETTE_COLORS[palette_col as usize]
    };

    

    sys.ppu.frame_buffer[ny][nx] = pixel_col;
    
    // eval_fg(sys, nx, ny)?;
    
    if nx == 0 {
        sys.ppu.sprite_outputs.clear();
        eval_fg(sys, nx, ny)?;
    }

    Ok(())
}