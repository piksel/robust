use std::fmt;


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
pub struct Mask {
    pub greyscale: bool,
    pub enable_start_bg: bool,
    pub enable_start_fg: bool,
    pub enable_bg: bool,
    pub enable_fg: bool,
    pub emphasis_red: bool,
    pub emphasis_green: bool,
    pub emphasis_blue: bool,
}

impl Default for Mask {
    fn default() -> Self {
        Self {
            greyscale: false,
            enable_start_bg: false,
            enable_start_fg: false,
            enable_bg: false,
            enable_fg: false,
            emphasis_red: false,
            emphasis_green: false,
            emphasis_blue: false,
         }
    }
}

impl From<u8> for Mask {
    fn from(value: u8) -> Self {
        Self {
            greyscale      : value & 0b0000_0001 != 0,
            enable_start_bg: value & 0b0000_0010 != 0,
            enable_start_fg: value & 0b0000_0100 != 0,
            enable_bg      : value & 0b0000_1000 != 0,
            enable_fg      : value & 0b0001_0000 != 0,
            emphasis_red   : value & 0b0010_0000 != 0,
            emphasis_green : value & 0b0100_0000 != 0,
            emphasis_blue  : value & 0b1000_0000 != 0,
        }
    }
}

pub struct Control {
    pub nametable_address: u16,
    pub vram_incr: u16,
    pub tall_sprites: bool,
    pub pattern_base_fg: u16,
    pub pattern_base_bg: u16,
    pub ppu_bg_out: bool,
    pub enable_nmi: bool,
}


impl Default for Control {
    fn default() -> Self {
        Self { 
            nametable_address: 0x2000,
            vram_incr: 1,
            tall_sprites: false,
            pattern_base_fg: 0,
            pattern_base_bg: 0,
            ppu_bg_out: false,
            enable_nmi: false,
        }
    }
}

impl fmt::Debug for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Control")
         .field("nametable_address", &format_args!("{:04x}", &self.nametable_address))
         .field("vram_incr", &self.vram_incr)
         .field("tall_sprites", &self.tall_sprites)
         .field("pattern_base_fg", &format_args!("{:04x}", &self.pattern_base_fg))
         .field("pattern_base_bg", &format_args!("{:04x}", &self.pattern_base_bg))
         .field("ppu_bg_out", &self.ppu_bg_out)
         .field("enable_nmi", &self.enable_nmi)
         .finish()
    }
}

impl From<u8> for Control {
    fn from(value: u8) -> Self {
        // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
        let tall_sprites = value & 0b0010_0000 != 0;

        Self {
            // 7  bit  0
            // ---- ----
            // VPHB SINN
            // |||| ||||
            // |||| ||++- Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
            nametable_address: match value & 0b0000_0011 {
                0 => 0x2000,
                1 => 0x2400,
                2 => 0x2800,
                3 => 0x2C00,
                _ => unreachable!()
            },
            // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
            // |||| |     (0: add 1, going across; 1: add 32, going down)
            vram_incr: if value & 0b0000_0100 != 0 {32} else {1},
            // |||| +---- Sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; ignored in 8x16 mode)
            pattern_base_fg: if value & 0b0000_1000 != 0 && !tall_sprites {0x1000} else {0},
            // |||+------ Background pattern table address (0: $0000; 1: $1000)
            pattern_base_bg: if value & 0b0001_0000 != 0 {0x1000} else {0},
            tall_sprites,
            // |+-------- PPU master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)  
            ppu_bg_out: value & 0b0100_0000 != 0,
            // +--------- Generate an NMI at the start of the vertical blanking interval (0: off; 1: on)    
            enable_nmi: value & 0b1000_0000 != 0,
        }
    }
}

pub struct Status {
    pub sprite_zero_hit: bool,
    pub vertical_blank: bool,
}

impl Default for Status {
    fn default() -> Self {
        Self { 
            sprite_zero_hit: false,
            vertical_blank: false,
        }
    }
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Self {
            sprite_zero_hit: value & 0b0100_0000 != 0,
            vertical_blank:  value & 0b1000_0000 != 0,
        }
    }
}

impl From<&Status> for u8 {
    fn from(value: &Status) -> Self {
        0
        | if value.sprite_zero_hit {0b0100_0000} else {0}
        | if value.vertical_blank  {0b1000_0000} else {0}
    }
}