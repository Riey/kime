/* Plain wl_keyboard probe (no text-input): xdg-shell toplevel that appends one
 * line per keyboard event to argv[1]:
 *   <ts_ms> enter
 *   <ts_ms> leave
 *   <ts_ms> press <evdev_code>
 *   <ts_ms> release <evdev_code>
 * Prints READY on stderr once the surface is mapped (buffer committed).
 * Used by W-03 (#744: keys must be forwarded when no text input is enabled)
 * and the repeat-cadence stretch assertions of W-02 (#666).
 */
#define _GNU_SOURCE
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <sys/mman.h>
#include <wayland-client.h>
#include "xdg-shell-client.h"

static struct wl_compositor *compositor;
static struct wl_shm *shm;
static struct wl_seat *seat;
static struct xdg_wm_base *wm_base;
static struct wl_surface *surface;
static struct wl_buffer *buffer;
static FILE *out;
static int mapped;

static uint32_t now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint32_t)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);
}

static void logline(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    fprintf(out, "%u ", now_ms());
    vfprintf(out, fmt, ap);
    fputc('\n', out);
    fflush(out);
    va_end(ap);
}

/* -- wl_keyboard -- */
static void kb_keymap(void *d, struct wl_keyboard *k, uint32_t fmt, int fd, uint32_t size) {
    close(fd);
}
static void kb_enter(void *d, struct wl_keyboard *k, uint32_t serial,
                     struct wl_surface *s, struct wl_array *keys) {
    logline("enter");
}
static void kb_leave(void *d, struct wl_keyboard *k, uint32_t serial, struct wl_surface *s) {
    logline("leave");
}
static void kb_key(void *d, struct wl_keyboard *k, uint32_t serial, uint32_t time,
                   uint32_t key, uint32_t state) {
    logline("%s %u", state == WL_KEYBOARD_KEY_STATE_PRESSED ? "press" : "release", key);
}
static void kb_modifiers(void *d, struct wl_keyboard *k, uint32_t serial, uint32_t dep,
                         uint32_t lat, uint32_t lock, uint32_t grp) {}
static void kb_repeat_info(void *d, struct wl_keyboard *k, int32_t rate, int32_t delay) {
    logline("repeat_info %d %d", rate, delay);
}
static const struct wl_keyboard_listener kb_listener = {
    kb_keymap, kb_enter, kb_leave, kb_key, kb_modifiers, kb_repeat_info,
};

/* -- wl_seat -- */
static void seat_caps(void *d, struct wl_seat *s, uint32_t caps) {
    if (caps & WL_SEAT_CAPABILITY_KEYBOARD) {
        struct wl_keyboard *kb = wl_seat_get_keyboard(s);
        wl_keyboard_add_listener(kb, &kb_listener, NULL);
    }
}
static void seat_name(void *d, struct wl_seat *s, const char *n) {}
static const struct wl_seat_listener seat_listener = {seat_caps, seat_name};

/* -- registry -- */
static void global_add(void *d, struct wl_registry *reg, uint32_t name,
                       const char *iface, uint32_t ver) {
    if (!strcmp(iface, wl_compositor_interface.name))
        compositor = wl_registry_bind(reg, name, &wl_compositor_interface, 4);
    else if (!strcmp(iface, wl_shm_interface.name))
        shm = wl_registry_bind(reg, name, &wl_shm_interface, 1);
    else if (!strcmp(iface, wl_seat_interface.name)) {
        seat = wl_registry_bind(reg, name, &wl_seat_interface, 5);
        wl_seat_add_listener(seat, &seat_listener, NULL);
    } else if (!strcmp(iface, xdg_wm_base_interface.name))
        wm_base = wl_registry_bind(reg, name, &xdg_wm_base_interface, 1);
}
static void global_remove(void *d, struct wl_registry *r, uint32_t n) {}
static const struct wl_registry_listener reg_listener = {global_add, global_remove};

/* -- xdg-shell -- */
static void wm_ping(void *d, struct xdg_wm_base *wm, uint32_t serial) {
    xdg_wm_base_pong(wm, serial);
}
static const struct xdg_wm_base_listener wm_listener = {wm_ping};

static struct wl_buffer *make_buffer(int w, int h) {
    int stride = w * 4;
    int size = stride * h;
    int fd = memfd_create("wlkbd", 0);
    if (ftruncate(fd, size) < 0) return NULL;
    void *data = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    memset(data, 0x80, size);
    munmap(data, size);
    struct wl_shm_pool *pool = wl_shm_create_pool(shm, fd, size);
    struct wl_buffer *b =
        wl_shm_pool_create_buffer(pool, 0, w, h, stride, WL_SHM_FORMAT_XRGB8888);
    wl_shm_pool_destroy(pool);
    close(fd);
    return b;
}

static void xdg_surf_configure(void *d, struct xdg_surface *xs, uint32_t serial) {
    xdg_surface_ack_configure(xs, serial);
    wl_surface_attach(surface, buffer, 0, 0);
    wl_surface_commit(surface);
    if (!mapped) {
        mapped = 1;
        fprintf(stderr, "READY\n");
        fflush(stderr);
    }
}
static const struct xdg_surface_listener xdg_surf_listener = {xdg_surf_configure};

static void toplevel_configure(void *d, struct xdg_toplevel *t, int32_t w, int32_t h,
                               struct wl_array *states) {}
static void toplevel_close(void *d, struct xdg_toplevel *t) { exit(0); }
static const struct xdg_toplevel_listener toplevel_listener = {toplevel_configure,
                                                               toplevel_close};

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "usage: wlkbd_probe <outfile>\n"); return 1; }
    out = fopen(argv[1], "w");
    if (!out) { fprintf(stderr, "cannot open %s\n", argv[1]); return 1; }
    struct wl_display *dpy = wl_display_connect(NULL);
    if (!dpy) { fprintf(stderr, "connect failed\n"); return 2; }
    struct wl_registry *reg = wl_display_get_registry(dpy);
    wl_registry_add_listener(reg, &reg_listener, NULL);
    wl_display_roundtrip(dpy);
    if (!compositor || !shm || !seat || !wm_base) {
        fprintf(stderr, "missing globals (compositor/shm/seat/xdg_wm_base)\n");
        return 3;
    }
    xdg_wm_base_add_listener(wm_base, &wm_listener, NULL);
    buffer = make_buffer(320, 240);
    if (!buffer) { fprintf(stderr, "buffer alloc failed\n"); return 4; }
    surface = wl_compositor_create_surface(compositor);
    struct xdg_surface *xs = xdg_wm_base_get_xdg_surface(wm_base, surface);
    xdg_surface_add_listener(xs, &xdg_surf_listener, NULL);
    struct xdg_toplevel *top = xdg_surface_get_toplevel(xs);
    xdg_toplevel_add_listener(top, &toplevel_listener, NULL);
    xdg_toplevel_set_title(top, "wlkbdprobe");
    wl_surface_commit(surface);
    while (wl_display_dispatch(dpy) != -1) {}
    return 0;
}
