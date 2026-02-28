use anyhow::Result;
use tracing::info;

use crate::config::Config;

// Simple 5x7 bitmap font for ASCII characters
const FONT_WIDTH: usize = 5;
const FONT_HEIGHT: usize = 7;

// Bitmap font data (1 = pixel on, 0 = pixel off)
// Basic ASCII characters from space (32) to ~ (126)
const FONT_DATA: &[u8] = &[
    // Space (32)
    0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
    // ! (33)
    0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
    // " (34)
    0b01010, 0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000,
    // # (35)
    0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010,
    // $ (36)
    0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
    // % (37)
    0b11000, 0b11001, 0b00010, 0b00100, 0b01000, 0b10011, 0b00011,
    // & (38)
    0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
    // ' (39)
    0b00100, 0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000,
    // ( (40)
    0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
    // ) (41)
    0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
    // * (42)
    0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000,
    // + (43)
    0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
    // , (44)
    0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
    // - (45)
    0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
    // . (46)
    0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
    // / (47)
    0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
    // 0 (48)
    0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
    // 1 (49)
    0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
    // 2 (50)
    0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
    // 3 (51)
    0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
    // 4 (52)
    0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
    // 5 (53)
    0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
    // 6 (54)
    0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
    // 7 (55)
    0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
    // 8 (56)
    0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
    // 9 (57)
    0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
    // : (58)
    0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
    // ; (59)
    0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b00100, 0b01000,
    // < (60)
    0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
    // = (61)
    0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
    // > (62)
    0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
    // ? (63)
    0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
    // @ (64)
    0b01110, 0b10001, 0b10001, 0b10111, 0b10101, 0b10111, 0b01000,
    // A (65)
    0b00100, 0b01010, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001,
    // B (66)
    0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
    // C (67)
    0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
    // D (68)
    0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
    // E (69)
    0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
    // F (70)
    0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
    // G (71)
    0b01110, 0b10001, 0b10000, 0b10000, 0b10011, 0b10001, 0b01110,
    // H (72)
    0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
    // I (73)
    0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
    // J (74)
    0b00001, 0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b01110,
    // K (75)
    0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
    // L (76)
    0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
    // M (77)
    0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
    // N (78)
    0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
    // O (79)
    0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
    // P (80)
    0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
    // Q (81)
    0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
    // R (82)
    0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
    // S (83)
    0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
    // T (84)
    0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
    // U (85)
    0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
    // V (86)
    0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
    // W (87)
    0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
    // X (88)
    0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
    // Y (89)
    0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
    // Z (90)
    0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
    // [ (91)
    0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
    // \ (92)
    0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000,
    // ] (93)
    0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
    // ^ (94)
    0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
    // _ (95)
    0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
    // ` (96)
    0b01000, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
    // a (97)
    0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111,
    // b (98)
    0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110,
    // c (99)
    0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110,
    // d (100)
    0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111,
    // e (101)
    0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110,
    // f (102)
    0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000,
    // g (103)
    0b00000, 0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001,
    // h (104)
    0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
    // i (105)
    0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110,
    // j (106)
    0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100,
    // k (107)
    0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
    // l (108)
    0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
    // m (109)
    0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001,
    // n (110)
    0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
    // o (111)
    0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110,
    // p (112)
    0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000,
    // q (113)
    0b00000, 0b00000, 0b01101, 0b10011, 0b01111, 0b00001, 0b00001,
    // r (114)
    0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000,
    // s (115)
    0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110,
    // t (116)
    0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110,
    // u (117)
    0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101,
    // v (118)
    0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
    // w (119)
    0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010,
    // x (120)
    0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001,
    // y (121)
    0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
    // z (122)
    0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111,
    // { (123)
    0b00110, 0b01000, 0b01000, 0b10000, 0b01000, 0b01000, 0b00110,
    // | (124)
    0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
    // } (125)
    0b01100, 0b00010, 0b00010, 0b00001, 0b00010, 0b00010, 0b01100,
    // ~ (126)
    0b00000, 0b00000, 0b00000, 0b01101, 0b10010, 0b00000, 0b00000,
];

pub struct Renderer {
    width: u32,
    height: u32,
    buffer: Vec<u8>, // RGBA buffer
    config: Config,
}

impl Renderer {
    pub fn new(width: u32, height: u32, config: &Config) -> Result<Self> {
        info!("Initializing renderer: {}x{}", width, height);

        // Initialize framebuffer buffer (RGBA)
        let buffer_size = (width * height * 4) as usize;
        let buffer = vec![0u8; buffer_size];

        Ok(Self {
            width,
            height,
            buffer,
            config: config.clone(),
        })
    }

    pub fn render(&mut self) -> Result<()> {
        // Clear background
        self.clear(self.config.background_color);

        // Draw some test content
        let text = "Hello, Bob!";
        let x = 50;
        let y = 100;
        self.draw_text(text, x, y, self.config.font_size as u32, self.config.text_color);

        // Draw a simple rectangle
        self.draw_rect(50, 150, 200, 50, [100, 150, 200, 255]);

        // Draw status text
        let status_text = "System running...";
        self.draw_text(
            status_text,
            50,
            250,
            (self.config.font_size * 0.8) as u32,
            [200, 200, 200, 255],
        );

        Ok(())
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        for pixel in self.buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, size: u32, color: [u8; 4]) {
        let scale = size as f32 / FONT_HEIGHT as f32;
        let char_spacing = (scale * 1.2) as i32; // Add some spacing between characters

        let mut current_x = x;

        for c in text.chars() {
            if c.is_ascii() && c as u8 >= 32 && c as u8 <= 126 {
                let char_idx = (c as u8 - 32) as usize;
                let char_offset = char_idx * FONT_HEIGHT;

                // Draw each row of the character
                for row in 0..FONT_HEIGHT {
                    let row_data = FONT_DATA[char_offset + row];

                    // Draw each column
                    for col in 0..FONT_WIDTH {
                        if (row_data >> (FONT_WIDTH - 1 - col)) & 1 == 1 {
                            let px = current_x + (col as f32 * scale) as i32;
                            let py = y + (row as f32 * scale) as i32;

                            // Scale the pixel
                            for sy in 0..scale as i32 {
                                for sx in 0..scale as i32 {
                                    self.set_pixel(
                                        (px + sx) as u32,
                                        (py + sy) as u32,
                                        color,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            current_x += char_spacing;
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) {
        for row in y..(y + height as i32) {
            for col in x..(x + width as i32) {
                if col >= 0 && col < self.width as i32 && row >= 0 && row < self.height as i32 {
                    self.set_pixel(col as u32, row as u32, color);
                }
            }
        }
    }

    pub fn draw_image(&mut self, x: i32, y: i32, image_data: &[u8], img_width: u32, img_height: u32) {
        // Simple image blitting (assumes RGBA format)
        for row in 0..img_height {
            for col in 0..img_width {
                let src_idx = ((row * img_width + col) * 4) as usize;
                if src_idx + 3 < image_data.len() {
                    let dst_x = x + col as i32;
                    let dst_y = y + row as i32;

                    if dst_x >= 0 && dst_x < self.width as i32 && dst_y >= 0 && dst_y < self.height as i32 {
                        let color = [
                            image_data[src_idx],
                            image_data[src_idx + 1],
                            image_data[src_idx + 2],
                            image_data[src_idx + 3],
                        ];
                        self.set_pixel(dst_x as u32, dst_y as u32, color);
                    }
                }
            }
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            self.buffer[idx..idx + 4].copy_from_slice(&color);
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}