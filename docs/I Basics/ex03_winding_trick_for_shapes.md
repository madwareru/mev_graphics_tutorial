# Lesson 3: The Winding Trick for Vector Shapes

> **Result:** `pictures/ex03_winding_trick_for_shapes.ppm`
>
> In this lesson we will learn a clever trick to draw arbitrary vector shapes
> using only a triangle drawing routine. Instead of testing each pixel against
> a complex polygon, we draw a fan of triangles from a fixed reference point
> where each triangle **inverts** the pixels it covers. The shape emerges
> naturally from the even-odd rule.

---

## What We Are Doing

In the previous lesson we used the winding number to test whether a pixel
lies inside a triangle. This works for any polygon, but performing the test
for every pixel inside a bounding box can be costly.

The **winding trick** takes a completely different approach: instead of
*testing* pixels, we *draw* triangles that physically realize the winding
number test. The idea is:

1. Pick any point `a` on the screen (it can be anywhere — even inside the
   shape or outside the image bounds).
2. For each edge `(v_i, v_{i+1})` of the shape's outline, draw triangle
   `(a, v_i, v_{i+1})` using a command that **inverts** the color of each
   pixel it touches.
3. After drawing all triangles, the shape is filled and the surrounding
   area is restored to its original color.

This works because of the even-odd rule: each point on the screen is covered
by either an odd or even number of triangles. Pixels covered by an odd number
get inverted; pixels covered by an even number return to their original color.

---

## The Invert Command

The module `ex03_winding_trick_for_shapes` introduces a new pixel drawing
command and some helper methods on `Color24`.

### `Color24::invert`

```rust
impl Color24 {
    pub fn invert(self) -> Color24 {
        Color24 {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        }
    }
}
```

Inverting a color means subtracting each channel from `255`. For example:
- Black `(0, 0, 0)` becomes white `(255, 255, 255)`.
- White `(255, 255, 255)` becomes black `(0, 0, 0)`.
- Red `(255, 0, 0)` becomes cyan `(0, 255, 255)`.

Applying the inversion twice returns the original color: `color.invert().invert() == color`.

### `Color24::lerp`

```rust
impl Color24 {
    pub fn lerp(self, other: Color24, t: f32) -> Color24 {
        Color24 {
            r: (self.r as f32 * (1.0 - t) + other.r as f32 * t)
                .round().clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * (1.0 - t) + other.g as f32 * t)
                .round().clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * (1.0 - t) + other.b as f32 * t)
                .round().clamp(0.0, 255.0) as u8,
        }
    }
}
```

**Lerp** (linear interpolation) blends between two colors. When `t = 0.0`,
the result is `self`. When `t = 1.0`, the result is `other`. Values in
between produce a smooth mix. We'll use this to create the gradient
background.

### `InvertPixelCommand`

```rust
pub struct InvertPixelCommand;

impl PixelDrawingCommand for InvertPixelCommand {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16) {
        let Some(color) = software_buffer.get_pixel(x, y) else { return };
        software_buffer.set_pixel(x, y, color.invert())
    }
}
```

Instead of setting a fixed color, `InvertPixelCommand` reads the current
pixel color, inverts it, and writes it back. This is the key to the winding
trick: a pixel that is covered by two triangles will be inverted twice,
returning to its original color.

---

## How the Winding Trick Works

Let's trace through what happens for different points on the screen.

Consider a simple shape (a triangle) and a reference point `a` placed
**outside** the shape. We draw three triangles, one for each edge:

```
(a, v1, v2), (a, v2, v3), (a, v3, v1)
```

**For a point P inside the shape:** The ray from `a` to P crosses the
shape's boundary exactly once. This means P lies inside exactly one of
the three triangles. It gets inverted once — it becomes the inverted color.

**For a point Q outside the shape (but inside the fan):** The ray from `a`
to Q crosses the shape's boundary either 0 or 2 times. If 0 times, Q is
not inside any triangle — it stays unchanged. If 2 times, Q is inside two
triangles — it gets inverted twice, returning to its original color.

**For a point R far outside the fan:** It's not inside any triangle —
unchanged.

The net result: the shape appears as an inverted region on the screen,
and everything else is unchanged.

### What if `a` is inside the shape?

