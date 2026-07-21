# Lesson I.0: Red Picture

> **Result:** `pictures/ex00_red_picture.ppm`
>
> In this lesson we will create a pixel buffer, fill it with a red color, and
> output the image to the terminal in PPM format. This is the simplest example —
> but it's where the foundation for everything else is laid.

---

## What We Are Doing

At its core, computer graphics is about working with an array of pixels.
A pixel is a dot on the screen that has a color. The screen is a rectangular
grid of such dots. Our task in this lesson is to create this grid in memory,
fill it with a single color, and save the result to a file.

We are not using windows, GPUs, or any graphics libraries. Everything happens
in plain Rust code: we work with a vector (array) of colors and output them
in the PPM text format directly to standard output.

---

## Data Structures

### `Color24` — pixel color

```rust
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color24 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
```

A color is represented by three bytes: red (`r`), green (`g`), and blue (`b`).
Each channel is a value from `0` to `255` (type `u8`). This is the so-called
**RGB 24-bit** format (8 bits per channel × 3 channels = 24 bits).

For example:
- `{ r: 255, g: 0, b: 0 }` — pure red
- `{ r: 0, g: 255, b: 0 }` — pure green
- `{ r: 0, g: 0, b: 255 }` — pure blue
- `{ r: 255, g: 255, b: 255 }` — white
- `{ r: 0, g: 0, b: 0 }` — black (default value)

The `Copy` and `Clone` traits allow copying a color value on assignment.
`Default` gives a black color (`r: 0, g: 0, b: 0`), which is convenient when
creating an empty buffer.

### `SoftwareBuffer` — pixel buffer

```rust
#[derive(Clone, Debug)]
pub struct SoftwareBuffer {
    pixels: Vec<Color24>,
    width: u16,
    height: u16,
}
```

`SoftwareBuffer` is the main object we will work with throughout the first
part of this tutorial. It contains:

- `pixels` — a vector of colors, a one-dimensional array storing all pixels of the image.
- `width` and `height` — the width and height of the image in pixels.

Importantly, although the image is two-dimensional (width × height), the pixels
are stored in a **one-dimensional vector**. This is the standard approach in
graphics. Pixels are arranged row by row, left to right, top to bottom.

The buffer is created like this:

```rust
impl SoftwareBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            pixels: vec![Color24::default(); width as usize * height as usize],
            width,
            height,
        }
    }
}
```

The `vec![Color24::default(); N]` macro creates a vector of `N` elements,
each initialized to black. The vector size equals `width * height` — exactly
as many pixels as needed for an image of that size.

---

## Basic Buffer Operations

The `ex00_base_operations` module extends `SoftwareBuffer` with a set of
basic methods.

### Getting Dimensions

```rust
pub fn get_width(&self) -> u16 {
    self.width
}

pub fn get_height(&self) -> u16 {
    self.height
}
```

Simple getters returning the buffer's width and height.

### Clearing the Buffer (`clear`)

```rust
pub fn clear(&mut self, color: Color24) {
    for pixel in self.pixels.iter_mut() {
        *pixel = color;
    }
}
```

The `clear` method fills the entire buffer with a single color. It iterates
over every pixel in the vector and assigns the given color to it. This is the
fastest way to set the background for a frame.

> **Why do we need `clear`?** In rendering, each frame usually starts with
> clearing the buffer. If you skip this step, pixels from the previous frame
> will remain on screen. In our example, we fill the buffer with a red color
> and get a solid red picture.

### Setting a Pixel (`set_pixel`)

```rust
pub fn set_pixel(&mut self, x: u16, y: u16, color: Color24) {
    let idx = y as usize * self.width as usize + x as usize;
    let Some(pixel) = self.pixels.get_mut(idx) else {
        return;
    };
    *pixel = color;
}
```

To color a specific pixel, we need to convert the 2D coordinates `(x, y)`
into a 1D index. The formula is simple:

```
index = y * width + x
```

Pixel `(0, 0)` is at the top-left corner. Pixel `(width-1, height-1)` is at
the bottom-right corner.

Notice the safe index handling: instead of directly indexing (`self.pixels[idx]`),
`get_mut` is used, which returns `None` if the coordinate is out of bounds.
In that case the method simply returns — a pixel outside the screen is ignored.

### Reading a Pixel (`get_pixel`)

```rust
pub fn get_pixel(&self, x: u16, y: u16) -> Option<Color24> {
    let idx = y as usize * self.width as usize + x as usize;
    self.pixels.get(idx).copied()
}
```

