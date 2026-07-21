use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24
    }
};

const MOON_RADIUS: u16 = 32;
const MOON_COLOR: Color24 = Color24 { r: 244, g: 244, b: 220 };
const SKY_COLOR: Color24 = Color24 { r: 17, g: 13, b: 22 };
const GROUND_COLOR: Color24 = Color24 { r: 112, g: 104, b: 170 };

const IMAGE_WIDTH: u16 = 640;
const IMAGE_HEIGHT: u16 = 480;

fn draw_moon(buffer: &mut SoftwareBuffer, x: i16, y: i16) {
    buffer.fill_circle(x, y, MOON_RADIUS, MOON_COLOR);
    buffer.fill_circle(x - 20, y + 10, MOON_RADIUS, SKY_COLOR);
}

/// This example demonstrates the using of simple drawing functions,
/// such as `fill_circle` and `fill_rectangle`. These functions are implemented
/// in the accompanying `ex01_basic_drawing` module.
/// 
/// We are creating the buffer of pixels, then it gets filled by the color of the sky.
/// Then a couple of filled circles are drawn, one with a color of the moon and the 
/// other with a color of the sky, making a moon-like shape. Then we fill the rectangle
/// on the bottom of the image with the color of the ground. Then it gets printed as a ppm 
/// image into the terminal.
pub fn main() {
    let mut buffer = SoftwareBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    buffer.clear(SKY_COLOR);

    draw_moon(&mut buffer, 488, 113);

    let ground_y = 297;
    let ground_height = IMAGE_HEIGHT - ground_y as u16;
    buffer.fill_rectangle(0, ground_y, IMAGE_WIDTH, ground_height, GROUND_COLOR);

    buffer.print_as_ppm();
}