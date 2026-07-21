# MEV_GRAPHICS_TUTORIAL
This is a tutorial on computer graphics basics that uses a 
[MEV](https://github.com/zakarumych/mev.git) library.

## The structure of a tutorial

The lessons are separated to different example programs, one by a lesson.

Learning material starts from a simple software rendering of things, where 
the user gets to see how to do a rendering without even having a window, 
printing pictures to the terminal in a simple text format known as PPM.

Then the user gets to see how to use the MEV library, still ignoring window 
stuff until the moment they actually need it.

At the moment the tutorial introduces the window with the help of a Winit 
library, the user will be already familiar with all the base building 
blocks of a graphics API. So they will be ready to start making fun stuff 
straight away.

Gradually, by the course of the tutorial, the user will create a game 
library similar to Raylib and will be able to make actual games with it.

## The list of lessons
> Disclaimer:
> > The approach on the writing of this tutorial series is the following:
> > 1. The author implements example programs that show how to do 
> > something. This process is done by hand to ensure that the code 
> > is easy to understand and not bloated with unnecessary stuff.
> > 2. The LLM is then asked to explain the code of an example in lesson 
> > file inside a `/docs` folder. Then the author does some minor review 
> > of what was said by the LLM. If the explanation is not concise, the
> > author does some minor tweaks to the explanation and then the lesson
> > is considered done.
> > 3. If you feel that the quality of the explanation is not good enough,
> > you are welcome to make a pull request with a better explanation. So we
> > will take the best of two worlds: the author of this tutorial series
> > will be able to get the good enough explanations faster, and you, fellow 
> > readers, will be able to improve these explanations, and the tutorial will
> > gradually become better. And anyone who was participating will be also
> > mentioned as the rightful contributor of the lesson.

The list of lessons is as follows:
- **docs/I Basics/**
  
  + **[ex00_red_picture](./docs/I%20Basics/ex00_red_picture.md)**

    Features the creation of a pixel buffer. Then this buffer is cleared with 
    a red color and drawn to the terminal using a PPM format.
  + **[ex01_draw_moon](./docs/I%20Basics/ex01_draw_moon.md)**
  
    Shows how to draw simple shapes on a screen sharing the same approach 
    with the usage of a pixel buffer and printing it to the terminal.
  + **[ex02_winding_number_triangle](./docs/I%20Basics/ex02_winding_number_triangle.md)**
  
    Shows how to draw a triangle using the winding number method.

  + **[ex03_winding_trick_for_shapes](./docs/I%20Basics/ex03_winding_trick_for_shapes.md)**

    Shows a trick that can be used to draw complex vector shapes through 
    a sequence of triangle drawings, where each next triangle inverts the 
    picture, and after all the triangles are drawn, the user will see the 
    vector shape.

  + **[ex04_barycentric_coordinates](./docs/I%20Basics/ex04_barycentric_coordinates.md)**
  
    Introduces the concept of barycentric coordinates and shows how to use 
    them to interpolate the values inside a triangle using red, green and 
    blue colors as a simple example.

  + **[ex05_uv_coordinates](./docs/I%20Basics/ex05_uv_coordinates.md)**
  
    This lesson is a natural continuation of the previous one, where the 
    user will learn about UV coordinates and will draw a couple of triangles 
    with UV coordinates interpolated leveraging the barycentric coordinates.

  + **[ex06_texture_mapping_nearest](./docs/I%20Basics/ex06_texture_mapping_nearest.md)**
  
    This lesson introduces the concept of texture mapping and shows how to 
    use it to draw a simple texture on a quad made of two triangles.

  + **[ex07_texture_mapping_bilinear](./docs/I%20Basics/ex07_texture_mapping_bilinear.md)**
  
    This lesson is a continuation of the previous one, where the user learns 
    how to get a more smooth texture by using bilinear interpolation instead 
    of choosing the nearest pixel in a texture.

  + **[ex08_drawing_simple_unlit_3d_model](./docs/I%20Basics/ex08_drawing_simple_unlit_3d_model.md)**
  
    This lesson introduces the concept of 3D models and shows how to draw 
    a simple unlit 3D model. The user will learn how to write a parser for 
    the Wavefront OBJ format and how to use it to load a 3D model. Then the 
    user will learn how to use previously learned concepts to draw the model. 
    The new concept of a vertex and pixel shader is introduced. Then the user
    writes its first shaders to draw a simple unlit 3D model of a dragon textured 
    with a bilinear interpolation.

  + **[ex09_visualize_normals](./docs/I%20Basics/ex09_visualize_normals.md)**

    This lesson is a preparation before the explanation on how to add lighting. 
    The user will learn what normals are, how it is possible to calculate them in
    the case when the model is not provided with them. The concept of smooth and 
    flat shading will be covered too, and in the end the user will learn how to 
    write a shader that visualizes normals of a model.

  + **[ex0A_gouraud_shading](./docs/I%20Basics/ex0A_gouraud_shading.md)**

    This lesson introduces the concept of Gouraud shading and shows how to 
    write a shader that draws a model with a basic lighting.
  
  + **[ex0B_phong_shading](./docs/I%20Basics/ex0B_phong_shading.md)**
    
    This lesson introduces the concept of Phong shading, which is basically a
    Gouraud shading but in a pixel shader stage and shows how to 
    write a shader that draws a model with smoother looking lighting.
  
- **docs/II MEV basics/**
  
  + **[ex10_mev_device_creation](./docs/II%20MEV%20basics/ex10_mev_device_creation.md)**
    
    Shows how to create a device and a queue. The approach of enumeration 
    on available devices will be covered. The user will learn how to check
    what features are available on a device and what type of device it is. 
    With this information the user will be able to create a device that 
    supports the features he needs.
  
  + **[ex11_mev_clear](./docs/II%20MEV%20basics/ex11_mev_clear.md)**
    
    This lesson sheds a light on the concept of images and buffers. The concept of
    a surface will also be covered. Then the user will learn how to create a "fake"
    surface for offscreen rendering. Then they will learn how to create a so-called
    command encoder, through which they will be able to create a command for the 
    clear of a frame image. Then the frame gets presented, and then they will copy 
    the contents of the frame image to the buffer that will be used to read the pixel 
    info. This pixel info then will be used to fill a pixel buffer known from the 
    previous part of a tutorial. At the end, the contents of the pixel buffer will be 
    printed to the terminal in a PPM format.

TODO: write the rest of the lessons.