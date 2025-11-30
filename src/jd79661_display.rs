use embedded_graphics::{pixelcolor::raw::RawU2, prelude::*, primitives::Rectangle};

use crate::{
    jd79661::{HEIGHT, PIXDEPTH, WIDTH},
    theme::Theme,
};

/// embedded_graphics support for the JD79661

#[derive(PartialEq, Clone, Copy)]
#[repr(u8)]
#[derive(Default)]
pub enum JD79661Color {
    Black = 0b00,
    #[default]
    White = 0b01,
    Yellow = 0b10,
    Red = 0b11,
}

impl PixelColor for JD79661Color {
    type Raw = RawU2;
}

const BUFFER_LENGTH: usize = WIDTH * HEIGHT * PIXDEPTH / 8;

pub struct JD79661Display {
    buffer: [u8; BUFFER_LENGTH],
}

impl JD79661Display {
    pub fn buffer(&self) -> &[u8; BUFFER_LENGTH] {
        &self.buffer
    }
}

impl Default for JD79661Display {
    fn default() -> Self {
        let color = JD79661Color::default() as u8;
        let byte = color | (color << 2) | (color << 4) | (color << 6);
        Self {
            buffer: [byte; BUFFER_LENGTH],
        }
    }
}

impl Dimensions for JD79661Display {
    fn bounding_box(&self) -> Rectangle {
        Rectangle {
            top_left: Point { x: 0, y: 0 },
            size: Size {
                width: WIDTH as u32,
                height: HEIGHT as u32,
            },
        }
    }
}

impl DrawTarget for JD79661Display {
    type Color = JD79661Color;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(point, color) = pixel;

            // XXX There is normally a 6px margin to the right which is not
            // shown. Unsure why.
            let point = Point {
                x: point.x - 3,
                y: point.y,
            };

            if !self.bounding_box().contains(point) {
                continue;
            }

            let x = point.x as usize;
            let y = point.y as usize;

            let byte_index = y * WIDTH * PIXDEPTH / 8 + x * PIXDEPTH / 8;
            let mut byte = self.buffer[byte_index];

            let pixel_index = (x % 4) as u8;

            let mask = 0b11 << (8 - (pixel_index + 1) * 2);
            let shifted = (color as u8) << (8 - (pixel_index + 1) * 2);

            byte |= shifted;
            byte &= !mask | shifted;

            self.buffer[byte_index] = byte;
        }
        Ok(())
    }
}

pub struct JD79661Theme;

impl JD79661Theme {
    pub fn new() -> Self {
        Self {}
    }
}

impl Theme for JD79661Theme {
    type Color = JD79661Color;

    fn background(&self) -> Self::Color {
        Self::Color::Black
    }

    fn text(&self) -> Self::Color {
        Self::Color::White
    }
}
