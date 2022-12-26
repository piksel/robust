use crate::font::Font;

pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl Screen {
    pub fn new(width: usize, height: usize, base_color: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![base_color; width * height]
        }
    }

    pub fn draw_text(&mut self, font: &Font, x: usize, y: usize, text: &str, color: u8) -> anyhow::Result<()> {
        // let mut byte_chars = [0u8; 4];
        for (li, line) in text.lines().enumerate() {

            
            for (ci, c) in line.chars().enumerate() {
                
                let font_offset = font.char_index(c);

                let char_x_offset = ci * font.width;
                let char_y_offset = li * font.height;

                for cy in 0..font.height {
                    for cx in 0..font.width {
                        let mono = font.get_pixel(font_offset as usize, cx, cy) as u32;
                        let by = y + cy + char_y_offset;
                        let bx = x + cx + char_x_offset;

                        let pixel = mono << 16 | mono << 8 | mono;

                        if pixel != 0 {
                            self.set_pixel(by, bx, match color {
                                0 => pixel,
                                1 => 0xffffffff ^ pixel,
                                _ => pixel,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn set_pixel(&mut self, by: usize, bx: usize, pixel: u32) {
        if bx >= self.width || by >= self.height {return;}
        // assert!(by < self.height, "{by} is larger than screen height {}", self.height);
        // assert!(bx < self.width, "{bx} is larger than screen width {}", self.width);
        let offset = (by * self.width) + bx;
        self.buffer[offset] = pixel;
    }
}