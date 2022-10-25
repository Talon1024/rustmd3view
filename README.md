OpenGL Hack: Vertex animation as a texture
==========================================

This is a demo of an "OpenGL hack" I thought up.

The hack
--------

After uploading a texture to the GPU, you reference it in the shader as a [sampler](https://www.khronos.org/opengl/wiki/Sampler_%28GLSL%29) uniform, and use the texture by calling the `texture` function with the sampler and a texture coordinate.

The `texture` function returns a `vec4`, which doesn't necessarily have to be a pixel colour. So what if I used it as a vertex position?!

What are the benefits and drawbacks?
----------------------

- ✔ Less CPU usage
- ✔ Fewer uploads
- ❌ More GPU usage... But GPUs have plenty of power to spare.
