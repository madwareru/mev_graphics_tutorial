# Lesson 8: Drawing a Simple Unlit 3D Model

> **Result:** `pictures/ex08_drawing_simple_unlit_3d_model.ppm`
>
> In this lesson we finally step into **3D**. We load an actual 3D model (a
> dragon) from an OBJ file, apply model-view-projection transformations,
> rasterize its triangles with a depth buffer, and texture-map them using
> bilinear filtering — all rendered with a programmable **vertex/pixel shader**
> pipeline running entirely in software.

---

## What We Are Doing

Up until now we've been drawing 2D triangles and quads with hand-placed
screen coordinates. Real 3D graphics works differently:

1. A **3D model** is stored as a collection of vertices (positions, texture
   coordinates, normals) and triangle indices.
2. Each vertex goes through a **vertex shader** that transforms it from
   model space → world space → view space → clip space using matrices.
3. The triangles are **clipped and rasterized** to screen pixels.
4. For each pixel, a **pixel shader** (fragment shader) computes the final
   color — in our case, by sampling a texture.

This lesson implements this entire pipeline in software: the `draw_obj_model`
method in `src/software_buffer/ex08_drawing_simple_unlit_3d_model.rs`, a
vertex/pixel shader trait system, and a depth buffer for correct visibility.

---

## The OBJ Model Format

The `obj_loader` module parses the Wavefront OBJ format, a widely-used plain
text format for 3D models. Our loader handles three types of data:

| OBJ keyword | Data               | Stored as                        |
|-------------|--------------------|----------------------------------|
| `v`         | Vertex position    | `glam::Vec4` (x, y, z, w = 1.0) |
| `vt`        | Texture coordinate | `glam::Vec2` (u, v)              |
| `vn`        | Vertex normal      | `glam::Vec4` (x, y, z, w = 0.0) |
| `f`         | Face (triangle)    | Three index triplets `v/vt/vn`   |

Each face line like `f 1/1/1 2/2/2 3/3/3` defines one triangle by referencing
three index triplets. Each triplet specifies which vertex position, texture
coordinate, and normal to use — allowing the same position to have different
texture coordinates on different faces (important for UV seams).

The `ObjModel::load_from_string` method deduplicates these triplets using a
`HashMap`, producing flat arrays with 0-based indices:

```rust
pub fn vs(&self) -> &[glam::Vec4]    // vertex positions
pub fn vts(&self) -> &[glam::Vec2]   // texture coordinates
pub fn vns(&self) -> &[glam::Vec4]   // normals
pub fn indices(&self) -> &[[u16; 3]] // triangle indices
```

> **Why `w = 1.0` for positions and `w = 0.0` for normals?** When a position
> vector is multiplied by a 4×4 transformation matrix, the `w = 1.0` component
> enables **translation** — the position can move in space. For normals,
> `w = 0.0` disables translation so that normals only rotate and scale
> (direction vectors should not be moved).

---

## The Vertex and Pixel Shader Traits

This lesson introduces a **programmable shader pipeline** inspired by real
GPUs. Instead of a single `PixelDrawingCommand`, we now have two stages:

### `VertexShaderData` — the per-vertex input

```rust
#[derive(Copy, Clone)]
pub struct VertexShaderData {
    pub position: glam::Vec4,
    pub tex_coord: glam::Vec2,
    pub normal: glam::Vec4,
}
```

This bundles all attributes of a single vertex: its position in clip space,
texture coordinate, and normal.

### `InterpolatedByBarycentric` — the interpolated output

```rust
pub trait InterpolatedByBarycentric: Copy {
    fn interpolate_by_barycentric(items: [Self; 3], barycentric_coords: [f32; 3]) -> Self;
    fn get_position(&self) -> glam::Vec4;
}
```

Any type implementing this trait can be produced by the vertex shader,
interpolated across the triangle using barycentric coordinates, and consumed
by the pixel shader. The built-in implementation for `VertexShaderData`
interpolates all three fields (position, tex_coord, normal) independently.

### `VertexShader` trait

