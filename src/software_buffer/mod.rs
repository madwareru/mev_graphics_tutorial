pub mod ex00_base_operations;
pub mod ex00_printing;
pub mod ex01_basic_drawing;
pub mod ex02_winding_number_triangle;
pub mod ex03_winding_trick_for_shapes;
pub mod ex04_barycentric_coordinates;
pub mod ex05_uv_coordinates;
pub mod ex06_texture_mapping_nearest;
pub mod ex07_texture_mapping_bilinear;
pub mod ex08_drawing_simple_unlit_3d_model;
pub mod ex09_visualize_normals;
pub mod ex0a_gouraud_shading;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color24 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug)]
pub struct SoftwareBuffer {
    pixels: Vec<Color24>,
    width: u16,
    height: u16,
}

impl SoftwareBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            pixels: vec![Color24::default(); width as usize * height as usize],
            width,
            height,
        }
    }
}