If `a` is placed inside the shape, the effect is reversed: the **outside**
of the shape gets inverted, and the inside stays unchanged. This is because
the ray from `a` (inside) to a point outside the shape crosses the boundary
once, while the ray to a point inside crosses 0 times.

The example places `a` at `(0, 0)` — the top-left corner of the image —
which is outside the letters, so the letters are filled with inverted colors.

### The reference point can be anywhere

One remarkable property: the reference point `a` can be **any point on the
screen**, and it doesn't affect the final result (as long as the triangles
fully cover the shape). The example demonstrates this by drawing the same
letter at multiple positions using the same reference point `a = (0, 0)`.

---

## The Gradient Background

Before drawing the letters, the example fills the buffer with a smooth
gradient:

```rust
for y in 0..buffer.get_height() {
    let v = (y as f32 / (buffer.get_height() - 1) as f32).clamp(0.0, 1.0);
    let left_col_color = MILD_ORANGE.lerp(VIBRANT_VIOLET, v);
    let right_col_color = VIBRANT_YELLOW.lerp(MILD_PINK, v);
    for x in 0..buffer.get_width() {
        let u = (x as f32 / (buffer.get_width() - 1) as f32).clamp(0.0, 1.0);
        buffer.set_pixel(x, y, left_col_color.lerp(right_col_color, u));
    }
}
```

This creates a **bilinear gradient**: each row interpolates between a
left color and a right color, and those colors themselves change smoothly
from top to bottom. The result is a rich, colorful background that makes
the inverted letters stand out beautifully.

---

## Defining the Letter Shapes

The letters are defined as sequences of points forming closed polygons:

```rust
const LETTER_COORDS: &[&[Point]] = &[
    &[
        // Outer outline of the letter "Ы" left side
        Point { x: 285, y: 50 },
        Point { x: 285, y: 140 },
        Point { x: 335, y: 140 },
        Point { x: 335, y: 100 },
        Point { x: 295, y: 100 },
        Point { x: 295, y: 50 },
        Point { x: 285, y: 50 },
    ],
    &[
        // Inner cutout — a hole in the "Ы"
        Point { x: 295, y: 110 },
        Point { x: 295, y: 130 },
        Point { x: 325, y: 130 },
        Point { x: 325, y: 110 },
    ],
    &[
        // Right vertical bar of the "Ы"
        Point { x: 345, y: 50 },
        Point { x: 345, y: 140 },
        Point { x: 355, y: 50 },
    ],
];
```

Each sub-shape is a closed polygon. The first sub-shape is the outer
boundary of the "Ы" letter. The second sub-shape is a rectangular hole
inside it — because the triangle fan inverts the hole area twice (once
from the outer outline, once from the hole), the net effect is that the
hole returns to its original background color. The third sub-shape is
the right vertical stroke of the "Ы".

---

## Drawing the Triangles

The drawing function is deceptively simple:

```rust
fn draw_letter(buffer: &mut SoftwareBuffer, offset_x: i16, offset_y: i16) {
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
```

For each sub-shape (closed polygon), we iterate over its edges. For each
edge `(v_i, v_{i+1})`, we draw triangle `(a, v_i, v_{i+1})` using the
`InvertPixelCommand`. The indices wrap around: `i % sub_shape.len()`
connects the last vertex back to the first.

Note that `a = (0, 0)` is fixed — it's always the top-left corner of the
image. The letter vertices are offset by `(offset_x, offset_y)` to place
the letter at different positions.

### Multiple instances

```rust
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
```

The same letter shape is drawn at six different positions on the screen.
Because the fan always originates from `(0, 0)`, the triangles from
different instances overlap, but the inversion operation is idempotent
in the sense that overlapping regions invert twice (or more) and return
to their original colors — as long as the letter instances don't overlap
with each other.

---

## Example Walkthrough

Now let's look at the full example — `examples/ex03_winding_trick_for_shapes.rs`:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex03_winding_trick_for_shapes::InvertPixelCommand
    },
    geometry::{Point, Triangle},
};