```rust
pub trait VertexShader {
    type Output: InterpolatedByBarycentric;
    fn transform_vertices(&self, input: VertexShaderData) -> Self::Output;
}
```

The vertex shader takes raw per-vertex data and transforms it. In our unlit
shader it simply applies the MVP matrix:

```rust
impl<'a> VertexShader for DrawUnlitObjModelShader<'a> {
    type Output = VertexShaderData;
    fn transform_vertices(&self, input: VertexShaderData) -> VertexShaderData {
        let position = self.model_view_projection_matrix * input.position;
        VertexShaderData { position, ..input }
    }
}
```

### `PixelShader` trait

```rust
pub trait PixelShader {
    type Input: InterpolatedByBarycentric;
    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        depth_texture: &mut [f32],
        vertex_input: Self::Input,
        fragment_x: u16,
        fragment_y: u16
    );
}
```

The pixel shader receives the interpolated vertex data for a specific screen
pixel, checks the depth buffer, and writes the final color.

---

## The Depth Buffer (Z-Buffer)

When drawing a 3D model, triangles can overlap. We need to know which
triangle is **in front** — closest to the camera. This is done with a
**depth buffer** (also called z-buffer): an array of floats, one per pixel,
that stores the depth of the nearest fragment drawn so far.

```rust
let mut depth_texture = vec![0.0; buffer.get_width() as usize * buffer.get_height() as usize];
```

In the pixel shader:

```rust
let fragment_index = (fragment_y as usize) * software_buffer.get_width() as usize
    + (fragment_x as usize);
if depth_texture[fragment_index] > vertex_input.position.z { return; }
depth_texture[fragment_index] = vertex_input.position.z;
```

- If the new fragment's `z` is **greater** than the stored depth, it's
  farther away — discard it.
- If it's **smaller** (closer), overwrite the depth and draw the pixel.
- We initialize the buffer to `0.0` (the far plane in our NDC convention),
  so any visible fragment (with `z < 0.0`) will pass.

> **Why greater z = farther?** Our projection matrix maps the near plane to
> `z = -1` and the far plane to `z = 1` in NDC. But after perspective divide
> and interpolation, the z values that survive into the pixel shader are in
> the range `[-1, 1]` where `-1` is closest and `1` is farthest. Since we
> initialize to `0.0`, fragments with `z < 0.0` (closer than mid-distance)
> pass the test. We'll refine this in later lessons.

---

## The 3D Drawing Pipeline

The `draw_obj_model` method implements the full pipeline:

```rust
impl SoftwareBuffer {
    pub fn draw_obj_model<TData, TShader>(
        &mut self,
        obj_model: &ObjModel,
        depth_texture: &mut [f32],
        shader: &TShader,
    ) where
        TData: InterpolatedByBarycentric,
        TShader: VertexShader<Output = TData> + PixelShader<Input = TData>,
    {
        assert_eq!(depth_texture.len(), self.width as usize * self.height as usize);
        let center_x = self.get_width() as f32 / 2.0;
        let center_y = self.get_height() as f32 / 2.0;

        for tri in obj_model.indices() {
            // --- Vertex Stage ---
            let vertices = tri.map(|id| {
                let position = obj_model.vs()[id as usize];
                let tex_coord = obj_model.vts()[id as usize];
                let normal = obj_model.vns()[id as usize];
                shader.transform_vertices(VertexShaderData { position, tex_coord, normal })
            });

            // --- Clipping Stage (NDC to screen space) ---
            let triangle = Triangle::new(
                Point {
                    x: (center_x * (vertices[0].get_position().x + 1.0)).round() as i16,
                    y: (center_y * (1.0 - vertices[0].get_position().y)).round() as i16,
                },
                Point {
                    x: (center_x * (vertices[1].get_position().x + 1.0)).round() as i16,
                    y: (center_y * (1.0 - vertices[1].get_position().y)).round() as i16,
                },
                Point {
                    x: (center_x * (vertices[2].get_position().x + 1.0)).round() as i16,
                    y: (center_y * (1.0 - vertices[2].get_position().y)).round() as i16,
                }
            );

            let min_x = triangle.a.x.min(triangle.b.x).min(triangle.c.x).max(0);
            let max_x = triangle.a.x.max(triangle.b.x).max(triangle.c.x).min((self.width - 1) as _);
            let min_y = triangle.a.y.min(triangle.b.y).min(triangle.c.y).max(0);
            let max_y = triangle.a.y.max(triangle.b.y).max(triangle.c.y).min((self.height - 1) as _);

            // --- Pixel Stage ---
            for y in min_y..=max_y {
                for x in (min_x..=max_x).filter(|&x| triangle.winding_n(min_x..=x, y) % 2 != 0) {
                    let barycentric_coords = Point { x, y }.calculate_barycentric_in(
                        [triangle.a, triangle.b, triangle.c]
                    );
                    let vertex_input = TData::interpolate_by_barycentric(vertices, barycentric_coords);
                    shader.draw_pixel(self, depth_texture, vertex_input, x as _, y as _);
                }
            }
        }
    }
}
```

