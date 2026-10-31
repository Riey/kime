/* Raw XTEST key injector: xtest_key <x11_keycode> <press|release|tap>
 * xdotool cannot inject unmapped raw keycodes; this can (needed by #721/#603).
 */
#include <X11/Xlib.h>
#include <X11/extensions/XTest.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: xtest_key <keycode> <press|release|tap>\n"); return 1; }
    Display *dpy = XOpenDisplay(NULL);
    if (!dpy) { fprintf(stderr, "no display\n"); return 2; }
    unsigned code = (unsigned)atoi(argv[1]);
    if (!strcmp(argv[2], "press") || !strcmp(argv[2], "tap"))
        XTestFakeKeyEvent(dpy, code, True, CurrentTime);
    if (!strcmp(argv[2], "release") || !strcmp(argv[2], "tap"))
        XTestFakeKeyEvent(dpy, code, False, CurrentTime);
    XSync(dpy, False);
    XCloseDisplay(dpy);
    return 0;
}
