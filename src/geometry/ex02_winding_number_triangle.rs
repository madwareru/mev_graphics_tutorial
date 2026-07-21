use std::ops::RangeInclusive;
use glam::vec2;
use super::Triangle;

impl Triangle {
    pub fn winding_n(self, x_range: RangeInclusive<i16>, y: i16) -> i32 {
        let max_y = self.a.y.max(self.b.y).max(self.c.y);
        // A hack to fix the problem when the line gets collided exactly
        // with points of the triangle. In this case the winding number
        // becomes not correct, so we slightly shift the line to be slightly
        // below (in the case of uppermost or middle points) or above the point
        // (in the case of a lowermost point).
        let y = if y == max_y { y as f32 - 0.005 } else { y as f32 + 0.005 };

        let (x0, x1) = (*x_range.start() as f32, *x_range.end() as f32);

        let mut winding_number = 0;
        for [a, b] in self.lines().map(|it| [
            vec2(it.start.x as f32, it.start.y as f32 ),
            vec2(it.end.x as f32, it.end.y as f32)
        ]) {
            let (min_x, min_y, max_x, max_y) = if a.y < b.y {
                (a.x, a.y, b.x, b.y)
            } else {
                (b.x, b.y, a.x, a.y)
            };

            if !(min_y..=max_y).contains(&y) {
                continue;
            }

            let dy = max_y - min_y;
            if dy < f32::EPSILON {
                if (x0..=x1).contains(&a.x) || (x0..=x1).contains(&b.x) {
                    winding_number += 1;
                }
                continue;
            }

            let t = (y - min_y) / dy;
            let x = min_x + t * (max_x - min_x);

            if (x0..=x1).contains(&x) {
                winding_number += 1;
            }
        }
        winding_number
    }
}