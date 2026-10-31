/* Minimal client that gives the seat a keyboard with NO keymap: creates a
 * zwp_virtual_keyboard_v1 and never uploads a keymap. wlroots then reports
 * keymap(format=NO_KEYMAP, /dev/null, size=0) to any input-method keyboard
 * grab — the #782 precondition. (A seat with no keyboard at all sends no
 * keymap event and never triggers the bug.)
 * Prints READY, then serves the connection until killed. */
#include <stdio.h>
#include <string.h>
#include <wayland-client.h>
#include "virtual-keyboard-unstable-v1-client.h"

static struct wl_seat *seat;
static struct zwp_virtual_keyboard_manager_v1 *mgr;

static void global_add(void *data, struct wl_registry *reg, uint32_t name,
                       const char *iface, uint32_t ver) {
    if (!strcmp(iface, wl_seat_interface.name))
        seat = wl_registry_bind(reg, name, &wl_seat_interface, 1);
    else if (!strcmp(iface, zwp_virtual_keyboard_manager_v1_interface.name))
        mgr = wl_registry_bind(reg, name, &zwp_virtual_keyboard_manager_v1_interface, 1);
}
static void global_remove(void *d, struct wl_registry *r, uint32_t n) {}
static const struct wl_registry_listener reg_listener = {global_add, global_remove};

int main(void) {
    struct wl_display *dpy = wl_display_connect(NULL);
    if (!dpy) { fprintf(stderr, "connect failed\n"); return 1; }
    wl_registry_add_listener(wl_display_get_registry(dpy), &reg_listener, NULL);
    wl_display_roundtrip(dpy);
    if (!seat || !mgr) { fprintf(stderr, "missing seat or vkbd manager\n"); return 1; }
    zwp_virtual_keyboard_manager_v1_create_virtual_keyboard(mgr, seat);
    wl_display_roundtrip(dpy);
    printf("READY\n");
    fflush(stdout);
    while (wl_display_dispatch(dpy) != -1) { }
    wl_display_disconnect(dpy);
    return 0;
}