Let's break it down step by step.

### Step 1: Compute screen center

```rust
let center_x = self.get_width() as f32 / 2.0;
let center_y = self.get_height() as f32 / 2.0;
```

After the vertex shader transforms vertices into **clip space**, the
coordinates are in NDC (Normalized Device Coordinates) where `x` and `y`
range from `-1` (left/bottom) to `+1` (right/top). We need to map this to
screen pixel coordinates.

### Step 2: The vertex stage (per triangle)

```rust
let vertices = tri.map(|id| {
    let position = obj_model.vs()[id as usize];
    let tex_coord = obj_model.vts()[id as usize];
    let normal = obj_model.vns()[id as usize];
    shader.transform_vertices(VertexShaderData { position, tex_coord, normal })
});
```

For each triangle in the model, we look up the three vertex indices and
gather the vertex data (position, UV, normal) from the model arrays. Then
we pass each vertex through the vertex shader, which applies the MVP matrix.

### Step 3: NDC to screen space conversion

```rust
Point {
    x: (center_x * (vertices[i].get_position().x + 1.0)).round() as i16,
    y: (center_y * (1.0 - vertices[i].get_position().y)).round() as i16,
}
```

The vertex shader outputs positions in **clip space** (a `glam::Vec4`). Since
we use an orthographic-like perspective projection, the `w` component is
already handled by `glam`, and the `x, y, z` values are effectively in NDC
after the built-in perspective divide that `glam` applies in its projection
matrix.

The mapping from NDC to screen space:

- **X:** `x_ndc` ranges from `-1` (left edge) to `+1` (right edge). We map
  it to `[0, width)` with `center_x * (x_ndc + 1.0)`.
- **Y:** NDC has Y pointing up (`-1` = bottom, `+1` = top), but screen Y
  points down. We invert with `center_y * (1.0 - y_ndc)`.

This is the software equivalent of the GPU's **viewport transform**.

### Step 4: Compute bounding box

```rust
let min_x = triangle.a.x.min(triangle.b.x).min(triangle.c.x).max(0);
let max_x = triangle.a.x.max(triangle.b.x).max(triangle.c.x).min((self.width - 1) as _);
let min_y = triangle.a.y.min(triangle.b.y).min(triangle.c.y).max(0);
let max_y = triangle.a.y.max(triangle.b.y).max(triangle.c.y).min((self.height - 1) as _);
```

We compute the axis-aligned bounding box of the triangle in screen space and
clamp it to the buffer dimensions. This is our rasterization region.

### Step 5: The pixel stage

```rust
for y in min_y..=max_y {
    for x in (min_x..=max_x).filter(|&x| triangle.winding_n(min_x..=x, y) % 2 != 0) {
        let barycentric_coords = Point { x, y }
            .calculate_barycentric_in([triangle.a, triangle.b, triangle.c]);
        let vertex_input = TData::interpolate_by_barycentric(
            vertices, barycentric_coords
        );
        shader.draw_pixel(self, depth_texture, vertex_input, x as _, y as _);
    }
}
```

For each pixel inside the triangle (using the winding number test from
lesson 2):

