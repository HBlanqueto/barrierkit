# Native libraries needed to build and link Skia.

{ pkgs }:
with pkgs;
[
  brotli
  expat
  fontconfig
  freetype
  icu
  libdrm
  libglvnd
  libjpeg
  libpng
  libwebp
  vulkan-headers
  zlib
]
