use super::{SoftwareBuffer};

impl SoftwareBuffer {
    pub fn print_as_ppm(&self) {
        println!("P3");
        println!("{} {}", self.width, self.height);
        println!("255");

        for pixel in self.pixels.iter() {
            println!(" {} {} {} ", pixel.r, pixel.g, pixel.b);
        }
    }
}
