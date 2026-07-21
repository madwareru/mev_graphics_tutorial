use crate::geometry::ex04_barycentric_coordinates::mix_2_components_by_barycentric;
use crate::geometry::Point;
use super::{SoftwareBuffer, ex02_winding_number_triangle::PixelDrawingCommand, Color24};

pub struct DrawUVTriangleCommand<'a> {
    pub positions: &'a [Point],
    pub uv_coords: &'a [(f32, f32)],
    pub indices: [u16; 3],
}
impl<'a> PixelDrawingCommand for DrawUVTriangleCommand<'a> {
    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        x: u16,
        y: u16
    ) {
        let indices = self.indices.map(|it| it as usize);

        assert_eq!(self.positions.len(), self.uv_coords.len());
        assert!((indices[0]) < self.positions.len());
        assert!((indices[1]) < self.positions.len());
        assert!((indices[2]) < self.positions.len());

        let positions = indices.map(|id| self.positions[id] );
        let uv_coords = indices.map(|id| self.uv_coords[id] );

        let point = Point { x: x as _, y: y as _ };
        let barycentric_coords = point.calculate_barycentric_in(positions);
        let (u, v) = mix_2_components_by_barycentric(uv_coords, barycentric_coords);

        let color = Color24 {
            r: (u * 255.0).round().clamp(0.0, 255.0) as u8,
            g: (v * 255.0).round().clamp(0.0, 255.0) as u8,
            b: 0
        };

        software_buffer.set_pixel(x, y, color);
    }
}