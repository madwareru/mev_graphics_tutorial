# Lesson 7: Texture Mapping — Bilinear Interpolation

> **Result:** `pictures/ex07_texture_mapping_bilinear.ppm`
>
> In this lesson we'll improve upon nearest neighbor sampling by implementing
> **bilinear interpolation**. Instead of picking the closest texel, we blend
> between the four texels surrounding the sampling point. The result is a
> smooth, softened image without the blocky pixel-art look of nearest neighbor.

---

## What We Are Doing

In the previous lesson we sampled a texture using nearest neighbor: for each
pixel on screen we converted its UV coordinate to integer texel indices by
rounding. This gave us a crisp, blocky result where each visible texel is a
sharp square.

Now we'll implement **bilinear filtering**: for each sampling point we look up
the four nearest texels (forming a 2×2 quad) and blend between them — first
horizontally, then vertically — using the fractional parts of the UV
coordinates as blend weights. This produces a smooth, continuous image with no
visible texel grid.

---

## Nearest Neighbor vs Bilinear: The Difference

**Nearest neighbor** maps each pixel to exactly one texel:

```
UV → scale → round → single texel
```

**Bilinear** maps each pixel to four texels and blends:

```
UV → scale → separate integer and fractional parts
         → four corner texels (x0,y0), (x1,y0), (x0,y1), (x1,y1)
         → lerp horizontally: blend (x0,y0)↔(x1,y0) and (x0,y1)↔(x1,y1) by u_frac
         → lerp vertically: blend the two results by v_frac
```

The result is mathematically equivalent to a 2D linear interpolation — hence
the name **bilinear**.

---

## The Bilinear Sampling Command

The `DrawTextureMappedTriangleBilinearCommand` in
`src/software_buffer/ex07_texture_mapping_bilinear.rs` implements the new
filtering method:

```rust
pub struct DrawTextureMappedTriangleBilinearCommand<'a> {
    pub positions: &'a [Point],
    pub uv_coords: &'a [(f32, f32)],
    pub indices: [u16; 3],
    pub texture: &'a [Color24],
    pub texture_width: u16,
    pub texture_height: u16,
}

impl<'a> PixelDrawingCommand for DrawTextureMappedTriangleBilinearCommand<'a> {
    fn draw_pixel(&self, software_buffer: &mut SoftwareBuffer, x: u16, y: u16) {
        let indices = self.indices.map(|it| it as usize);

        assert_eq!(self.positions.len(), self.uv_coords.len());
        assert!(indices[0] < self.positions.len());
        assert!(indices[1] < self.positions.len());
        assert!(indices[2] < self.positions.len());

        let positions = indices.map(|id| self.positions[id]);
        let uv_coords = indices.map(|id| self.uv_coords[id]);

        let point = Point { x: x as _, y: y as _ };
        let barycentric_coords = point.calculate_barycentric_in(positions);
        let (u, v) = mix_2_components_by_barycentric(uv_coords, barycentric_coords);

        // Invert v to get the correct orientation of the texture
        let v = 1.0 - v;

        let texture_x = u * (self.texture_width - 1) as f32;
        let texture_y = v * (self.texture_height - 1) as f32;

        let u_frac = texture_x.fract();
        let v_frac = texture_y.fract();

        let x0 = (texture_x.trunc() as u16).clamp(0, self.texture_width - 1);
        let y0 = (texture_y.trunc() as u16).clamp(0, self.texture_height - 1);
        let x1 = (x0 + 1).clamp(0, self.texture_width - 1);
        let y1 = (y0 + 1).clamp(0, self.texture_height - 1);

        let id0 = y0 as usize * self.texture_width as usize + x0 as usize;
        let id1 = y0 as usize * self.texture_width as usize + x1 as usize;
        let id2 = y1 as usize * self.texture_width as usize + x0 as usize;
        let id3 = y1 as usize * self.texture_width as usize + x1 as usize;

        let color0 = self.texture[id0].lerp(self.texture[id1], u_frac);
        let color1 = self.texture[id2].lerp(self.texture[id3], u_frac);

        software_buffer.set_pixel(x, y, color0.lerp(color1, v_frac));
    }
}
```

Let's break down what's different from the nearest neighbor version.

### Step 1: Compute continuous texture coordinates

```rust
let texture_x = u * (self.texture_width - 1) as f32;
let texture_y = v * (self.texture_height - 1) as f32;
```

This is the same as before: scale the UV coordinates to texel space. But now
we keep the result as a **float** — we don't round it to an integer.

### Step 2: Extract integer and fractional parts

```rust
let u_frac = texture_x.fract();
let v_frac = texture_y.fract();

let x0 = (texture_x.trunc() as u16).clamp(0, self.texture_width - 1);
let y0 = (texture_y.trunc() as u16).clamp(0, self.texture_height - 1);
```