const VIBRANT_VIOLET: Color24 = Color24 { r: 131, g: 46, b: 184 };
const VIBRANT_YELLOW: Color24 = Color24 { r: 221, g: 181, b: 71 };
const MILD_PINK: Color24 = Color24 { r: 184, g: 111, b: 125 };
const MILD_ORANGE: Color24 = Color24 { r: 202, g: 146, b: 91 };

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);

    // Step 1: Draw a gradient background
    for y in 0..buffer.get_height() {
        let v = (y as f32 / (buffer.get_height() - 1) as f32).clamp(0.0, 1.0);
        let left_col_color = MILD_ORANGE.lerp(VIBRANT_VIOLET, v);
        let right_col_color = VIBRANT_YELLOW.lerp(MILD_PINK, v);
        for x in 0..buffer.get_width() {
            let u = (x as f32 / (buffer.get_width() - 1) as f32).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, left_col_color.lerp(right_col_color, u));
        }
    }

    // Step 2: Define the letter drawing function
    fn draw_letter(buffer: &mut SoftwareBuffer, offset_x: i16, offset_y: i16) {
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

    // Step 3: Draw the letter at multiple positions
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

    // Step 4: Output
    buffer.print_as_ppm();
}
```

### Step 1: Gradient Background

The buffer is filled pixel by pixel with a bilinear gradient. The `v`
coordinate (vertical, 0 to 1) interpolates between orange and violet on
the left side, and between yellow and pink on the right side. The `u`
coordinate (horizontal, 0 to 1) interpolates between the left and right
colors for each row.

### Step 2: The Letter Drawing Function

`draw_letter` draws a single letter shape at a given offset. It uses the
winding trick: a fixed reference point `a = (0, 0)` and a fan of triangles
for each edge of each sub-shape.

### Step 3: Multiple Instances

Six instances of the letter are drawn at different positions, creating a
pattern across the image.

### Step 4: Output

```rust
buffer.print_as_ppm();
```

---

## How to Run the Example

```sh
cargo run --example ex03_winding_trick_for_shapes > pictures/ex03_winding_trick_for_shapes.ppm
```

Or build and run separately:

```sh
cargo build --release --example ex03_winding_trick_for_shapes
./target/release/examples/ex03_winding_trick_for_shapes > pictures/ex03_winding_trick_for_shapes.ppm
```

Open `pictures/ex03_winding_trick_for_shapes.ppm` in any image viewer. You
should see a colorful gradient background with the letters "Ы" drawn in
inverted colors at six positions across the screen.

---

## Summary

In this lesson we learned about:

- **The winding trick** — drawing a fan of triangles from a fixed reference
  point, using an invert command, to fill arbitrary vector shapes.
- **The even-odd rule in action** — the inversion count per pixel follows
  the winding number: odd → inverted, even → unchanged.
- **`Color24::invert`** — subtracting each channel from 255.
- **`Color24::lerp`** — linear interpolation between two colors.
- **`InvertPixelCommand`** — a pixel command that reads, inverts, and writes
  back the current pixel color.
- **The fan from a fixed point** — the reference point `a` can be anywhere
  on the screen; the result is the same regardless of its position.
- **Gradient backgrounds** — creating a smooth bilinear gradient by
  interpolating between four corner colors.

In the next lesson we'll introduce barycentric coordinates, a powerful tool
for interpolating values across the surface of a triangle.

---

## Exercises

### Exercise 1: Move the reference point

Change the reference point `a` from `(0, 0)` to the center of the image
`(320, 240)`. Does the result change? Try placing `a` inside one of the
letter shapes. What happens to that letter?

### Exercise 2: Draw your own shape

Define your own vector shape — a star, a heart, a house, or your initials
— and draw it using the winding trick. Make sure to define the shape as a
closed polygon (or multiple polygons for shapes with holes).

### Exercise 3: Solid color instead of invert

Create a new `FillPixelCommand` variant that uses the winding trick but
fills the shape with a solid color instead of inverting. Hint: you need to
track which pixels have been touched an odd number of times. One approach
is to draw the triangles with a special "marker" color first, then replace
the marked pixels with the fill color in a second pass.

### Exercise 4: Why does the inner cutout work?

In the letter "Ы", the second sub-shape is a small rectangle inside the
larger outline. Explain, step by step, why this rectangle appears as a
hole (the background gradient shows through) instead of being filled with
the inverted color. How many times is each pixel in the hole inverted?