use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex06_texture_mapping_nearest::DrawTextureMappedTriangleNearestCommand
    },
    geometry::{Point},
};

const SWIBORG_TEXTURE_BYTES : &[u8] = include_bytes!("../assets/swiborg.png");

const POSITIONS: &[Point] = &[
    Point { x: 154, y: 460 },
    Point { x: 90, y: 68 },
    Point { x: 486, y: 20 },
    Point { x: 550, y: 412 },
];
const UV_COORDS: &[(f32, f32)] = &[
    (0.25, 0.25),
    (0.25, 0.75),
    (0.75, 0.75),
    (0.75, 0.25)
];
const INDICES: [[u16; 3]; 2] = [
    [0, 1, 2],
    [0, 2, 3]
];

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });

    let image = image::load_from_memory(SWIBORG_TEXTURE_BYTES)
        .expect("Failed to load image");
    let image = image.to_rgb8();
    let texture_width = image.width() as u16;
    let texture_height = image.height() as u16;

    if texture_width != 0 && texture_height != 0 {
        let mut texture = vec![Color24{r: 0, g: 0, b: 0}; texture_width as usize * texture_height as usize ];
        for (i, pixel) in image.pixels().enumerate() {
            texture[i] = Color24 {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2]
            }
        }

        for indices in INDICES.iter().copied() {
            buffer.draw_triangle(
                indices.map(|id| POSITIONS[id as usize]).into(),
                &DrawTextureMappedTriangleNearestCommand {
                    positions: POSITIONS,
                    uv_coords: UV_COORDS,
                    indices,
                    texture: &texture,
                    texture_width,
                    texture_height
                }
            );
        }
    }

    buffer.print_as_ppm();
}