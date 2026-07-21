use super::{
    Color24,
    SoftwareBuffer,
    ex02_winding_number_triangle::PixelDrawingCommand
};

impl Color24 {
    pub fn invert(self) -> Color24 {
        Color24 { r: 255 - self.r, g: 255 - self.g, b: 255 - self.b }
    }
    pub fn lerp(self, other: Color24, t: f32) -> Color24 {
        Color24 {
            r: (self.r as f32 * (1.0 - t) + other.r as f32 * t).round().clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * (1.0 - t) + other.g as f32 * t).round().clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * (1.0 - t) + other.b as f32 * t).round().clamp(0.0, 255.0) as u8,
        }
    }
}

pub struct InvertPixelCommand;
impl PixelDrawingCommand for InvertPixelCommand {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16) {
        let Some(color) = software_buffer.get_pixel(x, y) else { return };
        software_buffer.set_pixel(x, y, color.invert())
    }
}