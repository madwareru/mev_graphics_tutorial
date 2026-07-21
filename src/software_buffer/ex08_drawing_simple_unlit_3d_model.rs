use super::{Color24, SoftwareBuffer};
use crate::{
    geometry::{Point, Triangle},
    obj_loader::ObjModel,
};

#[derive(Copy, Clone)]
pub struct VertexShaderData {
    pub position: glam::Vec4,
    pub tex_coord: glam::Vec2,
    pub normal: glam::Vec4,
}

pub trait InterpolatedByBarycentric: Copy {
    fn interpolate_by_barycentric(items: [Self; 3], barycentric_coords: [f32; 3]) -> Self;
    fn get_position(&self) -> glam::Vec4;
}

impl InterpolatedByBarycentric for VertexShaderData {
    fn interpolate_by_barycentric(vertices: [Self; 3], barycentric_coords: [f32; 3]) -> Self {
        barycentric_coords.iter().copied()
            .zip(vertices.iter().copied()).fold(
            VertexShaderData {
                position: glam::Vec4::ZERO,
                tex_coord: glam::Vec2::ZERO,
                normal: glam::Vec4::ZERO,
            },
            |acc, (mul, next)| VertexShaderData {
                position: acc.position + mul * next.position,
                tex_coord: acc.tex_coord + mul * next.tex_coord,
                normal: acc.normal + mul * next.normal,
            }
        )
    }

    fn get_position(&self) -> glam::Vec4 {
        self.position
    }
}

pub trait VertexShader {
    type Output: InterpolatedByBarycentric;
    fn transform_vertices(&self, input: VertexShaderData) -> Self::Output;
}

pub trait PixelShader {
    type Input: InterpolatedByBarycentric;
    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        depth_texture: &mut [f32],
        vertex_input: Self::Input,
        fragment_x: u16,
        fragment_y: u16
    );
}
impl SoftwareBuffer {
    pub fn draw_obj_model<TData: InterpolatedByBarycentric, TShader: VertexShader<Output=TData> + PixelShader<Input=TData>>(
        &mut self,
        obj_model: &ObjModel,
        depth_texture: &mut [f32],
        shader: &TShader,
    ) {
        assert_eq!(depth_texture.len(), self.width as usize * self.height as usize);
        let center_x = self.get_width() as f32 / 2.0;
        let center_y = self.get_height() as f32 / 2.0;
        for tri in obj_model.indices() {
            // Vertex stage
            let vertices = tri.map(|id| {
                let position = obj_model.vs()[id as usize];
                let tex_coord = obj_model.vts()[id as usize];
                let normal = obj_model.vns()[id as usize];
                shader.transform_vertices(VertexShaderData { position, tex_coord, normal })
            });

            // Clipping stage (anything outside the screen is discarded)
            let triangle = Triangle::new(
                // Vertices are centered around the center of the screen
                Point {
                    x: (center_x  * (vertices[0].get_position().x + 1.0)).round() as i16,
                    y: (center_y  * (1.0 - vertices[0].get_position().y)).round() as i16,
                },
                Point {
                    x: (center_x  * (vertices[1].get_position().x + 1.0)).round() as i16,
                    y: (center_y  * (1.0 - vertices[1].get_position().y)).round() as i16,
                },
                Point {
                    x: (center_x  * (vertices[2].get_position().x + 1.0)).round() as i16,
                    y: (center_y  * (1.0 - vertices[2].get_position().y)).round() as i16,
                }
            );
            let min_x = triangle.a.x.min(triangle.b.x).min(triangle.c.x).max(0);
            let max_x = triangle.a.x.max(triangle.b.x).max(triangle.c.x).min((self.width - 1) as _);
            let min_y = triangle.a.y.min(triangle.b.y).min(triangle.c.y).max(0);
            let max_y = triangle.a.y.max(triangle.b.y).max(triangle.c.y).min((self.height - 1) as _);

            // Pixel stage
            for y in min_y..=max_y {
                for x in (min_x..=max_x).filter(|&x| triangle.winding_n(min_x..=x, y) % 2 != 0) {
                    let barycentric_coords = Point { x, y }.calculate_barycentric_in(
                        [triangle.a, triangle.b, triangle.c]
                    );
                    let vertex_input = TData::interpolate_by_barycentric(vertices, barycentric_coords);
                    shader.draw_pixel(self, depth_texture, vertex_input, x as _, y as _);
                }
            }
        }
    }
}

pub struct DrawUnlitObjModelShader<'a> {
    pub model_view_projection_matrix: glam::Mat4,
    pub texture: &'a [Color24],
    pub texture_width: u16,
    pub texture_height: u16,
}
impl<'a> VertexShader for DrawUnlitObjModelShader<'a> {
    type Output = VertexShaderData;
    fn transform_vertices(&self, input: VertexShaderData) -> VertexShaderData {
        let position = self.model_view_projection_matrix * input.position;
        VertexShaderData { position, ..input }
    }
}

impl<'a> PixelShader for DrawUnlitObjModelShader<'a> {
    type Input = VertexShaderData;
    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        depth_texture: &mut [f32],
        vertex_input: VertexShaderData,
        fragment_x: u16,
        fragment_y: u16,
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

        software_buffer.set_pixel(fragment_x, fragment_y, color0.lerp(color1, v));
    }
}