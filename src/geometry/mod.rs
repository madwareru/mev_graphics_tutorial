pub mod ex02_winding_number_triangle;
pub mod ex04_barycentric_coordinates;

#[derive(Copy, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16
}

#[derive(Copy, Clone)]
pub struct Line {
    pub start: Point,
    pub end: Point
}
impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}
impl Triangle {
    pub fn new(a: Point, b: Point, c: Point) -> Self {
        Self { a, b, c }
    }
    pub fn lines(self) -> [Line; 3] {
        [
            Line::new(self.a, self.b),
            Line::new(self.b, self.c),
            Line::new(self.c, self.a)
        ]
    }
}
impl<TIterator: IntoIterator<Item=Point>> From<TIterator> for Triangle {
    fn from(points: TIterator) -> Self {
        let mut iter = points.into_iter();
        let a = iter.next().expect("Failed to get point a");
        let b = iter.next().expect("Failed to get point b");
        let c = iter.next().expect("Failed to get point c");
        Self { a, b, c }
    }
}