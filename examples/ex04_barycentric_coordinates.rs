use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex04_barycentric_coordinates::DrawBarycentricTriangleCommand
    },
    geometry::{Point, Triangle},
};

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });
    let triangle = Triangle::new(
        Point { x: 24, y: 456 },
        Point { x: 616, y: 456 },
        Point { x: 320, y: 24 },
    );
    buffer.draw_triangle(
        triangle,
        &DrawBarycentricTriangleCommand([
            triangle.a, 
            triangle.b, 
            triangle.c
        ])
    );
    buffer.print_as_ppm();
}