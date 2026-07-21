# Lesson 5: UV Coordinates

> **Result:** `pictures/ex05_uv_coordinates.ppm`
>
> In this lesson we will learn about **UV coordinates** — a way to map each
> vertex of a triangle to a point in a 2D parameter space. By interpolating
> these coordinates across the triangle using barycentric weights, we can
> assign a unique `(u, v)` pair to every pixel. We'll visualize them as
> colors to build intuition before applying real textures in the next lessons.

---

## What We Are Doing

In the previous lesson we used barycentric coordinates to interpolate colors
across a triangle. Now we'll interpolate something different: **UV
coordinates**.

UV coordinates are pairs `(u, v)` that map a point on the triangle to a point
in a 2D space, typically ranging from `0.0` to `1.0`. Think of them as
"addressing" a position on a flat square — `u` is the horizontal axis and `v`
is the vertical axis. When we later load a texture (an image), the UV
coordinates tell us which texel to sample for each pixel.

In this lesson we don't have a texture yet. Instead, we'll **visualize** the
UV coordinates directly: `u` maps to the red channel, `v` maps to the green
channel, and blue stays at 0. This produces a colorful gradient that lets us
see exactly how UV coordinates vary across the triangle's surface.

---

## The Concept of UV Coordinates

### What are UV coordinates?

Every vertex of a triangle can have a UV coordinate assigned to it. The
coordinate `(0, 0)` typically corresponds to one corner of the texture,
and `(1, 1)` to the opposite corner:

```
(0,0) ----u----> (1,0)
  |               |
  v               v
  |               |
(0,1) ----u----> (1,1)
```

When we draw a triangle, each pixel inside it gets an interpolated UV
coordinate — a blend of the three vertex UVs, weighted by barycentric
coordinates. This interpolated UV tells us where in the texture to look up
the color for that pixel.

### Why "UV"?

The letters `U` and `V` are used (instead of `X` and `Y`) to avoid confusion
with the spatial coordinates of the vertices. `X` and `Y` refer to positions
on the screen; `U` and `V` refer to positions in the texture space.

### UV coordinates and quads

A single triangle has three vertices, but many shapes (like rectangles) are
built from **two triangles** forming a quad. Each of the four corners of the
quad gets a UV coordinate, and the two triangles share vertices:

```
Quad:                    UV space:

  v0 ------ v3           (0,0) ---- (1,0)
   | \      |              | \       |
   |  \     |              |  \      |
   |   \    |              |   \     |
   |    \   |              |    \    |
   |     \  |              |     \   |
  v1 ------ v2           (0,1) ---- (1,1)

Triangle 1: v0, v1, v2
Triangle 2: v0, v2, v3
```

The quad is split along the diagonal from `v0` to `v2`. Both triangles share
the edge `v0–v2`, ensuring the UV coordinates are continuous across the
boundary.

---

## Indexed Geometry

This lesson introduces a new concept: **indexed geometry**. Instead of
specifying vertex positions separately for each triangle, we define a shared
list of vertices and reference them by index.

```rust
const POSITIONS: &[Point] = &[
    Point { x: 154, y: 460 },  // vertex 0: bottom-left
    Point { x: 90,  y: 68  },  // vertex 1: top-left
    Point { x: 486, y: 20  },  // vertex 2: top-right
    Point { x: 550, y: 412 },  // vertex 3: bottom-right
];
const UV_COORDS: &[(f32, f32)] = &[
    (0.0, 0.0),  // vertex 0
    (0.0, 1.0),  // vertex 1
    (1.0, 1.0),  // vertex 2
    (1.0, 0.0),  // vertex 3
];
const INDICES: [[u16; 3]; 2] = [
    [0, 1, 2],  // triangle 1: vertices 0, 1, 2
    [0, 2, 3],  // triangle 2: vertices 0, 2, 3
];
```

- **`POSITIONS`** — the four corners of the quad in screen space.
- **`UV_COORDS`** — the UV coordinate assigned to each corner. The mapping
  follows the standard convention: bottom-left is `(0, 0)`, top-left is
  `(0, 1)`, top-right is `(1, 1)`, bottom-right is `(1, 0)`.
- **`INDICES`** — two triangles, each defined by three vertex indices. The
  indices reference into the `POSITIONS` and `UV_COORDS` arrays.