1. **Compute barycentric coordinates** for this pixel relative to the
   triangle's screen-space vertices.
2. **Interpolate vertex data** — position, UV, and normal are all blended
   using the barycentric weights.
3. **Execute the pixel shader** — which checks the depth buffer and samples
   the texture.

> **A note on perspective-correct interpolation:** Real 3D graphics uses
> perspective-correct interpolation (dividing by `w` before interpolating).
> In this first 3D lesson we use simple barycentric interpolation in screen
> space, which works reasonably well for models with mild perspective. We'll
> improve this in later lessons.

---

## The Unlit Pixel Shader

The `DrawUnlitObjModelShader` implements both `VertexShader` and `PixelShader`.
Its pixel shader does bilinear texture sampling with depth testing:

```rust
fn draw_pixel(
    &self,
    software_buffer: &mut SoftwareBuffer,
    depth_texture: &mut [f32],
    vertex_input: VertexShaderData,
    fragment_x: u16,
    fragment_y: u16,
) {
    let fragment_index = (fragment_y as usize) * software_buffer.get_width() as usize
        + (fragment_x as usize);
    if depth_texture[fragment_index] > vertex_input.position.z { return; }
    depth_texture[fragment_index] = vertex_input.position.z;

    let (u, v) = (vertex_input.tex_coord.x, vertex_input.tex_coord.y);
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

    software_buffer.set_pixel(fragment_x, fragment_y, color0.lerp(color1, v_frac));
}
```

The depth test and bilinear sampling are the same techniques we've already
seen — now combined into a single shader that runs for every pixel of every
triangle of the 3D model.

---

## The Model-View-Projection Matrix

The vertex shader transforms vertices using a single combined matrix:

```rust
let model_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.0))
    * glam::Mat4::from_scale(glam::Vec3::new(2.0, 2.0, 2.0));

let view_matrix = glam::camera::rh::view::look_at_mat4(
    glam::Vec3::new(450.0, -500.0, -500.0),  // camera position
    glam::Vec3::new(0.0, 0.0, 0.0),           // look-at target
    glam::Vec3::new(0.0, 1.0, 0.0)            // up vector
);

let projection_matrix = glam::camera::rh::proj::opengl::perspective(
    std::f32::consts::FRAC_PI_2,              // field of view (90°)
    buffer.get_width() as f32 / buffer.get_height() as f32, // aspect ratio
    0.1,                                       // near plane
    1000.0                                     // far plane
);

let model_view_projection_matrix = projection_matrix * view_matrix * model_matrix;
```

These three matrices work together:

| Matrix | Purpose | Space |
|--------|---------|-------|
| **Model** | Positions, rotates, and scales the model | Model → World |
| **View** | Places the camera | World → View (eye) |
| **Projection** | Applies perspective (foreshortening) | View → Clip (NDC) |

We multiply them in **reverse order** (projection × view × model) so that
a vertex goes through model → world → view → clip in one matrix-vector
multiplication.

The `glam` library provides convenient constructors:
- `look_at_mat4` — creates a view matrix from camera position, target, and up.
- `perspective` — creates a perspective projection matrix with the given
  field of view, aspect ratio, and near/far planes.

> **Why right-handed?** `glam::camera::rh` creates right-handed coordinate
> systems, which is the OpenGL convention and is common in graphics tutorials.
> In a right-handed system, the camera looks down the **negative Z** axis.

---

## Example Walkthrough

Now let's look at the full example — `examples/ex08_drawing_simple_unlit_3d_model.rs`:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex08_drawing_simple_unlit_3d_model::DrawUnlitObjModelShader
    },
    obj_loader::ObjModel,
};

