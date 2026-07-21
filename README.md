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
> You can find the results of lesson executions in the pictures folder. Most
> of the pictures are in PPM format, so it's impossible to draw then in this
> document, sadly. To see them, you need to use a PPM viewer, which is usually
> available in the operating system, so it's yet another reason to check out the 
> repository and do the experiments. 
> 
> You can also run the shell script `run_all_examples.sh` to run all the examples
> and regenerate the pictures. This could be useful if you want to make modifications
> to the examples and want to see the results.

The list of lessons is as follows:
- **docs/I Basics/ex00_red_picture**

  Features the creation of a pixel buffer. Then this buffer is cleared with 
  a red color and drawn to the terminal using a PPM format.
- **docs/I Basics/ex01_draw_moon**
  
  Shows how to draw simple shapes on a screen sharing the same approach 
  with the usage of a pixel buffer and printing it to the terminal.
- **docs/I Basics/ex02_winding_number_triangle**
  
  Shows how to draw a triangle using the winding number method.

- **docs/I Basics/ex03_winding_trick_for_shapes**

  Shows a trick that can be used to draw complex vector shapes through 
  a sequence of triangle drawings, where each next triangle inverts the 
  picture, and after all the triangles are drawn, the user will see the 
  vector shape.

- **docs/I Basics/ex04_barycentric_coordinates**
  
  Introduces the concept of barycentric coordinates and shows how to use 
  them to interpolate the values inside a triangle using red, green and 
  blue colors as a simple example.

- **docs/I Basics/ex05_uv_coordinates**
  
  This lesson is a natural continuation of the previous one, where the 
  user will learn about UV coordinates and will draw a couple of triangles 
  with UV coordinates interpolated leveraging the barycentric coordinates.

- **docs/I Basics/ex06_texture_mapping_nearest**
  
  This lesson introduces the concept of texture mapping and shows how to 
  use it to draw a simple texture on a quad made of two triangles.

- **docs/I Basics/ex07_texture_mapping_bilinear**
  
  This lesson is a continuation of the previous one, where the user learns 
  how to get a more smooth texture by using bilinear interpolation instead 
  of choosing the nearest pixel in a texture.

- **docs/I Basics/ex08_drawing_simple_unlit_3d_model**
  
  This lesson introduces the concept of 3D models and shows how to draw 
  a simple unlit 3D model. The user will learn how to write a parser for 
  the Wavefront OBJ format and how to use it to load a 3D model. Then the 
  user will learn how to use previously learned concepts to draw the model. 
  The new concept of a vertex and pixel shader is introduced. Then the user
  writes its first shaders to draw a simple unlit 3D model of a dragon textured 
  with a bilinear interpolation.

- **docs/I Basics/ex09_visualize_normals**

  This lesson is a preparation before the explanation on how to add lighting. 
  The user will learn what normals are, how it is possible to calculate them in
  the case when the model is not provided with them. The concept of smooth and 
  flat shading will be covered too, and in the end the user will learn how to 
  write a shader that visualizes normals of a model.

- **docs/I Basics/ex0A_gouraud_shading**

  This lesson introduces the concept of Gouraud shading and shows how to 
  write a shader that draws a model with a basic lighting.