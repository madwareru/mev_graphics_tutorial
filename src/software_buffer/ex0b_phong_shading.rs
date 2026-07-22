use crate::software_buffer::{Color24, SoftwareBuffer};
use crate::software_buffer::ex08_drawing_simple_unlit_3d_model::{PixelShader, VertexShader, VertexShaderData};

pub struct DrawPhongShadedModelShader<'a> {
    pub model_matrix: glam::Mat4,
    pub view_matrix: glam::Mat4,
    pub proj_matrix: glam::Mat4,
    pub light_direction: glam::Vec4,
    pub light_color: glam::Vec3,
    pub ambient_color: glam::Vec3,
    pub texture: &'a [Color24],
    pub texture_width: u16,
    pub texture_height: u16,
}

impl<'a> VertexShader for DrawPhongShadedModelShader<'a> {
    type Output = VertexShaderData;
    fn transform_vertices(&self, input: VertexShaderData) -> VertexShaderData {
        let position = (self.proj_matrix * self.view_matrix * self.model_matrix) * input.position;
        let normal = (self.model_matrix * input.normal).normalize_or_zero();
        VertexShaderData { position, normal, ..input }
    }
}

impl <'a> PixelShader for DrawPhongShadedModelShader<'a> {
    type Input = VertexShaderData;

    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        depth_texture: &mut [f32],
        vertex_input: VertexShaderData,
        fragment_x: u16,
        fragment_y: u16
    ) {
        let fragment_index = (fragment_y as usize) * software_buffer.get_width() as usize + (fragment_x as usize);
        if depth_texture[fragment_index] > vertex_input.position.z { return; }
        depth_texture[fragment_index] = vertex_input.position.z;

        let normal = vertex_input.normal;
        let normal = if normal.length_squared() > 1.0 { normal.normalize_or_zero() } else { normal };
        let reversed_light_dir = (-self.light_direction).normalize_or_zero();
        let attenuation = normal.dot(reversed_light_dir).max(0.0);
        let light_color = (self.ambient_color + self.light_color * attenuation).clamp(glam::Vec3::ZERO, glam::Vec3::ONE);

        let (u, v) = (vertex_input.tex_coord.x, vertex_input.tex_coord.y);
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
        let mut color = color0.lerp(color1, v);

        color.r = (color.r as f32 * light_color.x).clamp(0.0, 255.0) as u8;
        color.g = (color.g as f32 * light_color.y).clamp(0.0, 255.0) as u8;
        color.b = (color.b as f32 * light_color.z).clamp(0.0, 255.0) as u8;

        software_buffer.set_pixel(fragment_x, fragment_y, color);
    }
}