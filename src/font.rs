use anyhow::bail;

pub struct Font {
    bitmap: Vec<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

impl Font {
    pub const ROWS: usize = 4; 
    pub const COLS: usize = 32;

    pub fn try_from_bytes<B: IntoIterator<Item=u8>>(bytes: B) -> anyhow::Result<Self> {
        
        let mut bytes = bytes.into_iter();
        let header = BitmapFontHeader::try_from_iter(&mut bytes)?;
        if header.version != 1 {bail!("Unsupported BMF version {}", header.version)}
        if header.version != 1 {bail!("Unsupported BMF version {}", header.version)}
        if header.color_mode != ColorMode::Outlined {bail!("Unsupported BMF color mode {:?}", header.color_mode)}
        if header.outline != 1 {bail!("Unsupported BMF outline size {}", header.outline)}

        let width = header.width as usize;
        let height = header.height as usize;
        assert_eq!(width, 10);
        assert_eq!(height, 18);
        let font_width = width * Self::COLS;
        let font_height = height * Self::ROWS;

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
        Ok(Font {
            bitmap,
            width,
            height,
        })
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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ColorMode {
    Mono = 0x00,
    Outlined = 0x01,
}

pub struct BitmapFontHeader {
    pub version: u8,
    pub width: u8,
    pub height: u8,
    pub outline: u8,
    pub color_mode: ColorMode,
}

impl BitmapFontHeader {
    pub const MAGIC: [u8; 3] = [
        'B' as u8,
        'M' as u8,
        'F' as u8,
    ];

    pub fn try_from_iter(bytes: &mut impl Iterator<Item = u8>) -> anyhow::Result<Self> {

        match (bytes.next(), bytes.next(), bytes.next()) {
            (Some(a), Some(b), Some(c)) if [a, b, c] == Self::MAGIC => {
                Ok(Self {
                    version: bytes.next().ok_or_else(|| anyhow::format_err!("Version missing"))?,
                    width: bytes.next().ok_or_else(|| anyhow::format_err!("Width missing"))?,
                    height: bytes.next().ok_or_else(|| anyhow::format_err!("Height missing"))?,
                    outline: bytes.next().ok_or_else(|| anyhow::format_err!("Outline missing"))?,
                    color_mode: bytes.next().ok_or_else(|| anyhow::format_err!("Color mode missing"))?.try_into()?,
                })
            },
            (Some(a), Some(b), Some(c)) => anyhow::bail!("Invalid BMF header magic bytes {a:02x} {b:02x} {c:02x}"),
            _ =>  anyhow::bail!("Input font file too small"),
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        [
            Self::MAGIC[0],
            Self::MAGIC[1],
            Self::MAGIC[2],
            self.version,
            self.width,
            self.height,
            self.outline,
            self.color_mode as u8,
        ]
    }
}

impl IntoIterator for BitmapFontHeader {
    type Item = u8;
    type IntoIter = std::array::IntoIter<u8, 8>;

    fn into_iter(self) -> Self::IntoIter {
        let it = self.to_bytes().into_iter();

        it
    }
}

impl TryFrom<u8> for ColorMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ColorMode::Mono),
            1 => Ok(ColorMode::Outlined),
            n => bail!("Unsupported color mode {n}")
        }
    }
}