> **Why indexed geometry?** In real 3D models, vertices are shared between
> many triangles. Storing each vertex once and referencing it by index saves
> memory and ensures consistency. A vertex carries all its attributes
> (position, UV, normal, color) together, and the index buffer defines how
> vertices are connected into triangles. This is exactly how GPU vertex and
> index buffers work.

---

## The Drawing Command

The `DrawUVTriangleCommand` in `src/software_buffer/ex05_uv_coordinates.rs`
interpolates UV coordinates and visualizes them as colors:

```rust
pub struct DrawUVTriangleCommand<'a> {
    pub positions: &'a [Point],
    pub uv_coords: &'a [(f32, f32)],
    pub indices: [u16; 3],
}

impl<'a> PixelDrawingCommand for DrawUVTriangleCommand<'a> {
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

        let color = Color24 {
            r: (u * 255.0).round().clamp(0.0, 255.0) as u8,
            g: (v * 255.0).round().clamp(0.0, 255.0) as u8,
            b: 0
        };

        software_buffer.set_pixel(x, y, color);
    }
}
```

Let's break it down step by step.

### Step 1: Look up vertex data by index

```rust
let indices = self.indices.map(|it| it as usize);
let positions = indices.map(|id| self.positions[id]);
let uv_coords = indices.map(|id| self.uv_coords[id]);
```

The command receives the full vertex arrays and the three indices for this
triangle. It looks up the three positions and three UV coordinates that
belong to this specific triangle.

### Step 2: Compute barycentric coordinates

```rust
let point = Point { x: x as _, y: y as _ };
let barycentric_coords = point.calculate_barycentric_in(positions);
```

Same as in the previous lesson: compute the barycentric weights `(α, β, γ)`
for the current pixel relative to the triangle's three vertices.

### Step 3: Interpolate UV coordinates

```rust
let (u, v) = mix_2_components_by_barycentric(uv_coords, barycentric_coords);
```

Using `mix_2_components_by_barycentric` from the previous lesson, we
interpolate the UV coordinates. The result is:

```
u = α·u0 + β·u1 + γ·u2
v = α·v0 + β·v1 + γ·v2
```

### Step 4: Visualize as color

```rust
let color = Color24 {
    r: (u * 255.0).round().clamp(0.0, 255.0) as u8,
    g: (v * 255.0).round().clamp(0.0, 255.0) as u8,
    b: 0
};
```

We map `u` to the red channel and `v` to the green channel (both scaled from
`[0, 1]` to `[0, 255]`). Blue is set to 0. This produces a visual
representation of the UV space:

- `(0, 0)` → black (no red, no green)
- `(1, 0)` → pure red
- `(0, 1)` → pure green
- `(1, 1)` → yellow (full red + full green)

---

## Example Walkthrough

Now let's look at the full example — `examples/ex05_uv_coordinates.rs`:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex05_uv_coordinates::DrawUVTriangleCommand
    },
    geometry::{Point},
};

