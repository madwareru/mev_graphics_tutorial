use crate::geometry::{Triangle};
use super::{Color24, SoftwareBuffer};

/// A trait for a command that draws a pixel on the buffer.
pub trait PixelDrawingCommand {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16);
}

/// A command that fills the pixel with the given color.
pub struct FillPixelCommand(pub Color24);
impl PixelDrawingCommand for FillPixelCommand {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16) {
        software_buffer.set_pixel(x, y, self.0)
    }
}
impl SoftwareBuffer {
    /// Draws a triangle on the buffer using the given command.
    /// Uses a winding number to determine if the pixel is inside
    /// a triangle.
    pub fn draw_triangle(
        &mut self,
        triangle: Triangle,
        command: &impl PixelDrawingCommand
    ) {
        let min_x = triangle.a.x.min(triangle.b.x).min(triangle.c.x).max(0);
        let max_x = triangle.a.x.max(triangle.b.x).max(triangle.c.x);
        let min_y = triangle.a.y.min(triangle.b.y).min(triangle.c.y).max(0);
        let max_y = triangle.a.y.max(triangle.b.y).max(triangle.c.y);
        for y in min_y..=max_y {
            for x in (min_x..=max_x).filter(|&x| triangle.winding_n(min_x..=x, y) % 2 != 0) {
                command.draw_pixel(self, x as _, y as _);
            }
        }
    }
}