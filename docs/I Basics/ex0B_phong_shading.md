# Lesson I.11: Phong Shading

> **Result:** `pictures/ex0B_phong_shading.ppm`
>
> In this lesson we upgrade from Gouraud to **Phong shading**. Instead of
> computing lighting per vertex and interpolating the resulting color, we
> interpolate the **normal vector** across the triangle and compute lighting
> **per pixel**. This produces more accurate shading — especially for
> specular highlights and sharp shadow boundaries that fall inside a single
> triangle.

---

## What We Are Doing

In the previous lesson we implemented **Gouraud shading**: the vertex shader
computed a per-vertex light intensity, and the pixel shader interpolated that
intensity across the triangle. This is efficient — lighting is evaluated only
3 times per triangle — but it has a fundamental limitation: any lighting
detail that falls **between** vertices is lost.

**Phong shading** (named after Bui Tuong Phong, 1973) takes a different
approach: the **normal** is interpolated across the triangle, and the lighting
calculation is performed **per pixel** in the pixel shader. This means every
pixel gets its own lighting evaluation based on a smoothly varying normal,
capturing highlights and shadow transitions that Gouraud would miss.

The trade-off is cost: instead of 3 lighting evaluations per triangle,
Phong performs one evaluation **per rasterized pixel** — potentially hundreds
or thousands per triangle. On modern GPUs this is trivially fast, but in our
software renderer it's noticeably more expensive.

---

## Gouraud vs Phong: The Key Difference

```
Gouraud shading:
  Vertex Shader          Pixel Shader
  ─────────────          ────────────
  compute light          interpolate color
  output: color          multiply texture × color

Phong shading:
  Vertex Shader          Pixel Shader
  ─────────────          ────────────
  transform normal       interpolate normal
  output: normal         compute light per pixel
                         multiply texture × light
```

The critical shift: in Gouraud, the **color** (light intensity) is
interpolated. In Phong, the **normal** is interpolated, and the lighting
computation moves to the pixel shader.

| Technique | Interpolated | Lighting computed | Cost | Quality |
|-----------|-------------|-------------------|------|---------|
| **Flat** | Nothing | Once per triangle | Cheapest | Faceted |
| **Gouraud** | Light intensity | Per vertex (×3) | Cheap | Smooth, misses intra-triangle detail |
| **Phong** | Normal vector | Per pixel | Expensive | Smooth, captures all detail |

---

## The Phong Shader

The `DrawPhongShadedModelShader` in
`src/software_buffer/ex0b_phong_shading.rs` reuses `VertexShaderData` (from
lesson 8) as its interpolated type — the same struct that carries the normal.
The difference is entirely in **where** the lighting is computed.

### The vertex shader

```rust
impl<'a> VertexShader for DrawPhongShadedModelShader<'a> {
    type Output = VertexShaderData;
    fn transform_vertices(&self, input: VertexShaderData) -> VertexShaderData {
        let position = (self.proj_matrix * self.view_matrix * self.model_matrix) * input.position;
        let normal = (self.model_matrix * input.normal).normalize_or_zero();
        VertexShaderData { position, normal, ..input }
    }
}
```

This is identical to the **normals shader** from lesson 9: the position is
transformed by the full MVP matrix, and the normal is transformed by the model
matrix only (to stay in world space). The normal is then **normalized** with
`.normalize_or_zero()` — the model matrix (especially scaling) can change the
normal's length, and we need unit-length normals for correct per-pixel
lighting. Normalizing here, once per vertex, means the interpolated normals
start from a consistent unit length before being blended. No lighting is
computed here — the normal is passed through for the pixel shader to use.

### The pixel shader

