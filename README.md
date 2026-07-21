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
- *ex00_red_picture*
  Features the creation of a pixel buffer. Then this buffer is cleared with 
  a red color and drawn to the terminal using a PPM format.
  [ex00_red_picture](./pictures/ex00_red_picture.ppm)