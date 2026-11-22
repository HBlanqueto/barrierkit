# usage: egl-probe.sh   (inside a dev shell that has gcc and libEGL)
# Prints, for each EGL device in the order smithay's `EGLDevice::enumerate` returns them,
# the GL_RENDERER. The experiment measures the first one.
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
probe=${TMPDIR:-/tmp}/blur-phase-0-egl-probe
gcc "$here/egl-probe.c" -o "$probe" -lEGL
MESA=$(echo "$LD_LIBRARY_PATH" | tr ':' '\n' | grep -m1 -e '-mesa-' | sed 's#/lib$##')
for variant in plain always-software vendor-only vendor-and-always-software; do
  echo "=== $variant"
  case $variant in
    plain) env "$probe" ;;
    always-software) env LIBGL_ALWAYS_SOFTWARE=1 "$probe" ;;
    vendor-only) env __EGL_VENDOR_LIBRARY_DIRS=$MESA/share/glvnd/egl_vendor.d "$probe" ;;
    vendor-and-always-software) env LIBGL_ALWAYS_SOFTWARE=1 __EGL_VENDOR_LIBRARY_DIRS=$MESA/share/glvnd/egl_vendor.d "$probe" ;;
  esac 2>&1 | grep -E "^===|eglQueryDevicesEXT|GL_RENDERER|GL_VERSION|MESA-EGL"
done
