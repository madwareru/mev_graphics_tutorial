# Lesson I.1: Draw a Moon

> **Result:** `pictures/ex01_draw_moon.ppm`
>
> In this lesson we will draw a simple night scene with a crescent moon
> and a ground plane. We'll learn how to draw basic shapes — circles and
> rectangles — by testing individual pixels against geometric formulas.

---

## What We Are Doing

In the previous lesson we filled the entire buffer with a single color.
Now we want to draw actual shapes. The general approach to drawing shapes
on a pixel buffer is called the **"per-pixel test"** method:

1. Find the bounding box of the shape (the smallest rectangle that fully
   contains it).
2. For each pixel in that bounding box, test whether the pixel lies inside
   the shape using a mathematical formula.
3. If it does, set its color. If not, leave it unchanged.

This is the simplest possible drawing algorithm. It's not the fastest, but 
it's easy to understand and correct.

---

## Drawing Functions

The module `ex01_basic_drawing` adds two new methods to `SoftwareBuffer`.

### `fill_circle` — drawing a filled circle

```rust
pub fn fill_circle(
    &mut self,
    x_origin: i16,
    y_origin: i16,
    radius: u16,
    color: Color24,
) {
    let [min_x, max_x] = [
        x_origin as i32 - radius as i32,
        x_origin as i32 + radius as i32,
    ]
    .map(|it| it.clamp(0, self.width as i32) as i16);
    let [min_y, max_y] = [
        y_origin as i32 - radius as i32,
        y_origin as i32 + radius as i32,
    ]
    .map(|it| it.clamp(0, self.height as i32) as i16);

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
```

Let's break this down.

**Parameters:**
- `x_origin`, `y_origin` — the center of the circle. Note these are `i16`
  (signed), not `u16`. This allows the circle center to be placed outside
  the visible area, which is useful when you only want to show part of a
  circle (e.g., a moon that is partially off-screen).
- `radius` — the radius of the circle in pixels.
- `color` — the fill color.

**Bounding box calculation:**

```rust
let [min_x, max_x] = [
    x_origin as i32 - radius as i32,
    x_origin as i32 + radius as i32,
]
.map(|it| it.clamp(0, self.width as i32) as i16);
```

The bounding box extends from `x_origin - radius` to `x_origin + radius`
in both directions. Since coordinates are signed, the box could extend
beyond the buffer edges. We clamp it to the image bounds `[0, width)`
and `[0, height)` so we never iterate over pixels that don't exist.

**The circle test:**

```rust
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
```

The equation of a circle with center `(cx, cy)` and radius `r` is:

```
(x - cx)² + (y - cy)² = r²
```

A point `(x, y)` is **inside** the circle if:

```
(x - cx)² + (y - cy)² ≤ r²
```

We precompute `r²` (as `r_squared`) to avoid computing square roots.
For each pixel in the bounding box, we compute `dx = cx - x` and
`dy = cy - y`, then check if `dx² + dy² ≤ r²`. If true, the pixel
is inside the circle and we draw it.

> **Why compute `r_squared` instead of `radius`?** The distance from
> a point to the center is `sqrt(dx² + dy²)`. Comparing with `radius`
> would require a square root on every pixel, which is expensive.
> Since `r ≥ 0`, comparing `dx² + dy² ≤ r²` is mathematically equivalent
> and avoids square roots entirely.

### `fill_rectangle` — drawing a filled rectangle

```rust
pub fn fill_rectangle(
    &mut self,
    x_origin: i16,
    y_origin: i16,
    width: u16,
    height: u16,
    color: Color24,
) {
    let [min_x, max_x] = [x_origin as i32, x_origin as i32 + width as i32]
        .map(|it| it.clamp(0, self.width as i32) as i16);
    let [min_y, max_y] = [y_origin as i32, y_origin as i32 + height as i32]
        .map(|it| it.clamp(0, self.height as i32) as i16);
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            self.set_pixel(x as _, y as _, color);
        }
    }
}
```

A rectangle is even simpler: it's just a range of `x` values and a range
of `y` values. The bounding box *is* the rectangle itself. We clamp the
coordinates to the image bounds and draw every pixel in the range.

Note that `x_origin` and `y_origin` are `i16` (signed), while `width` and
`height` are `u16` (unsigned). This means the rectangle's top-left corner
can be placed at negative coordinates (clamped to the visible area), which
allows partially off-screen rectangles.

---

## Drawing a Crescent Moon

The moon in this example is drawn with a clever trick:

```rust
fn draw_moon(buffer: &mut SoftwareBuffer, x: i16, y: i16) {
    buffer.fill_circle(x, y, MOON_RADIUS, MOON_COLOR);
    buffer.fill_circle(x - 20, y + 10, MOON_RADIUS, SKY_COLOR);
}
```

1. First, draw a full circle of the moon color (pale yellow-white) at the
   moon's center position.
2. Then, draw a second circle of the **sky color** at a slightly offset
   position — shifted 20 pixels to the left and 10 pixels down — with the
   same radius.

The effect is that the second circle "erases" part of the first one,
leaving only a crescent shape. This is a simple but powerful technique:
you can create complex shapes by layering filled primitives on top of
each other.

This technique is called **constructive solid geometry** (CSG) in its
simplest form — building shapes by combining simpler ones. Here we use
subtraction (drawing sky color over the moon), but the same approach
works for addition (overlapping shapes of different colors).

---

## Example Walkthrough

Now let's look at the full example — `examples/ex01_draw_moon.rs`:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24
    }
};

