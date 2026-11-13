Mesa 3D 26.2.4 (soft-gl/ in the Windows 7 package)

soft-gl/opengl32.dll and soft-gl/libgallium_wgl.dll are Mesa 3D 26.2.4 (https://mesa3d.org),
the x86 MSVC build from mesa-dist-win 26.2.4 (https://github.com/pal1000/mesa-dist-win), unchanged.
They give OpenGL to computers without a video driver.

Licences, from the Mesa source at tag mesa-26.2.4
(https://gitlab.freedesktop.org/mesa/mesa/-/tree/mesa-26.2.4/licenses):

- MIT.txt         the core of Mesa and most of its code. The copyright holders are named in the
                  SPDX header of each source file.
- Apache-2.0.txt, BSL-1.0.txt, SGI-B-2.0.txt
                  the other licences Mesa lists for parts of its code.
- mesa-dist-win-MIT.txt
                  the build scripts of mesa-dist-win (Copyright (c) 2017-2020 pal1000).

libgallium_wgl.dll includes LLVM 23.1.2 (llvmpipe), under the Apache License 2.0 with the LLVM
Exception, which does not require this notice for compiled code; it is given for completeness.