```rust
impl<'a> PixelShader for DrawPhongShadedModelShader<'a> {
    type Input = VertexShaderData;

    fn draw_pixel(
        &self,
        software_buffer: &mut SoftwareBuffer,
        depth_texture: &mut [f32],
        vertex_input: VertexShaderData,
        fragment_x: u16,
        fragment_y: u16
    ) {
        // Depth test (same as before)
        let fragment_index = (fragment_y as usize) * software_buffer.get_width() as usize
            + (fragment_x as usize);
        if depth_texture[fragment_index] > vertex_input.position.z { return; }
        depth_texture[fragment_index] = vertex_input.position.z;

        // --- Per-pixel lighting ---
        let normal = vertex_input.normal;
        let normal = if normal.length_squared() > 1.0 {
            normal.normalize_or_zero()
        } else {
            normal
        };
        let reversed_light_dir = -self.light_direction;
        let attenuation = normal.dot(reversed_light_dir).max(0.0);
        let light_color = (self.ambient_color + self.light_color * attenuation)
            .clamp(glam::Vec3::ZERO, glam::Vec3::ONE);

        // --- Bilinear texture sampling (same as before) ---
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
        let mut color = color0.lerp(color1, v_frac);

        // --- Multiply texture by per-pixel light color ---
        color.r = (color.r as f32 * light_color.x).clamp(0.0, 255.0) as u8;
        color.g = (color.g as f32 * light_color.y).clamp(0.0, 255.0) as u8;
        color.b = (color.b as f32 * light_color.z).clamp(0.0, 255.0) as u8;

        software_buffer.set_pixel(fragment_x, fragment_y, color);
    }
}
```

Let's focus on what's new compared to the Gouraud pixel shader.

### Step 1: Receive the interpolated normal

```rust
let normal = vertex_input.normal;
```

In Gouraud, `vertex_input` was `GouraudSharedShaderData` with a `color` field.
In Phong, `vertex_input` is `VertexShaderData` with a `normal` field. The
normal has been interpolated across the triangle by
`interpolate_by_barycentric` — it's a blend of the three vertex normals,
varying smoothly from pixel to pixel.

### Step 2: Renormalize

```rust
let normal = if normal.length_squared() > 1.0 {
    normal.normalize_or_zero()
} else {
    normal
};
```

The vertex shader already normalizes each vertex normal, but after barycentric
interpolation the normal is generally **not** a unit vector again. Blending
three unit-length normals produces a vector whose length depends on how
similar the directions are — if they diverge, the result is shorter; if they
align closely, it stays near unit length. The dot product `n · l` assumes
`|n| = 1` — otherwise the attenuation would be scaled by the normal's length,
producing incorrect brightness.

The renormalization condition `length_squared() > 1.0` is an optimization:
if the interpolated normal happens to be shorter than unit length (which is
common when blending normals pointing in different directions), the dot
product will be **smaller** than it should be — but this produces a slightly
darker, still acceptable result. Only when the normal is **longer** than unit
length (which would make the surface too bright) do we renormalize.

> **A stricter approach** would be to always renormalize:
> ```rust
> let normal = normal.normalize_or_zero();
> ```
> This is more correct but slightly more expensive. The conditional approach
> is a pragmatic compromise for a software renderer.

### Step 3: Compute lighting per pixel

```rust
let reversed_light_dir = -self.light_direction;
let attenuation = normal.dot(reversed_light_dir).max(0.0);
let light_color = (self.ambient_color + self.light_color * attenuation)
    .clamp(glam::Vec3::ZERO, glam::Vec3::ONE);
```

This is the **same** Lambert diffuse formula from the Gouraud lesson:

```
attenuation = max(n · l, 0)
light_color = ambient + diffuse_light × attenuation
```

The difference is that this computation now runs **for every pixel**, using
the interpolated and renormalized normal — not just 3 times per triangle.

### Step 4: Texture sampling and modulation

The bilinear texture sampling is identical to lessons 7–10. The final step
multiplies the texture color by the per-pixel light color:

```rust
color.r = (color.r as f32 * light_color.x).clamp(0.0, 255.0) as u8;
color.g = (color.g as f32 * light_color.y).clamp(0.0, 255.0) as u8;
color.b = (color.b as f32 * light_color.z).clamp(0.0, 255.0) as u8;
```

---

## Why Phong Matters: The Specular Case

