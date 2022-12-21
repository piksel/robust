const WIDTH: usize = 512;
const HEIGHT: usize = 480;
const CHAR_HEIGHT: usize = 18;
const CHAR_WIDTH: usize = 10;
const CHAR_ROWS: usize = 4; 
const CHAR_COLS: usize = 32;
const FONT_WIDTH: usize = CHAR_WIDTH * CHAR_COLS;
const FONT_HEIGHT: usize = CHAR_HEIGHT * CHAR_ROWS;

pub struct Font {
    bitmap: [[u8; FONT_WIDTH]; FONT_HEIGHT]
}

impl Font {
    pub fn from_bytes<B: IntoIterator<Item=u8>>(bytes: B) -> Self {
        let mut bytes = bytes.into_iter(); 
        let mut bitmap = [[0; FONT_WIDTH]; FONT_HEIGHT];
        for y in 0..FONT_HEIGHT {
            for x in 0..FONT_WIDTH {
                let Some(byte) = bytes.next() else {
                    panic!("EOF at col: {x}, row: {y}");
                };
                bitmap[y][x] = byte;
                // eprint!("{byte:02x}")
            }
            // eprintln!();
        }
        Font {
            bitmap
        }
    }
    
    pub fn draw_text(&self, buffer: &mut Vec<u32>, x: usize, y: usize, text: &str) -> anyhow::Result<()> {



        let mut byte_chars = [0u8; 4];
        for (ci, c) in text.chars().enumerate() {
            let font_offset = if c.is_ascii() {
                c.encode_utf8(&mut byte_chars);
                byte_chars[0]
            } else {
                0
            };
            

            let char_y = (font_offset as usize / 0x20) * CHAR_HEIGHT;
            let char_x = (font_offset as usize % 0x20) * CHAR_WIDTH;
            let char_x_offset = ci * CHAR_WIDTH;

            for cy in 0..CHAR_HEIGHT {
                for cx in 0..CHAR_WIDTH {
                    let mono = self.bitmap[char_y + cy][char_x + cx] as u32;
                    let by = y + cy;
                    let bx = x + cx + char_x_offset;

                    let color = mono << 16 | mono << 8 | mono;
                    let offset = (by * WIDTH) + bx;

                    if mono != 0 {
                        buffer[offset] = color;
                    }
                }
            }
        }

        Ok(())
    }
}