- `texture_x.trunc()` gives the integer part (e.g., `3.75` → `3.0`), which is
  the index of the **top-left** texel in the 2×2 quad.
- `texture_x.fract()` gives the fractional part (e.g., `3.75` → `0.75`), which
  tells us how close the sampling point is to the next texel to the right.
- `x0`, `y0` are the coordinates of the **top-left** corner of the 2×2 block.
- `u_frac` and `v_frac` are the **blend weights** between the left and right
  (and top and bottom) texels.

### Step 3: Find the four surrounding texels

```rust
let x1 = (x0 + 1).clamp(0, self.texture_width - 1);
let y1 = (y0 + 1).clamp(0, self.texture_height - 1);
```

- `x1`, `y1` are the coordinates of the **bottom-right** corner of the 2×2
  block.
- The `clamp` ensures we never go out of bounds, even at the right or bottom
  edge of the texture.

The four texels are:

| Texel        | Coordinates | ID formula                        |
|-------------|-------------|-----------------------------------|
| Top-left    | `(x0, y0)`  | `y0 * width + x0`  (id0)         |
| Top-right   | `(x1, y0)`  | `y0 * width + x1`  (id1)         |
| Bottom-left | `(x0, y1)`  | `y1 * width + x0`  (id2)         |
| Bottom-right| `(x1, y1)`  | `y1 * width + x1`  (id3)         |

```
(x0, y0)  --- u_frac --->  (x1, y0)
   |                          |
   v                          v
  u_frac                     u_frac
   |                          |
   v                          v
(x0, y1)  --- u_frac --->  (x1, y1)
```

### Step 4: Blend horizontally (twice)

```rust
let color0 = self.texture[id0].lerp(self.texture[id1], u_frac);
let color1 = self.texture[id2].lerp(self.texture[id3], u_frac);
```

- `color0` is the horizontal blend between the top-left and top-right texels.
- `color1` is the horizontal blend between the bottom-left and bottom-right
  texels.
- The blend weight `u_frac` determines how much of the right texel to use:
  - `u_frac = 0.0` → pure left texel
  - `u_frac = 1.0` → pure right texel

### Step 5: Blend vertically

```rust
software_buffer.set_pixel(x, y, color0.lerp(color1, v_frac));
```

Finally, we blend the two horizontal results vertically using `v_frac`:
- `v_frac = 0.0` → pure top row
- `v_frac = 1.0` → pure bottom row

The result is a weighted average of all four surrounding texels.

### An Intuition: Bilinear as a Bézier Patch

If you've encountered Bézier curves or splines before, bilinear interpolation
may feel familiar — and it should. The four texels `(x0, y0)`, `(x1, y0)`,
`(x0, y1)`, `(x1, y1)` can be thought of as the four **control points** of a
**bilinear surface patch**. The fractional coordinates `u_frac` and `v_frac`
are the parameters `(s, t)` that evaluate a point on that surface:

1. Evaluate two linear Bézier curves along the top and bottom rows at
   parameter `u_frac` — this gives two intermediate points.
2. Evaluate one linear Bézier curve between those two points at parameter
   `v_frac` — this gives the final surface point.

Equivalently, you could lerp vertically first and then horizontally — the
result is the same (the operation is commutative). This is exactly how
bilinear patches work in computer-aided design, except there the control
points are 3D positions rather than colors.

> **Why this matters:** The same principle extends to **bicubic**
> interpolation, where 16 control points (a 4×4 grid) define a smooth surface
> using cubic polynomials — the texture equivalent of a Catmull-Rom or
> Hermite spline patch.

### The `lerp` method

We're using `Color24::lerp` introduced in Lesson 3:

```rust
pub fn lerp(self, other: Color24, t: f32) -> Color24 {
    Color24 {
        r: (self.r as f32 * (1.0 - t) + other.r as f32 * t).round().clamp(0.0, 255.0) as u8,
        g: (self.g as f32 * (1.0 - t) + other.g as f32 * t).round().clamp(0.0, 255.0) as u8,
        b: (self.b as f32 * (1.0 - t) + other.b as f32 * t).round().clamp(0.0, 255.0) as u8,
    }
}
```

Each color channel is interpolated independently using the formula:

```
result = self · (1 - t) + other · t
```

This is the same operation as `mix` in GLSL or `lerp` in HLSL.

---

## Example Walkthrough

The example — `examples/ex07_texture_mapping_bilinear.rs` — is nearly identical
to the nearest neighbor example, except it uses the bilinear command:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex07_texture_mapping_bilinear::DrawTextureMappedTriangleBilinearCommand
    },
    geometry::{Point},
};

const SWIBORG_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/swiborg.png");

