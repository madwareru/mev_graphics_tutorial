use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex02_winding_number_triangle::FillPixelCommand
    },
    geometry::{Point, Triangle},
};

/// This example demonstrates the concept of a winding number used to draw
/// vector shapes. Then this trick is used to draw a colored triangle on a
/// screen.
pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });
    buffer.draw_triangle(
        Triangle::new(
            Point { x: 24, y: 456 },
            Point { x: 616, y: 456 },
            Point { x: 320, y: 24 },
        ),
        &FillPixelCommand(Color24 { r: 0x90, g: 0x70, b: 0x68 })
    );
    buffer.print_as_ppm();
}