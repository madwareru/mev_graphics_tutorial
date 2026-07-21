use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex05_uv_coordinates::DrawUVTriangleCommand
    },
    geometry::{Point},
};

const POSITIONS: &[Point] = &[
    Point { x: 154, y: 460 },
    Point { x: 90, y: 68 },
    Point { x: 486, y: 20 },
    Point { x: 550, y: 412 },
];
const UV_COORDS: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.0, 1.0),
    (1.0, 1.0),
    (1.0, 0.0)
];
const INDICES: [[u16; 3]; 2] = [
    [0, 1, 2],
    [0, 2, 3]
];

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });
    for indices in INDICES.iter().copied() {
        buffer.draw_triangle(
            indices.map(|id| POSITIONS[id as usize]).into(),
            &DrawUVTriangleCommand {
                positions: POSITIONS,
                uv_coords: UV_COORDS, 
                indices
            }
        );
    }
    buffer.print_as_ppm();
}