Similar to `set_pixel`, but for reading. Returns `Option<Color24>`:
`Some(color)` if the coordinate is inside the buffer, and `None` otherwise.

---

## Outputting an Image in PPM Format

The `ex00_printing` module adds the `print_as_ppm` method, which outputs the
buffer contents to standard output in **PPM (Portable Pixel Map)** format.

```rust
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
```

### What is PPM?

**PPM** is a simple text-based image format. It consists of a header followed
by a list of pixel values. The P3 format (textual PPM) looks like this:

```
P3
<width> <height>
<maximum color value>
r g b   r g b   r g b ...
r g b   r g b   r g b ...
...
```

Let's break it down line by line:

1. `P3` — the magic number denoting the textual PPM format (as opposed to
   `P6`, which is binary).
2. `640 480` — the width and height of the image.
3. `255` — the maximum color channel value (8 bits = 255).
4. Then come triplets of numbers `r g b` for each pixel, left to right,
   top to bottom.

PPM is not the most efficient format (textual, large file size), but it's
ideal for learning: you can read it with your eyes and open it in any image
editor. Most image viewers support PPM out of the box.

> In this tutorial, we redirect the program's output to a file with the `.ppm`
> extension. The resulting file can be opened in any image viewer.

---

## Example Walkthrough

Now let's look at the example itself — `examples/ex00_red_picture.rs`:

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24
    }
};

const NICE_RED: Color24 = Color24 { r: 207, g: 81, b: 99 };

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(NICE_RED);
    buffer.print_as_ppm();
}
```

Only three lines of logic — but let's break down each one.

### Step 1: Imports

```rust
use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24
    }
};
```

We import `SoftwareBuffer` and `Color24` from the `mev_graphics_tutorial`
library. These are the same structures we covered above.

### Step 2: Color

```rust
const NICE_RED: Color24 = Color24 { r: 207, g: 81, b: 99 };
```

We define a constant — a pleasant red color. This is not pure red
`(255, 0, 0)`, but a softer shade. The values are chosen to make the
picture look aesthetically pleasing.

### Step 3: Creating the Buffer

```rust
let mut buffer = SoftwareBuffer::new(640, 480);
```

We create a buffer with dimensions 640×480 pixels. This is the standard VGA
resolution, familiar to many from old games and monitors. The buffer is
initialized with black pixels (`Color24::default()`).

The `mut` keyword is required — we will be modifying the buffer (filling it
with color).

### Step 4: Clearing

```rust
buffer.clear(NICE_RED);
```

We fill the entire buffer with the red color. After this line, each of the
307,200 pixels (640 × 480) contains `NICE_RED`.

### Step 5: Output

```rust
buffer.print_as_ppm();
```

We output the buffer contents in PPM format to standard output. If we
redirect this output to a file, we get a finished image.

---

## How to Run the Example

From the project root, run:

```sh
cargo run --example ex00_red_picture > pictures/ex00_red_picture.ppm
```

Or, if building and running separately:

```sh
cargo build --release --example ex00_red_picture
./target/release/examples/ex00_red_picture > pictures/ex00_red_picture.ppm
```

The resulting file `pictures/ex00_red_picture.ppm` can be opened in any
image viewer — you will see a solid red 640×480 picture.

You can also run all examples at once using the script:

```sh
./run_examples.sh
```

---

## Summary

In this lesson we learned about:

- **`Color24`** — a structure for storing an RGB color (3 bytes).
- **`SoftwareBuffer`** — a one-dimensional pixel vector with width and height.
- **Creating a buffer** via `SoftwareBuffer::new(width, height)`.
- **Clearing the buffer** via `clear(color)` — filling all pixels with a single color.
- **Setting and reading pixels** via `set_pixel` and `get_pixel` with conversion
  of 2D coordinates to a 1D index.
- **The PPM format** and outputting an image via `print_as_ppm()`.

This is the basic building block. In the following lessons we will learn to
draw shapes on this buffer — starting with a circle (moon) in lesson
`ex01_draw_moon`.

---

## Exercises

### Exercise 1: Out-of-bounds pixel write

Take the example from this lesson and try drawing a single pixel at a fixed
position *after* the clear, like this:

```rust
buffer.set_pixel(700, 0, Color24 { r: 255, g: 255, b: 255 });
```

**Question:** Why does this call, instead of doing nothing, result in a 
white pixel being drawn somewhere inside the image?

### Exercise 2: Safer `set_pixel` and `get_pixel`
Rewrite `set_pixel` and `get_pixel` methods to ensure that they aren't 
let the caller read or write from unexpected locations of the buffer.