const MOON_RADIUS: u16 = 32;
const MOON_COLOR: Color24 = Color24 { r: 244, g: 244, b: 220 };
const SKY_COLOR: Color24 = Color24 { r: 17, g: 13, b: 22 };
const GROUND_COLOR: Color24 = Color24 { r: 112, g: 104, b: 170 };

const IMAGE_WIDTH: u16 = 640;
const IMAGE_HEIGHT: u16 = 480;

fn draw_moon(buffer: &mut SoftwareBuffer, x: i16, y: i16) {
    buffer.fill_circle(x, y, MOON_RADIUS, MOON_COLOR);
    buffer.fill_circle(x - 20, y + 10, MOON_RADIUS, SKY_COLOR);
}

pub fn main() {
    let mut buffer = SoftwareBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    buffer.clear(SKY_COLOR);

    draw_moon(&mut buffer, 488, 113);

    let ground_y = 297;
    let ground_height = IMAGE_HEIGHT - ground_y as u16;
    buffer.fill_rectangle(0, ground_y, IMAGE_WIDTH, ground_height, GROUND_COLOR);

    buffer.print_as_ppm();
}
```

### Step 1: Constants

```rust
const MOON_RADIUS: u16 = 32;
const MOON_COLOR: Color24 = Color24 { r: 244, g: 244, b: 220 };
const SKY_COLOR: Color24 = Color24 { r: 17, g: 13, b: 22 };
const GROUND_COLOR: Color24 = Color24 { r: 112, g: 104, b: 170 };
```

We define the scene's colors and the moon's radius as constants. The
moon color is a pale yellow-white, the sky is a deep dark blue, and
the ground is a muted purple-blue.

### Step 2: Create the Buffer

```rust
let mut buffer = SoftwareBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
buffer.clear(SKY_COLOR);
```

Create a 640×480 buffer and fill it with the sky color. This is our
background.

### Step 3: Draw the Moon

```rust
draw_moon(&mut buffer, 488, 113);
```

The moon is centered at `(488, 113)` — in the upper-right area of the
image. The `draw_moon` function draws two offset circles as described
above, creating a crescent.

### Step 4: Draw the Ground

```rust
let ground_y = 297;
let ground_height = IMAGE_HEIGHT - ground_y as u16;
buffer.fill_rectangle(0, ground_y, IMAGE_WIDTH, ground_height, GROUND_COLOR);
```

We compute `ground_height` as `IMAGE_HEIGHT - ground_y` so the rectangle
extends from `y = 297` to the bottom of the image. This fills the lower
third of the screen with the ground color.

### Step 5: Output

```rust
buffer.print_as_ppm();
```

Output the result in PPM format, just like in lesson 0.

---

## The Drawing Order Matters

The order in which we draw shapes is critical:

1. **Clear** — fills the entire buffer with the sky color.
2. **Moon base** — draws a full circle in moon color.
3. **Moon cutout** — draws a sky-colored circle offset from the moon,
   "erasing" part of it to create a crescent.
4. **Ground** — fills the bottom rectangle with the ground color, which
   paints over any part of the moon that lies in that area.

If we drew the ground before the moon, and the moon is low enough, the moon 
would appear on top of the ground, as well as a secret sky-colored circle. 
If we drew the cutout circle before the base circle, nothing would be erased 
(the sky-colored circle would be drawn first, then the moon-colored circle 
would cover it entirely).

This layering is the essence of how 2D rendering works — shapes are drawn
in sequence, and later shapes overwrite earlier ones.

---

## How to Run the Example

```sh
cargo run --example ex01_draw_moon > pictures/ex01_draw_moon.ppm
```

Or build and run separately:

```sh
cargo build --release --example ex01_draw_moon
./target/release/examples/ex01_draw_moon > pictures/ex01_draw_moon.ppm
```

Open `pictures/ex01_draw_moon.ppm` in any image viewer. You should see a
night scene with a crescent moon in the upper-right area and a purple-blue
ground plane at the bottom.

---

## Summary

In this lesson we learned about:

- **Per-pixel shape testing** — iterating over a bounding box and testing
  each pixel against a geometric formula.
- **`fill_circle`** — drawing a filled circle by checking `dx² + dy² ≤ r²`
  for each pixel in the bounding box.
- **`fill_rectangle`** — drawing a filled rectangle by iterating over a
  range of x and y coordinates.
- **Shape composition** — creating a crescent moon by drawing a sky-colored
  circle offset from a moon-colored circle, effectively "erasing" part of it.
- **Drawing order** — the sequence of draw calls determines which shapes
  appear on top.

In the next lesson we'll learn how to draw a triangle using the winding
number method.

---

## Exercises

### Exercise 1: Move the moon

Change the `draw_moon` call to place the moon in the lower-left corner of
the image. What happens to the crescent cutout when the moon is near the
edge? Why does the `fill_circle` function still work correctly even when
the center is near the boundary?

### Exercise 2: Multiple moons

Draw three moons of different sizes at different positions: one large moon
(radius 48), one medium (radius 32), and one small (radius 16). Use a
different shade of yellow for each.

### Exercise 3: A starry sky

Add a few stars to the sky. A star is just a single pixel (or a small
cluster of pixels) drawn with a white color. Use `set_pixel` to draw
stars at random-looking positions. For example, to draw a star at
`(x, y)`:

```rust
buffer.set_pixel(x, y, Color24 { r: 255, g: 255, b: 255 });
```

### Exercise 4: Think about performance

For a circle of radius 32, how many pixels are in the bounding box?
How many of those are actually inside the circle? What is the ratio
(inside / total)? This ratio is called the **fill efficiency** of the
algorithm. How does it change as the radius grows?