use std::sync::Arc;

use crate::api::DrawApi;
use gamercade_core::{Color, GraphicsParameters, PixelBuffer, Rom, BYTES_PER_PIXEL};

#[derive(Clone)]
pub struct DrawContext {
    pub(crate) frame_buffer: PixelBuffer,
    pub(crate) rom: Arc<Rom>,
}

impl DrawContext {
    pub fn new(rom: Arc<Rom>) -> Self {
        Self {
            frame_buffer: PixelBuffer::init_from_rom(&rom),
            rom,
        }
    }
}

impl DrawApi for DrawContext {
    fn sprite(&mut self, graphics_parameters: i32, transparency_mask: i64, x: i32, y: i32) {
        let GraphicsParameters {
            palette_index,
            sprite_sheet_index,
            sprite_index,
            ..
        } = graphics_parameters.into();

        let palette = match self.rom.graphics.palette(palette_index) {
            Some(palette) => palette,
            None => return,
        };
        let sheet = match self.rom.graphics.sprite_sheet(sprite_sheet_index) {
            Some(sheet) => sheet,
            None => return,
        };

        self.frame_buffer
            .draw_sprite(sheet, sprite_index, palette, x, y, transparency_mask);
    }

    fn clear_screen(&mut self, graphics_parameters: i32) {
        let GraphicsParameters {
            color_index,
            palette_index,
            ..
        } = graphics_parameters.into();

        self.rom
            .clear_buffer(color_index, palette_index, &mut self.frame_buffer);
    }

    fn set_pixel(&mut self, graphics_parameters: i32, x: i32, y: i32) {
        let GraphicsParameters {
            color_index,
            palette_index,
            ..
        } = graphics_parameters.into();

        if let (Ok(x), Ok(y)) = (self.validate_x(x), self.validate_y(y)) {
            if let Some(palette) = self.rom.graphics.palette(palette_index) {
                let color = palette[color_index];
                self.set_pixel_safe(x, y, color)
            }
        }
    }

    fn line(&mut self, graphics_parameters: i32, x0: i32, y0: i32, x1: i32, y1: i32) {
        let GraphicsParameters {
            color_index,
            palette_index,
            ..
        } = graphics_parameters.into();

        let palette = match self.rom.graphics.palette(palette_index) {
            Some(palette) => palette,
            None => return,
        };

        let color = palette[color_index];

        // Optimized horizontal or veritcal lines
        if x0 == x1 {
            self.draw_line_vertical(x0, y0, y1, color);
            return;
        } else if y0 == y1 {
            self.draw_line_horizontal(x0, x1, y0, color);
            return;
        }

        if (y1 - y0).abs() < (x1 - x0).abs() {
            if x0 > x1 {
                self.draw_line_low(x1, y1, x0, y0, color);
            } else {
                self.draw_line_low(x0, y0, x1, y1, color);
            }
        } else if y0 > y1 {
            self.draw_line_high(x1, y1, x0, y0, color);
        } else {
            self.draw_line_high(x0, y0, x1, y1, color);
        }
    }

    fn rect(&mut self, graphics_parameters: i32, x: i32, y: i32, width: i32, height: i32) {
        let GraphicsParameters {
            color_index,
            palette_index,
            ..
        } = graphics_parameters.into();

        let color = match self.rom.graphics.palette(palette_index) {
            Some(palette) => palette[color_index],
            None => return,
        };

        let x1 = x + width;
        let y1 = y + height;

        // Top
        self.draw_line_horizontal(x, x1, y, color);

        // Bottom
        self.draw_line_horizontal(x, x1, y1, color);

        // Left
        self.draw_line_vertical(x, y, y1, color);

        // Right
        self.draw_line_vertical(x1, y, y1, color);
    }

    fn rect_filled(&mut self, graphics_parameters: i32, x: i32, y: i32, width: i32, height: i32) {
        let GraphicsParameters {
            color_index,
            palette_index,
            ..
        } = graphics_parameters.into();

        let color = match self.rom.graphics.palette(palette_index) {
            Some(palette) => palette[color_index],
            None => return,
        };

        let x1 = x + width;
        let y1 = y + height;

        (y..y1).for_each(|y| {
            self.draw_line_horizontal(x, x1, y, color);
        })
    }

