use crate::geometry::ex04_barycentric_coordinates::mix_2_components_by_barycentric;
use crate::geometry::Point;
use crate::software_buffer::{Color24, SoftwareBuffer};
use crate::software_buffer::ex02_winding_number_triangle::PixelDrawingCommand;

pub struct DrawTextureMappedTriangleBilinearCommand<'a> {
    pub positions: &'a [Point],
    pub uv_coords: &'a [(f32, f32)],
    pub indices: [u16; 3],
    pub texture: &'a [Color24],
    pub texture_width: u16,
    pub texture_height: u16,
}
impl <'a> PixelDrawingCommand for DrawTextureMappedTriangleBilinearCommand<'a> {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16) {
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
        // Invert v to get the correct orientation of the texture
        let v = 1.0 - v;

        let texture_x = u * (self.texture_width - 1) as f32;
        let texture_y = v * (self.texture_height - 1) as f32;

        let u = texture_x.fract();
        let v = texture_y.fract();

        let x0 = (texture_x.trunc() as u16).clamp(0, self.texture_width - 1);
        let y0 = (texture_y.trunc() as u16).clamp(0, self.texture_height - 1);
        let x1 = (x0 + 1).clamp(0, self.texture_width - 1);
        let y1 = (y0 + 1).clamp(0, self.texture_height - 1);

        let id0 = y0 as usize * self.texture_width as usize + x0 as usize;
        let id1 = y0 as usize * self.texture_width as usize + x1 as usize;
        let id2 = y1 as usize * self.texture_width as usize + x0 as usize;
        let id3 = y1 as usize * self.texture_width as usize + x1 as usize;

        let color0 = self.texture[id0].lerp(self.texture[id1], u);
        let color1 = self.texture[id2].lerp(self.texture[id3], u);

        software_buffer.set_pixel(x, y, color0.lerp(color1, v));
    }
}