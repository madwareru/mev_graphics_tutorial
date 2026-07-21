use super::{
    ex08_drawing_simple_unlit_3d_model::{PixelShader, VertexShader, VertexShaderData},
    Color24,
    SoftwareBuffer
};

pub struct DrawNormalsShader {
    pub model_matrix: glam::Mat4,
    pub view_matrix: glam::Mat4,
    pub proj_matrix: glam::Mat4,
}
impl VertexShader for DrawNormalsShader {
    type Output = VertexShaderData;
    fn transform_vertices(&self, input: VertexShaderData) -> VertexShaderData {
        let position = (self.proj_matrix * self.view_matrix * self.model_matrix) * input.position;
        let normal = self.model_matrix * input.normal;
        VertexShaderData { position, normal, ..input }
    }
}
impl PixelShader for DrawNormalsShader {
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

        let r = ((vertex_input.normal.x + 1.0) / 2.0 * 255.0).round().clamp(0.0, 255.0) as u8;
        let g = ((vertex_input.normal.y + 1.0) / 2.0 * 255.0).round().clamp(0.0, 255.0) as u8;
        let b = ((vertex_input.normal.z + 1.0) / 2.0 * 255.0).round().clamp(0.0, 255.0) as u8;

        let color = Color24 { r, g, b };

        software_buffer.set_pixel(fragment_x, fragment_y, color);
    }
}