const DRAGON_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/dragon.png");
const DRAGON_MODEL_TEXT: &str = include_str!("../assets/dragon.obj");

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });
    let mut depth_texture = vec![
        0.0;
        buffer.get_width() as usize * buffer.get_height() as usize
    ];

    // Load texture
    let image = image::load_from_memory(DRAGON_TEXTURE_BYTES)
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

    // Load the 3D model
    let dragon_model = ObjModel::load_from_string(DRAGON_MODEL_TEXT).unwrap();

    // Set up transformation matrices
    let model_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.0))
        * glam::Mat4::from_scale(glam::Vec3::new(2.0, 2.0, 2.0));

    let view_matrix = glam::camera::rh::view::look_at_mat4(
        glam::Vec3::new(450.0, -500.0, -500.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        glam::Vec3::new(0.0, 1.0, 0.0)
    );

    let projection_matrix = glam::camera::rh::proj::opengl::perspective(
        std::f32::consts::FRAC_PI_2,
        buffer.get_width() as f32 / buffer.get_height() as f32,
        0.1,
        1000.0
    );

    let model_view_projection_matrix = projection_matrix * view_matrix * model_matrix;

    // Draw the model
    buffer.draw_obj_model(
        &dragon_model,
        &mut depth_texture,
        &DrawUnlitObjModelShader {
            model_view_projection_matrix,
            texture: &texture,
            texture_width: image.width() as u16,
            texture_height: image.height() as u16
        }
    );

    buffer.print_as_ppm();
}
```

### Step 1: Load the texture

Same as in the previous lessons — embed `dragon.png`, decode it with the
`image` crate, and convert to `Vec<Color24>`.

### Step 2: Load the 3D model

```rust
let dragon_model = ObjModel::load_from_string(DRAGON_MODEL_TEXT).unwrap();
```

The OBJ file `assets/dragon.obj` is embedded with `include_str!` and parsed
into an `ObjModel` with all vertices, UVs, normals, and triangle indices.

### Step 3: Set up matrices

```rust
let model_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.0))
    * glam::Mat4::from_scale(glam::Vec3::new(2.0, 2.0, 2.0));
```

The model matrix first scales the dragon by 2× in all axes, then translates
it (by zero — no movement, just the scale). The translation is written first
but applied last: the combined matrix does `translate(scale(v))`.

```rust
let view_matrix = glam::camera::rh::view::look_at_mat4(
    glam::Vec3::new(450.0, -500.0, -500.0),
    glam::Vec3::new(0.0, 0.0, 0.0),
    glam::Vec3::new(0.0, 1.0, 0.0)
);
```

The camera is positioned at `(450, -500, -500)` looking at the origin.
The Y coordinate of the camera is negative (`-500`), placing the camera
**above** the model (in our scene's up direction), looking downward.

```rust
let projection_matrix = glam::camera::rh::proj::opengl::perspective(
    std::f32::consts::FRAC_PI_2,  // 90° field of view
    buffer.get_width() as f32 / buffer.get_height() as f32,  // 4:3 aspect
    0.1,     // near plane at z = 0.1
    1000.0   // far plane at z = 1000.0
);
```

A 90° vertical field of view gives a wide-angle perspective.

### Step 4: Draw the model

```rust
buffer.draw_obj_model(
    &dragon_model,
    &mut depth_texture,
    &DrawUnlitObjModelShader {
        model_view_projection_matrix,
        texture: &texture,
        texture_width: image.width() as u16,
        texture_height: image.height() as u16
    }
);
```

The shader owns the combined MVP matrix and the texture data. It transforms
every vertex and texture-samples every pixel of every triangle.

### Step 5: Output

```rust
buffer.print_as_ppm();
```

---

## How to Run the Example

```sh
cargo run --example ex08_drawing_simple_unlit_3d_model > pictures/ex08_drawing_simple_unlit_3d_model.ppm
```

Or build and run separately:

```sh
cargo build --release --example ex08_drawing_simple_unlit_3d_model
./target/release/examples/ex08_drawing_simple_unlit_3d_model > pictures/ex08_drawing_simple_unlit_3d_model.ppm
```

Open `pictures/ex08_drawing_simple_unlit_3d_model.ppm` in any image viewer.
You should see a 3D dragon model rendered with its texture, viewed from a
slightly elevated angle.

---

## Summary

In this lesson we learned about:

- **OBJ model loading** — parsing vertex positions, texture coordinates,
  normals, and triangle indices from a Wavefront OBJ file.
- **The vertex/pixel shader pattern** — a two-stage programmable pipeline
  where the vertex shader transforms vertices and the pixel shader computes
  per-pixel colors.
- **Model-View-Projection matrices** — combining model, view, and projection
  transforms into a single MVP matrix.
- **NDC to screen space** — mapping the `[-1, 1]` NDC range to pixel
  coordinates, with Y-axis inversion.
- **The depth buffer (z-buffer)** — storing per-pixel depth to resolve
  visibility when triangles overlap.
- **Bilinear texture sampling on a 3D model** — reusing the filtering
  technique from lesson 7 across thousands of triangles.

In the next lesson we'll visualize the normals of the dragon model to build
intuition before moving on to lighting.

---

## Exercises

### Exercise 1: Rotate the model

Add a rotation around the Y axis to the model matrix. Use
`glam::Mat4::from_rotation_y(angle)` where `angle` varies over time. You can
make `angle` a constant for a single frame, or try different angles to see the
dragon from all sides. The structure is:
```rust
let model_matrix = glam::Mat4::from_translation(...)
    * glam::Mat4::from_rotation_y(0.5)  // rotate 0.5 radians
    * glam::Mat4::from_scale(...);
