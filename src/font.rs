// const WIDTH: usize = 512;
// const HEIGHT: usize = 480;
// const CHAR_HEIGHT: usize = 18;
// const CHAR_WIDTH: usize = 10;

pub struct Font {
    bitmap: Vec<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

impl Font {
    pub const ROWS: usize = 4; 
    pub const COLS: usize = 32;

    pub fn from_bytes<B: IntoIterator<Item=u8>>(bytes: B, width: usize, height: usize) -> Self {
        let font_width = width * Self::COLS;
        let font_height = height * Self::ROWS;
        let mut bytes = bytes.into_iter(); 
        let mut bitmap = vec![vec![0; font_width]; font_height];
        for y in 0..font_height {
            for x in 0..font_width {
                let Some(byte) = bytes.next() else {
                    panic!("EOF at col: {x}, row: {y}");
                };
                bitmap[y][x] = byte;
                // eprint!("{byte:02x}")
            }
            // eprintln!();
        }
        Font {
            bitmap,
            width,
            height,
        }
    }

    pub fn get_pixel(&self, char_index: usize, x: usize, y: usize) -> u8 {
        let char_y = (char_index / Self::COLS) * self.height;
        let char_x = (char_index % Self::COLS) * self.width;
        // eprintln!("CI: {char_index}, cy: {char_y}, cx: {char_x}, y: {y}, x: {x}");
        self.bitmap[char_y + y][char_x + x]
    }

    pub(crate) fn char_index(&self, c: char) -> usize {
        let mut byte_chars = [0u8; 4];
        match c {
            '█' => {0x1f},
            '▓' => {0x1e},
            '▒' => {0x1d},
            '░' => {0x1c},
            '▄' => {0x1b},
            '▀' => {0x1a},
            c if c.is_ascii() => {
                c.encode_utf8(&mut byte_chars);
                byte_chars[0] as usize
            },
            _ => {
                c.encode_utf8(&mut byte_chars);
                eprintln!("Byte chars: {:?}", byte_chars);
                0
            }
        }
    }
}