With **diffuse-only** lighting (what we have now), the difference between
Gouraud and Phong is subtle on the dragon model. The real advantage of Phong
shading becomes visible with **specular highlights** — bright spots where
light reflects directly toward the viewer.

Specular highlights can be very small — smaller than a single triangle. With
Gouraud shading, if a highlight falls between vertices, it's completely
missed: the per-vertex light intensity is low, and interpolation can't
reconstruct the peak. With Phong shading, the per-pixel normal catches the
exact angle where the highlight occurs, producing a sharp bright spot.

```
Gouraud (misses highlight):
  vertex: dim    vertex: dim    vertex: dim
     ·····························
     · (highlight is here, but    ·
     ·  all vertices are dim, so  ·
     ·  interpolation is dim too) ·
     ·····························

Phong (catches highlight):
  vertex: dim    pixel: BRIGHT   vertex: dim
     ··············●···············
     · (per-pixel normal catches  ·
     ·  the exact highlight angle)·
     ······························
```

We'll add specular lighting in a future lesson. For now, the per-pixel normal
computation already provides slightly smoother shading transitions than
Gouraud, especially on triangles with rapidly varying normals.

---

## The Interpolated Normal: A Closer Look

In Gouraud shading, we interpolated **colors** (light intensities in `[0, 1]`).
Interpolating colors is safe — any weighted average of valid colors is still
a valid color.

In Phong shading, we interpolate **normals** (direction vectors). This is
more delicate:

1. **The interpolated normal is not unit length.** The vertex shader
   normalizes each vertex normal, but blending three unit-length vectors
   produces a vector whose length depends on how similar the directions are.
   If all three normals point the same way, the result is unit length. If
   they point in very different directions, the result is shorter. This is
   why we renormalize in the pixel shader.

2. **The interpolated normal is not "physically correct."** Barycentric
   interpolation of normals is a linear blend in Cartesian space, which
   doesn't perfectly correspond to angular interpolation. For example,
   blending `(1, 0, 0)` and `(0, 1, 0)` at 50% gives `(0.5, 0.5, 0)`,
   which after normalization is `(0.707, 0.707, 0)` — a 45° angle. But the
   "correct" angular midpoint between 0° and 90° is indeed 45°, so in this
   case it works out. In general, the linear approximation is good enough
   for most rendering.

3. **The normal still needs the model matrix transform.** The vertex shader
   transforms the normal by `model_matrix` before interpolation. This ensures
   the normal is in world space, where the light direction is also defined.

---

## Example Walkthrough

