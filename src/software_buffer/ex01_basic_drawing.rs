use super::{Color24, SoftwareBuffer};

impl SoftwareBuffer {
    /// Draws a filled circle on the buffer.
    pub fn fill_circle(&mut self, x_origin: i16, y_origin: i16, radius: u16, color: Color24) {
        let [min_x, max_x] = [
            x_origin as i32 - radius as i32,
            x_origin as i32 + radius as i32
        ].map(|it| it.clamp(0, self.width as i32) as i16);
        let [min_y, max_y] = [
            y_origin as i32 - radius as i32,
            y_origin as i32 + radius as i32
        ].map(|it| it.clamp(0, self.height as i32) as i16);

        let r_squared = (radius * radius) as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let dx = x_origin as i32 - x as i32;
                let dy = y_origin as i32 - y as i32;
                if r_squared >= (dx * dx + dy * dy) {
                    self.set_pixel(x as _, y as _, color);
                }
            }
        }
    }

    /// Draws a filled rectangle on the buffer.
    pub fn fill_rectangle(&mut self, x_origin: i16, y_origin: i16, width: u16, height: u16, color: Color24) {
        let [min_x, max_x] = [
            x_origin as i32,
            x_origin as i32 + width as i32
        ].map(|it| it.clamp(0, self.width as i32) as i16);
        let [min_y, max_y] = [
            y_origin as i32,
            y_origin as i32 + height as i32
        ].map(|it| it.clamp(0, self.height as i32) as i16);
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                self.set_pixel(x as _, y as _, color);
            }
        }
    }
}