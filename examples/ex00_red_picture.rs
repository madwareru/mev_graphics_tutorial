use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24
    }
};

const NICE_RED: Color24 = Color24 { r: 207, g: 81, b: 99 };

/// This example demonstrates the simplest possible drawing.
/// We are creating the buffer of pixels, then it gets filled by a color.
/// Then it gets printed as a ppm image into the terminal.
///
/// The functions for clearing of a buffer and for printing of ppm image
/// are extending the functionality of the `SoftwareBuffer` struct. They
/// are implemented in accompanying modules `ex00_base_operations` and
/// `ex00_ppm_image`.
pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(NICE_RED);
    buffer.print_as_ppm();
}