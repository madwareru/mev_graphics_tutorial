use glam::{vec2, Vec2};
use super::Point;

impl Point {
    pub fn calculate_barycentric_in(self, positions: [Self; 3]) -> [f32; 3] {
        let pt = vec2(self.x as f32, self.y as f32);
        let positions = positions.map(|it| vec2(it.x as f32, it.y as f32));
        calculate_barycentric(pt, positions)
    }
}

fn calculate_barycentric(pt: Vec2, [pt0, pt1, pt2]: [Vec2; 3]) -> [f32; 3] {
    fn area(lhs: Vec2, rhs: Vec2) -> f32 {
        let lhs = glam::vec3(lhs.x, lhs.y, 0.0);
        let rhs = glam::vec3(rhs.x, rhs.y, 0.0);
        lhs.cross(rhs).z
    }

    let e0 = pt1 - pt0;
    let e1 = pt2 - pt1;

    let v0 = pt - pt0;
    let v1 = pt - pt1;
    let v2 = pt - pt2;

    let a = area(e0, e1);
    [
        area(v1, v2) / a,
        area(v2, v0) / a,
        area(v0, v1) / a
    ]
}

pub fn mix_1_component_by_barycentric(values: [f32; 3], barycentric: [f32; 3]) -> f32 {
    barycentric.iter().copied()
        .zip(values.iter().copied())
        .fold(0.0, |acc, (mul, x)| acc + x * mul)
}

pub fn mix_2_components_by_barycentric(values: [(f32, f32); 3], barycentric: [f32; 3]) -> (f32, f32) {
    barycentric.iter().copied()
        .zip(values.iter().copied())
        .fold((0.0, 0.0), |(acc_x, acc_y), (mul, (x, y))|
            (
                acc_x + x * mul,
                acc_y + y * mul
            )
        )
}

pub fn mix_3_components_by_barycentric(values: [(f32, f32, f32); 3], barycentric: [f32; 3]) -> (f32, f32, f32) {
    barycentric.iter().copied()
        .zip(values.iter().copied())
        .fold((0.0, 0.0, 0.0), |(acc_x, acc_y, acc_z), (mul, (x, y, z))|
            (
                acc_x + x * mul,
                acc_y + y * mul,
                acc_z + z * mul
            )
        )
}

pub fn mix_4_components_by_barycentric(values: [(f32, f32, f32, f32); 3], barycentric: [f32; 3]) -> (f32, f32, f32, f32) {
    barycentric.iter().copied()
        .zip(values.iter().copied())
        .fold((0.0, 0.0, 0.0, 0.0), |(acc_x, acc_y, acc_z, acc_w), (mul, (x, y, z, w))|
            (
                acc_x + x * mul,
                acc_y + y * mul,
                acc_z + z * mul,
                acc_w + w * mul
            )
        )
}