The example — `examples/ex0B_phong_shading.rs` — is structurally identical to
the Gouraud example, with the shader swapped:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex0b_phong_shading::DrawPhongShadedModelShader
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

    let dragon_model = ObjModel::load_from_string(DRAGON_MODEL_TEXT).unwrap();

    let model_matrix = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 0.0))
        * glam::Mat4::from_scale(glam::vec3(2.0, 2.0, 2.0));

    let view_matrix = glam::camera::rh::view::look_at_mat4(
        glam::vec3(450.0, -500.0, -500.0),
        glam::vec3(0.0, 0.0, 0.0),
        glam::vec3(0.0, 1.0, 0.0)
    );

    let proj_matrix = glam::camera::rh::proj::opengl::perspective(
        std::f32::consts::FRAC_PI_2,
        buffer.get_width() as f32 / buffer.get_height() as f32,
        0.1,
        1000.0
    );

    buffer.draw_obj_model(
        &dragon_model,
        &mut depth_texture,
        &DrawPhongShadedModelShader {
            model_matrix,
            view_matrix,
            proj_matrix,
            light_direction: glam::vec4(0.0, -1.0, -1.0, 0.0).normalize(),
            light_color: glam::vec3(0.8, 0.5, 0.0),
            ambient_color: glam::vec3(0.0, 0.0, 0.4),
            texture: &texture,
            texture_width: image.width() as u16,
            texture_height: image.height() as u16
        }
    );

    buffer.print_as_ppm();
}
```

The lighting parameters are identical to the Gouraud lesson — same light
direction, same colors. This makes it easy to compare the two renders
side by side and spot the differences.

---

## How to Run the Example

```sh
cargo run --example ex0B_phong_shading > pictures/ex0B_phong_shading.ppm
```

Or build and run separately:

```sh
cargo build --release --example ex0B_phong_shading
./target/release/examples/ex0B_phong_shading > pictures/ex0B_phong_shading.ppm
```

Open `pictures/ex0B_phong_shading.ppm` in any image viewer. Compare it with
`pictures/ex0A_gouraud_shading.ppm` from the previous lesson. With
diffuse-only lighting, the difference is subtle — look for slightly smoother
shading transitions on curved areas of the dragon, especially where the
normal changes rapidly across a triangle.

---

## Summary

In this lesson we learned about:

- **Phong shading** — interpolating the **normal** across the triangle and
  computing lighting **per pixel** in the pixel shader.
- **Gouraud vs Phong** — interpolating light intensity (Gouraud) vs
  interpolating normals (Phong), and why Phong captures intra-triangle
  lighting detail that Gouraud misses.
- **Normal renormalization** — why the interpolated normal needs to be
  renormalized and the conditional approach used in the shader.
- **Reusing `VertexShaderData`** — Phong passes the normal through the
  vertex shader unchanged (after model transform), without pre-computing
  lighting.
- **Per-pixel lighting cost** — the trade-off between Gouraud's 3
  evaluations per triangle and Phong's one evaluation per pixel.
- **Why specular highlights need Phong** — small highlights that fall between
  vertices are invisible in Gouraud but captured in Phong.

In the next section we'll move beyond software rendering and introduce the
**MEV** graphics API for hardware-accelerated rendering.

---

## Exercises

### Exercise 1: Compare Gouraud and Phong side by side

Render both `ex0A_gouraud_shading` and `ex0B_phong_shading` and open the
images side by side. Can you spot any differences? Look carefully at areas
where the surface curvature is high — the dragon's snout, claws, or wing
membranes. The Phong version should have slightly smoother shading
transitions.

### Exercise 2: Always renormalize

Replace the conditional renormalization with unconditional normalization:

```rust
let normal = normal.normalize_or_zero();
```

Compare the result. Is the image brighter, darker, or the same? Why might
the conditional approach (`length_squared() > 1.0`) produce slightly
different results in areas where the interpolated normal is shorter than
unit length?

### Exercise 3: Add specular highlights

Extend the pixel shader to compute a **specular term** using the Phong
reflection model:

```rust
let view_dir = /* direction from surface to camera */;
let reflect_dir = (-reversed_light_dir).reflect(normal);
let spec_attenuation = reflect_dir.dot(view_dir).max(0.0).powf(shininess);
let specular = specular_color * spec_attenuation;
```

Add `specular` to the final light color. Use a high `shininess` value (e.g.,
32 or 64) for tight highlights. Now compare Gouraud and Phong — the
difference should be dramatic, with Phong capturing sharp specular spots that
Gouraud completely misses.

> **Hint:** Computing `view_dir` requires the surface position in world
> space, which we don't currently pass to the pixel shader. You'll need to
> add a world-space position field to the interpolated data, or approximate
> the view direction as constant (valid when the camera is far from the
> model).

### Exercise 4: Flat shading with per-pixel normals

Instead of using the model's smooth vertex normals, compute a **face normal**
per triangle (as in lesson 9, exercise 6) and assign it to all three
vertices. With Phong shading, the interpolated normal will be constant
across each triangle, producing **flat shading** — each triangle is a single
solid shade. Compare this with the smooth Phong result to see how much the
vertex normals contribute to the dragon's smooth appearance.

### Exercise 5: Measure the performance difference

Add timing to both the Gouraud and Phong examples:

```rust
let start = std::time::Instant::now();
buffer.draw_obj_model(/* ... */);
let elapsed = start.elapsed();
eprintln!("Render time: {:?}", elapsed);
```

How much slower is Phong than Gouraud? The difference depends on the number
of pixels rendered — more pixels means more per-pixel lighting evaluations.
Try increasing the buffer size to 1280×960 and compare again.