const POSITIONS: &[Point] = &[
    Point { x: 154, y: 460 },
    Point { x: 90,  y: 68  },
    Point { x: 486, y: 20  },
    Point { x: 550, y: 412 },
];
const UV_COORDS: &[(f32, f32)] = &[
    (0.0, 0.0),
    (0.0, 1.0),
    (1.0, 1.0),
    (1.0, 0.0)
];
const INDICES: [[u16; 3]; 2] = [
    [0, 1, 2],
    [0, 2, 3]
];

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });
    for indices in INDICES.iter().copied() {
        buffer.draw_triangle(
            indices.map(|id| POSITIONS[id as usize]).into(),
            &DrawUVTriangleCommand {
                positions: POSITIONS,
                uv_coords: UV_COORDS,
                indices
            }
        );
    }
    buffer.print_as_ppm();
}
```

### Step 1: Define the quad

The quad is defined by four vertices in screen space. Note that the quad is
not a perfect rectangle — it's a slightly distorted quadrilateral. This is
intentional: it shows that UV coordinates interpolate correctly even when
the shape on screen doesn't match the texture's rectangular shape.

### Step 2: Assign UV coordinates

Each vertex gets a UV coordinate mapping it to a corner of the unit square:

| Vertex | Screen position | UV coordinate | Visualized color |
|--------|----------------|---------------|-----------------|
| 0      | bottom-left    | `(0, 0)`      | black           |
| 1      | top-left       | `(0, 1)`      | green           |
| 2      | top-right      | `(1, 1)`      | yellow          |
| 3      | bottom-right   | `(1, 0)`      | red             |

### Step 3: Draw both triangles

```rust
for indices in INDICES.iter().copied() {
    buffer.draw_triangle(
        indices.map(|id| POSITIONS[id as usize]).into(),
        &DrawUVTriangleCommand {
            positions: POSITIONS,
            uv_coords: UV_COORDS,
            indices
        }
    );
}
```

We iterate over the two triangles defined by `INDICES`. For each triangle,
we create a `DrawUVTriangleCommand` that knows the full vertex arrays and
which three indices belong to this triangle. The `draw_triangle` method
(from lesson 2) handles the winding number test and calls our command for
each pixel inside.

### Step 4: Output

```rust
buffer.print_as_ppm();
```

---

## How to Run the Example

```sh
cargo run --example ex05_uv_coordinates > pictures/ex05_uv_coordinates.ppm
```

Or build and run separately:

```sh
cargo build --release --example ex05_uv_coordinates
./target/release/examples/ex05_uv_coordinates > pictures/ex05_uv_coordinates.ppm
```

Open `pictures/ex05_uv_coordinates.ppm` in any image viewer. You should see
a distorted quad filled with a smooth gradient: black in the bottom-left,
green in the top-left, yellow in the top-right, and red in the bottom-right.
The diagonal seam between the two triangles should be invisible — the UV
coordinates are continuous across it.

---

## The Seam Between Triangles

An important detail: the two triangles share the edge from vertex 0 to
vertex 2. Along this shared edge, both triangles compute the same barycentric
interpolation, so the UV coordinates (and thus the colors) match perfectly.
There is no visible seam.

This works because:
1. The shared vertices (0 and 2) have the same UV coordinates in both
   triangles.
2. Barycentric interpolation along an edge depends only on the two endpoints
   of that edge — the third vertex's weight is 0 along the edge.

If the shared vertices had different UV coordinates in each triangle, there
would be a visible discontinuity along the seam. This is why indexed geometry
is important: it guarantees that shared vertices have consistent attributes.

---

## Summary

In this lesson we learned about:

- **UV coordinates** — 2D pairs `(u, v)` that map vertices to positions in a
  texture space, typically in the range `[0, 1]`.
- **UV interpolation** — using barycentric coordinates to compute a unique
  `(u, v)` for every pixel inside a triangle.
- **Visualizing UVs as colors** — mapping `u` to red and `v` to green to see
  how coordinates vary across the surface.
- **Indexed geometry** — defining vertices once and referencing them by index
  to form triangles, sharing data efficiently.
- **Quads from two triangles** — splitting a quadrilateral along a diagonal,
  with shared vertices ensuring continuity.
- **Seamless interpolation** — why the shared edge between two triangles
  produces no visible discontinuity.

In the next lesson we'll replace the UV visualization with actual texture
sampling — looking up real pixel colors from an image file.

---

## Exercises

### Exercise 1: Swap UV axes

Swap the `u` and `v` values in the `UV_COORDS` array (e.g., change `(0.0, 0.0)`
to `(0.0, 0.0)`, `(0.0, 1.0)` to `(1.0, 0.0)`, etc.). How does the gradient
change? Can you predict the result before running it?

### Exercise 2: Repeat the texture

Change the UV coordinates to go beyond the `[0, 1]` range — for example, use
`(0.0, 0.0)`, `(0.0, 2.0)`, `(2.0, 2.0)`, `(2.0, 0.0)`. What happens to the
colors? Since we're visualizing UVs directly (not sampling a texture yet),
the colors will clamp at 255. But there are other ways to handle such cases:
- Repeat the uv by taking a fractional part of `u` and `v`.
- Mirror the uv by inverting fractional parts of `u` and `v` in each odd-numbered iteration.
- The modes could be combined separately, for example, clamp on `v`, but repeat or mirror on `u`
Try to implement drawing commands to see how UV looks with different wrapping modes. 
Try also wrap using negative values for UV coordinates.

### Exercise 3: Add additional triangles

Try to form more complex shapes by adding more vertices and triangles using
our new approach with indices. Can you draw a circle? How would you map UV
coordinates to the circle? What about a ring of quads? Can we draw a letter "Ы" 
from Lesson 3 with this approach?