```

### Exercise 2: Move the camera

Change the camera position in `look_at_mat4`. Try moving closer to the dragon,
farther away, or circling around it by varying the X and Z coordinates. What
happens when the camera goes inside the model?

### Exercise 3: Change the field of view

Adjust the field of view in the perspective matrix. Try a narrow FOV
(e.g., `FRAC_PI_6` = 30°) for a more "zoomed in" look, or a very wide FOV
(e.g., `FRAC_PI_2 * 1.5` = 135°) for a fisheye effect. How does the
perspective distortion change?

### Exercise 4: Disable the depth buffer

Set `depth_texture` to `None` or skip the depth test in the pixel shader.
How does the image look without depth testing? Which triangles appear on top?
This demonstrates why the z-buffer is essential for correct 3D rendering.

### Exercise 5: Count the triangles

Add a counter to `draw_obj_model` that prints the number of triangles drawn.
How many triangles does the dragon model have? This gives you a sense of the
complexity of real 3D models versus our simple quad examples.

### Exercise 6: Back face culling

Right now we rasterize **every** triangle in the model, including those on
the far side of the dragon that face **away** from the camera. These
back-facing triangles are always overwritten by closer, front-facing ones
thanks to the depth buffer — but we're still wasting time iterating over
their pixels.

**Back face culling** skips triangles that face away from the viewer before
rasterization even begins. The key insight is the **winding order** of the
triangle's vertices in screen space: if the three vertices go clockwise (when
viewed from the camera), the triangle is facing away; if counter-clockwise,
it's facing toward the camera (or vice versa, depending on the convention).

To determine the winding direction, compute the **signed area** of the
triangle in screen space using the 2D cross product (the same technique from
lesson 4):

```rust
let edge1 = glam::vec2(
    triangle.b.x as f32 - triangle.a.x as f32,
    triangle.b.y as f32 - triangle.a.y as f32,
);
let edge2 = glam::vec2(
    triangle.c.x as f32 - triangle.a.x as f32,
    triangle.c.y as f32 - triangle.a.y as f32,
);
let signed_area = edge1.x * edge2.y - edge1.y * edge2.x;
```

If `signed_area` has the "wrong" sign (e.g., positive when your convention
expects negative), the triangle is back-facing — skip it entirely.

Modify `draw_obj_model` to add a `back_face_culling: bool` parameter. When
enabled, check the signed area after the NDC-to-screen conversion and
`continue` to the next triangle if it's back-facing. Compare the rendered
image with and without culling — it should look the same, but the program
should run faster. How many triangles are culled? Add a counter to find out.

> **Why does winding order matter?** 3D models define each triangle with a
> specific vertex order (clockwise or counter-clockwise) that indicates which
> side is the "front." This convention is baked into the OBJ file. When the
> triangle is projected to screen space, a front-facing triangle preserves
> its winding direction, while a back-facing triangle's winding direction
> flips — that's how we can tell them apart using the sign of the signed area.