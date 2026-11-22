MESA=$(echo "$LD_LIBRARY_PATH" | tr ':' '\n' | grep -m1 -e '-mesa-' | sed 's#/lib$##')
export __EGL_VENDOR_LIBRARY_DIRS=$MESA/share/glvnd/egl_vendor.d
