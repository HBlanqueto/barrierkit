#include <EGL/egl.h>
#include <EGL/eglext.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    const char *client = eglQueryString(EGL_NO_DISPLAY, EGL_EXTENSIONS);
    printf("client extensions: %s\n", client ? client : "(null)");
    PFNEGLQUERYDEVICESEXTPROC qd = (PFNEGLQUERYDEVICESEXTPROC)eglGetProcAddress("eglQueryDevicesEXT");
    PFNEGLQUERYDEVICESTRINGEXTPROC qs = (PFNEGLQUERYDEVICESTRINGEXTPROC)eglGetProcAddress("eglQueryDeviceStringEXT");
    PFNEGLGETPLATFORMDISPLAYEXTPROC gpd = (PFNEGLGETPLATFORMDISPLAYEXTPROC)eglGetProcAddress("eglGetPlatformDisplayEXT");
    if (!qd) { printf("no eglQueryDevicesEXT\n"); return 1; }
    EGLDeviceEXT devs[8]; EGLint n = 0;
    EGLBoolean ok = qd(8, devs, &n);
    printf("eglQueryDevicesEXT ok=%d n=%d err=0x%x\n", ok, n, eglGetError());
    for (int i = 0; i < n; i++) {
        const char *ext = qs(devs[i], EGL_EXTENSIONS);
        printf("dev %d extensions: %s\n", i, ext ? ext : "(null)");
        EGLDisplay d = gpd(EGL_PLATFORM_DEVICE_EXT, devs[i], NULL);
        EGLint maj, min;
        if (!eglInitialize(d, &maj, &min)) { printf("dev %d eglInitialize failed 0x%x\n", i, eglGetError()); continue; }
        printf("dev %d EGL %d.%d vendor=%s ver=%s\n", i, maj, min, eglQueryString(d, EGL_VENDOR), eglQueryString(d, EGL_VERSION));
        printf("dev %d display ext: %s\n", i, eglQueryString(d, EGL_EXTENSIONS));
        eglBindAPI(EGL_OPENGL_ES_API);
        EGLint attr[] = {EGL_CONTEXT_MAJOR_VERSION, 3, EGL_NONE};
        EGLContext c = eglCreateContext(d, EGL_NO_CONFIG_KHR, EGL_NO_CONTEXT, attr);
        printf("dev %d context=%p err=0x%x\n", i, c, eglGetError());
        if (c && eglMakeCurrent(d, EGL_NO_SURFACE, EGL_NO_SURFACE, c)) {
            typedef const unsigned char *(*GetString)(unsigned);
            GetString gs = (GetString)eglGetProcAddress("glGetString");
            printf("GL_VENDOR=%s\nGL_RENDERER=%s\nGL_VERSION=%s\n", gs(0x1F00), gs(0x1F01), gs(0x1F02));
        } else printf("dev %d makeCurrent failed 0x%x\n", i, eglGetError());
    }
    return 0;
}
