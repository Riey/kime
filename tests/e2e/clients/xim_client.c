/* Minimal XIM client: PreeditNothing|StatusNothing, prints committed UTF-8 to out file. */
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <locale.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    const char *outpath = argc > 1 ? argv[1] : "/dev/stdout";
    setlocale(LC_ALL, "");
    XSetLocaleModifiers("");
    Display *dpy = XOpenDisplay(NULL);
    if (!dpy) { fprintf(stderr, "no display\n"); return 1; }
    Window win = XCreateSimpleWindow(dpy, DefaultRootWindow(dpy), 0, 0, 300, 100,
                                     0, 0, WhitePixel(dpy, DefaultScreen(dpy)));
    XStoreName(dpy, win, "ximprobe");
    XSelectInput(dpy, win, KeyPressMask | KeyReleaseMask | FocusChangeMask |
                            ExposureMask | StructureNotifyMask);
    XMapWindow(dpy, win);
    XIM im = XOpenIM(dpy, NULL, NULL, NULL);
    if (!im) { fprintf(stderr, "XOpenIM failed\n"); return 2; }
    XIC ic = XCreateIC(im, XNInputStyle, XIMPreeditNothing | XIMStatusNothing,
                       XNClientWindow, win, XNFocusWindow, win, NULL);
    if (!ic) { fprintf(stderr, "XCreateIC failed\n"); return 3; }
    FILE *out = fopen(outpath, "w");
    if (!out) return 4;
    char buf[128];
    KeySym ks;
    Status st;
    XEvent ev;
    for (;;) {
        XNextEvent(dpy, &ev);
        if (ev.type == MapNotify) {
            XSetInputFocus(dpy, win, RevertToParent, CurrentTime);
            XSetICFocus(ic);
            fprintf(stderr, "READY\n");
            fflush(stderr);
        }
        if (XFilterEvent(&ev, None)) continue;
        if (ev.type == KeyPress) {
            int n = Xutf8LookupString(ic, &ev.xkey, buf, sizeof(buf) - 1, &ks, &st);
            if ((st == XLookupChars || st == XLookupBoth) && n > 0) {
                buf[n] = 0;
                fprintf(out, "%s", buf);
                fflush(out);
            }
        }
    }
}