    fn circle(&mut self, graphics_parameters: i32, x: i32, y: i32, radius: i32) {
        let top = self.validate_y(y - radius);
        let bottom = self.validate_y(y + radius);
        let left = self.validate_x(x - radius);
        let right = self.validate_x(x + radius);

        let GraphicsParameters {
            color_index,
            palette_index,
            ..
        } = graphics_parameters.into();

        let color = match self.rom.graphics.palette(palette_index) {
            Some(palette) => palette[color_index],
            None => return,
        };

        if top.is_err() || bottom.is_err() || left.is_err() || right.is_err() {
            return;
        }

        let x0 = x as usize;
        let y0 = y as usize;

        let mut f = 1 - radius;
        let mut ddf_x = 0;
        let mut ddf_y = -2 * radius;
        let mut x = 0;
        let mut y = radius;

        self.set_pixel_safe(XCord(x0), YCord(y0 + radius as usize), color);
        self.set_pixel_safe(XCord(x0), YCord(y0 - radius as usize), color);
        self.set_pixel_safe(XCord(x0 + radius as usize), YCord(y0 as usize), color);
        self.set_pixel_safe(XCord(x0 - radius as usize), YCord(y0 as usize), color);

        while x < y {
            if f >= 0 {
                y -= 1;
                ddf_y += 2;
                f += ddf_y;
            };

            x += 1;
            ddf_x += 2;
            f += ddf_x + 1;
            self.set_pixel_safe(XCord(x0 + x as usize), YCord(y0 + y as usize), color);
            self.set_pixel_safe(XCord(x0 - x as usize), YCord(y0 + y as usize), color);
            self.set_pixel_safe(XCord(x0 + x as usize), YCord(y0 - y as usize), color);
            self.set_pixel_safe(XCord(x0 - x as usize), YCord(y0 - y as usize), color);
            self.set_pixel_safe(XCord(x0 + y as usize), YCord(y0 + x as usize), color);
            self.set_pixel_safe(XCord(x0 - y as usize), YCord(y0 + x as usize), color);
            self.set_pixel_safe(XCord(x0 + y as usize), YCord(y0 - x as usize), color);
            self.set_pixel_safe(XCord(x0 - y as usize), YCord(y0 - x as usize), color);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct XCord(usize);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct YCord(usize);

impl DrawContext {
    fn width(&self) -> i32 {
        self.rom.width()
    }

    fn height(&self) -> i32 {
        self.rom.height()
    }

    fn validate_x(&self, x: i32) -> Result<XCord, &'static str> {
        if x >= 0 && x < self.width() {
            Ok(XCord(x as usize))
        } else {
            Err("invalid X screen coordinate")
        }
    }

    fn validate_y(&self, y: i32) -> Result<YCord, &'static str> {
        if y >= 0 && y < self.height() {
            Ok(YCord(y as usize))
        } else {
            Err("invalid Y screen coordinate")
        }
    }

    fn set_pixel_safe(&mut self, x: XCord, y: YCord, color: Color) {
        let pixel_index = self.x_y_cord_to_pixel_buffer_index(x, y);
        let color = color.into_pixel_data();
        self.frame_buffer[pixel_index..pixel_index + BYTES_PER_PIXEL].copy_from_slice(&color);
    }

    fn x_y_cord_to_pixel_buffer_index(&self, x: XCord, y: YCord) -> usize {
        (x.0 + (y.0 * self.width() as usize)) * BYTES_PER_PIXEL
    }

    // TODO: Handle out of bounds pixels
    fn draw_line_low(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;

        let y_adjust = if dy < 0 {
            dy = -dy;
            -1
        } else {
            1
        };

        let mut d = (2 * dy) - dx;
        let mut y = y0;

        for x in x0..=x1 {
            if let (Ok(valid_x), Ok(valid_y)) = (self.validate_x(x), self.validate_y(y)) {
                self.set_pixel_safe(valid_x, valid_y, color);
                if d > 0 {
                    y += y_adjust;
                    d += 2 * (dy - dx);
                } else {
                    d += 2 * dy;
                }
            } else {
                return;
            }
        }
    }

    // TODO: Handle out of bounds pixels
    fn draw_line_high(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let mut dx = x1 - x0;
        let dy = y1 - y0;

        let x_adjust = if dx < 0 {
            dx = -dx;
            -1
        } else {
            1
        };

        let mut d = (2 * dx) - dy;
        let mut x = x0;

        for y in y0..=y1 {
            if let (Ok(valid_x), Ok(valid_y)) = (self.validate_x(x), self.validate_y(y)) {
                self.set_pixel_safe(valid_x, valid_y, color);
                if d > 0 {
                    x += x_adjust;
                    d += 2 * (dx - dy);
                } else {
                    d += 2 * dx;
                }
            } else {
                return;
            }
        }
    }

    // TODO: Can optimize this further with direct access into
    // the pixel buffers?
    fn draw_line_vertical(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
        if x < 0 || x > self.width() - 1 {
            return;
        }

        let (start, end) = if y0 < y1 { (y0, y1) } else { (y1, y0) };

        if start > self.height() || end < 0 {
            return;
        }

        let start = start.max(0) as usize;
        let end = end.min(self.height() as i32) as usize;

        let width = self.width() as usize;
        let start_index = (start * width) + x as usize;
        let pixel_count = (end - start) + 1;
        let color = color.into_pixel_data();

        self.frame_buffer
            .pixel_buffer
            .chunks_exact_mut(BYTES_PER_PIXEL)
            .skip(start_index)
            .step_by(width)
            .take(pixel_count)
            .for_each(|pixel| pixel.copy_from_slice(&color));
    }

    fn draw_line_horizontal(&mut self, x0: i32, x1: i32, y: i32, color: Color) {
        if y < 0 || y > self.height() {
            return;
        }

        let (start, end) = if x0 < x1 { (x0, x1) } else { (x1, x0) };

        if start > self.width() - 1 || end < 0 {
            return;
        }

        let start = start.max(0) as usize;
        let end = end.min(self.width() as i32 - 1) as usize;
        let y = y as usize;

        let start_index = (y * self.width() as usize) + start;
        let pixel_count = (end - start) + 1;
        let color = color.into_pixel_data();

        self.frame_buffer
            .pixel_buffer
            .chunks_exact_mut(BYTES_PER_PIXEL)
            .skip(start_index)
            .take(pixel_count)
            .for_each(|pixel| pixel.copy_from_slice(&color));
    }
}