const POSITIONS: &[Point] = &[
    Point { x: 154, y: 460 },
    Point { x: 90,  y: 68  },
    Point { x: 486, y: 20  },
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

    let mut texture = vec![
        Color24 { r: 0, g: 0, b: 0 };
        image.width() as usize * image.height() as usize
    ];
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
            &DrawTextureMappedTriangleBilinearCommand {
                positions: POSITIONS,
                uv_coords: UV_COORDS,
                indices,
                texture: &texture,
                texture_width: image.width() as u16,
                texture_height: image.height() as u16
            }
        );
    }

    buffer.print_as_ppm();
}
```

The only difference from the nearest neighbor example is the command name:
`DrawTextureMappedTriangleBilinearCommand` instead of
`DrawTextureMappedTriangleNearestCommand`. The vertex data, UV coordinates,
texture loading, and drawing loop are identical — only the filtering
algorithm changes.

This is the beauty of the pixel command pattern: the rendering pipeline stays
the same, and only the per-pixel logic is swapped out.

---

## Visual Comparison

The cropped UV coordinates `(0.25, 0.25)`–`(0.75, 0.75)` help make the
difference visible:

- **Nearest neighbor:** The texture appears sharp but blocky. You can clearly
  see the individual texels as squares, especially along diagonal edges in the
  texture.
- **Bilinear:** The texture appears smooth and continuous. Edges are softened,
  and there are no visible texel boundaries. However, it may look slightly
  blurry compared to the crisp nearest neighbor result.

---

## How to Run the Example

```sh
cargo run --example ex07_texture_mapping_bilinear > pictures/ex07_texture_mapping_bilinear.ppm
```

Or build and run separately:

```sh
cargo build --release --example ex07_texture_mapping_bilinear
./target/release/examples/ex07_texture_mapping_bilinear > pictures/ex07_texture_mapping_bilinear.ppm
```

Open `pictures/ex07_texture_mapping_bilinear.ppm` in any image viewer. Compare
it with `pictures/ex06_texture_mapping_nearest.ppm` from the previous lesson.
The bilinear version should look smoother — especially on diagonal features
and edges in the swiborg texture.

---

## When to Use Each Filter

| Filter | Best for | Avoid when |
|--------|----------|------------|
| **Nearest neighbor** | Pixel art, retro games, diagrams, any image where crispness matters | Photographs, smooth gradients, detailed textures viewed up close |
| **Bilinear** | Photographs, smooth surfaces, general 3D rendering | Pixel art (it blurs the sharp edges), textures viewed at extreme angles |

Modern GPUs support multiple filtering modes and even **mipmapping** (using
progressively smaller versions of the texture for distant objects), which
combines bilinear filtering with level-of-detail selection. But the core
idea — averaging the four nearest texels — is the foundation of all texture
filtering.

---

## Summary

In this lesson we learned about:

- **Bilinear interpolation** — blending between the four texels surrounding a
  sampling point, first horizontally then vertically.
- **Fractional UV coordinates** — separating the integer part (texel index)
  from the fractional part (blend weight).
- **The 2×2 texel quad** — looking up four surrounding texels and blending
  them with `lerp`.
- **`Color24::lerp`** — per-channel linear interpolation between two colors,
  reused from Lesson 3.
- **Nearest neighbor vs bilinear** — sharp and blocky vs smooth and soft,
  each with its own use cases.
- **Filtering as a swap-in** — the pixel command pattern lets us change the
  filtering algorithm without touching the rendering pipeline.

In the next lesson we'll combine texture mapping with 3D — loading an OBJ
model, projecting its vertices onto the screen, and drawing a textured 3D
object.

---

## Exercises

### Exercise 1: Compare side by side

Modify the example to draw both the nearest neighbor and bilinear versions in
the same image — for example, left half with nearest, right half with bilinear.
You'll need two commands and a way to split the screen. Hint: modify
`draw_pixel` to check the `x` coordinate.

### Exercise 2: Extreme close-up

Change the UV coordinates to sample a very small region of the texture —
for example, `(0.45, 0.45)` to `(0.55, 0.55)`. This effectively zooms into
the texture. Compare how nearest neighbor (from lesson 6) and bilinear handle
the zoom. Which one looks better?

### Exercise 3: Extreme stretch

Change the screen positions to make the quad very large (e.g., 600×400 pixels)
while keeping the same cropped UV coordinates. The texture will be severely
stretched. How do the two filtering methods compare when each screen pixel
maps to a fraction of a texel? What about when many screen pixels map to the
same texel?

### Exercise 4: Implement bicubic filtering

**Advanced.** Bilinear blends 4 texels. Bicubic blends 16 texels (a 4×4 grid)
using cubic polynomials for even smoother results. If you're up for a
challenge, try implementing a `DrawTextureMappedTriangleBicubicCommand`. The
`Color24::lerp` method can still be used — just apply it multiple times to
simulate a cubic curve (Catmull-Rom or Hermite interpolation).