use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex03_winding_trick_for_shapes::InvertPixelCommand
    },
    geometry::{Point, Triangle},
};

const LETTER_COORDS: &[&[Point]] = &[
    &[
        Point { x: 285, y: 50 },
        Point { x: 285, y: 140 },
        Point { x: 335, y: 140 },
        Point { x: 335, y: 100 },
        Point { x: 295, y: 100 },
        Point { x: 295, y: 50 },
        Point { x: 285, y: 50 },
    ],
    &[
        Point { x: 295, y: 110 },
        Point { x: 295, y: 130 },
        Point { x: 325, y: 130 },
        Point { x: 325, y: 110 },
    ],
    &[
        Point { x: 345, y: 50 },
        Point { x: 345, y: 140 },
        Point { x: 355, y: 140 },
        Point { x: 355, y: 50 },
    ]
];

const VIBRANT_VIOLET: Color24 = Color24 { r: 131, g: 46, b: 184 };
const VIBRANT_YELLOW: Color24 = Color24 { r: 221, g: 181, b: 71 };
const MILD_PINK: Color24 = Color24 { r: 184, g: 111, b: 125 };
const MILD_ORANGE: Color24 = Color24 { r: 202, g: 146, b: 91 };

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    for y in 0..buffer.get_height() {
        let v = (y as f32 / (buffer.get_height() - 1) as f32).clamp(0.0, 1.0);
        let left_col_color = MILD_ORANGE.lerp(VIBRANT_VIOLET, v);
        let right_col_color = VIBRANT_YELLOW.lerp(MILD_PINK, v);
        for x in 0..buffer.get_width() {
            let u = (x as f32 / (buffer.get_width() - 1) as f32).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, left_col_color.lerp(right_col_color, u));
        }
    }

    fn draw_letter(buffer: &mut SoftwareBuffer, offset_x: i16, offset_y: i16) {
        // You can set this to any point on the screen.
        // The effect will be the same.
        let a = Point { x: 0, y: 0 };
        for sub_shape in LETTER_COORDS {
            for i in 1..=sub_shape.len() {
                let mut b = sub_shape[i - 1];
                b.x += offset_x;
                b.y += offset_y;
                let mut c = sub_shape[i % sub_shape.len()];
                c.x += offset_x;
                c.y += offset_y;
                buffer.draw_triangle(Triangle::new(a, b, c), &InvertPixelCommand);
            }
        }
    }

    for (offset_x, offset_y) in [
        (0, 0),
        (-100, 150),
        (100, 150),
        (-200, 300),
        (0, 300),
        (200, 300),
    ] {
        draw_letter(&mut buffer, offset_x, offset_y);
    }

    buffer.print_as_ppm();
}