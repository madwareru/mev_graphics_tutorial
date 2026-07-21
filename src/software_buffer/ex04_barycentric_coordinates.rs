use crate::geometry::{Point};
use crate::geometry::ex04_barycentric_coordinates::mix_3_components_by_barycentric;
use super::{Color24, SoftwareBuffer, ex02_winding_number_triangle::PixelDrawingCommand};

pub struct DrawBarycentricTriangleCommand(pub [Point; 3]);
impl PixelDrawingCommand for DrawBarycentricTriangleCommand {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16) {
        let point = Point { x: x as _, y: y as _ };
        let barycentric_coords = point.calculate_barycentric_in(self.0);
        let colors = [
            Color24 { r: 255, g: 0, b: 0 },
            Color24 { r: 0, g: 255, b: 0 },
            Color24 { r: 0, g: 0, b: 255 }
        ];
        let colors = colors.map(|color| (color.r as f32, color.g as f32, color.b as f32));
        let (r, g, b) = mix_3_components_by_barycentric(colors, barycentric_coords);
        let color = Color24 {
            r: r.round().clamp(0.0, 255.0) as u8,
            g: g.round().clamp(0.0, 255.0) as u8,
            b: b.round().clamp(0.0, 255.0) as u8
        };
        software_buffer.set_pixel(x, y, color);
    }
}