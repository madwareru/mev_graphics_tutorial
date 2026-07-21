use super::{Color24, SoftwareBuffer};

impl SoftwareBuffer {
    /// Returns the width of the buffer in pixels.
    pub fn get_width(&self) -> u16 {
        self.width
    }

    /// Returns the height of the buffer in pixels.
    pub fn get_height(&self) -> u16 {
        self.height
    }

    /// Clears the buffer with the given color.
    pub fn clear(&mut self, color: Color24) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color;
        }
    }

    /// Sets the color of the pixel at the given position.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color24) {
        let idx = y as usize * self.width as usize + x as usize;
        let Some(pixel) = self.pixels.get_mut(idx) else {
            return;
        };
        *pixel = color;
    }

    /// Returns the color of the pixel at the given position.
    /// If the position is out of bounds, returns None
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<Color24> {
        let idx = y as usize * self.width as usize + x as usize;
        self.pixels.get(idx).copied()
    }
}
