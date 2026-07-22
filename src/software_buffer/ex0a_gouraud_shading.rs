use super::{
    Color24,
    SoftwareBuffer,
    ex08_drawing_simple_unlit_3d_model::{
        InterpolatedByBarycentric,
        PixelShader,
        VertexShader,
        VertexShaderData
    }
};

#[derive(Copy, Clone)]
pub struct GouraudSharedShaderData {
    pub position: glam::Vec4,
    pub tex_coord: glam::Vec2,
    pub color: glam::Vec3,
}

impl InterpolatedByBarycentric for GouraudSharedShaderData {
    fn interpolate_by_barycentric(vertices: [Self; 3], barycentric_coords: [f32; 3]) -> Self {
        barycentric_coords.iter().copied()
            .zip(vertices.iter().copied()).fold(
            GouraudSharedShaderData {
                position: glam::Vec4::ZERO,
                tex_coord: glam::Vec2::ZERO,
                color: glam::Vec3::ZERO,
            },
            |acc, (mul, next)| GouraudSharedShaderData {
                position: acc.position + mul * next.position,
                tex_coord: acc.tex_coord + mul * next.tex_coord,
                color: acc.color + mul * next.color,
            }
        )
    }

    fn get_position(&self) -> glam::Vec4 {
        self.position
    }
}

pub struct DrawGouraudShadedModelShader<'a> {
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
impl<'a> VertexShader for DrawGouraudShadedModelShader<'a> {
    type Output = GouraudSharedShaderData;
    fn transform_vertices(&self, input: VertexShaderData) -> GouraudSharedShaderData {
        let normal = (self.model_matrix * input.normal).normalize_or_zero();
        let position = (self.proj_matrix * self.view_matrix * self.model_matrix) * input.position;
        let reversed_light_dir = (-self.light_direction).normalize_or_zero();
        let attenuation = normal.dot(reversed_light_dir).max(0.0);
        let color = (self.ambient_color + self.light_color * attenuation).clamp(glam::Vec3::ZERO, glam::Vec3::ONE);
        GouraudSharedShaderData { position, tex_coord: input.tex_coord, color }
    }
}
impl<'a> PixelShader for DrawGouraudShadedModelShader<'a> {
    type Input = GouraudSharedShaderData;

    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        depth_texture: &mut [f32],
        vertex_input: GouraudSharedShaderData,
        fragment_x: u16,
        fragment_y: u16
    ) {
        let fragment_index = (fragment_y as usize) * software_buffer.get_width() as usize + (fragment_x as usize);
        if depth_texture[fragment_index] > vertex_input.position.z { return; }
        depth_texture[fragment_index] = vertex_input.position.z;

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

        color.r = (color.r as f32 * vertex_input.color.x).clamp(0.0, 255.0) as u8;
        color.g = (color.g as f32 * vertex_input.color.y).clamp(0.0, 255.0) as u8;
        color.b = (color.b as f32 * vertex_input.color.z).clamp(0.0, 255.0) as u8;

        software_buffer.set_pixel(fragment_x, fragment_y, color);
    }
}