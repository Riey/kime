/* Persistent zwp_virtual_keyboard_v1 injector for the kime e2e harness.
 *
 * Reads commands from stdin (keep the pipe open, e.g. a fifo opened O_RDWR):
 *   k <evdev_code>                   tap (press + 25ms + release)
 *   p <evdev_code>                   press (hold)
 *   r <evdev_code>                   release
 *   d/u <evdev_code>                 legacy aliases for p/r
 *   m <dep> <latch> <lock> <group>   send a raw modifiers event (manual override)
 *   q                                quit
 *
 * Uploads a real xkbcommon keymap (rules=evdev, layout us) — kime-wayland
 * crashes if the seat has no keyboard/keymap when it grabs, so this client
 * MUST be connected before kime-wayland starts.
 *
 * Modifier state is tracked with xkb_state: every press/release feeds
 * xkb_state_update_key (evdev + 8) and, whenever the serialized
 * depressed/latched/locked mods or the effective group change, a matching
 * zwp_virtual_keyboard_v1.modifiers event is sent after the key event —
 * mirroring what a real compositor-facing keyboard does. This is what makes
 * Shift+Left selections and lone-modifier tests (e.g. #714, #760) work.
 */
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <sys/mman.h>
#include <wayland-client.h>
#include <xkbcommon/xkbcommon.h>
#include "virtual-keyboard-unstable-v1-client.h"

static struct wl_seat *seat;
static struct zwp_virtual_keyboard_manager_v1 *mgr;
static struct xkb_state *xstate;
static uint32_t last_dep, last_lat, last_lock, last_grp;

static void global_add(void *data, struct wl_registry *reg, uint32_t name,
                       const char *iface, uint32_t ver) {
    if (!strcmp(iface, wl_seat_interface.name))
        seat = wl_registry_bind(reg, name, &wl_seat_interface, 1);
    else if (!strcmp(iface, zwp_virtual_keyboard_manager_v1_interface.name))
        mgr = wl_registry_bind(reg, name, &zwp_virtual_keyboard_manager_v1_interface, 1);
}
static void global_remove(void *d, struct wl_registry *r, uint32_t n) {}
static const struct wl_registry_listener reg_listener = {global_add, global_remove};

static uint32_t now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint32_t)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);
}

static void sync_mods(struct zwp_virtual_keyboard_v1 *kbd) {
    uint32_t dep = xkb_state_serialize_mods(xstate, XKB_STATE_MODS_DEPRESSED);
    uint32_t lat = xkb_state_serialize_mods(xstate, XKB_STATE_MODS_LATCHED);
    uint32_t lock = xkb_state_serialize_mods(xstate, XKB_STATE_MODS_LOCKED);
    uint32_t grp = xkb_state_serialize_layout(xstate, XKB_STATE_LAYOUT_EFFECTIVE);
    if (dep != last_dep || lat != last_lat || lock != last_lock || grp != last_grp) {
        zwp_virtual_keyboard_v1_modifiers(kbd, dep, lat, lock, grp);
        last_dep = dep;
        last_lat = lat;
        last_lock = lock;
        last_grp = grp;
    }
}

static void key_event(struct zwp_virtual_keyboard_v1 *kbd, uint32_t code, int pressed) {
    zwp_virtual_keyboard_v1_key(kbd, now_ms(), code,
                                pressed ? WL_KEYBOARD_KEY_STATE_PRESSED
                                        : WL_KEYBOARD_KEY_STATE_RELEASED);
    /* xkb keycode space = evdev + 8 */
    xkb_state_update_key(xstate, code + 8, pressed ? XKB_KEY_DOWN : XKB_KEY_UP);
    sync_mods(kbd);
}

int main(void) {
    struct wl_display *dpy = wl_display_connect(NULL);
    if (!dpy) { fprintf(stderr, "connect failed\n"); return 1; }
    struct wl_registry *reg = wl_display_get_registry(dpy);
    wl_registry_add_listener(reg, &reg_listener, NULL);
    wl_display_roundtrip(dpy);
    if (!seat || !mgr) { fprintf(stderr, "missing seat/vk-manager\n"); return 2; }

    struct zwp_virtual_keyboard_v1 *kbd =
        zwp_virtual_keyboard_manager_v1_create_virtual_keyboard(mgr, seat);

    struct xkb_context *ctx = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
    struct xkb_rule_names names = {.rules = "evdev", .model = "pc105",
                                   .layout = "us", .variant = "", .options = ""};
    struct xkb_keymap *km = xkb_keymap_new_from_names(ctx, &names, XKB_KEYMAP_COMPILE_NO_FLAGS);
    if (!km) { fprintf(stderr, "keymap compile failed\n"); return 3; }
    xstate = xkb_state_new(km);
    char *kmstr = xkb_keymap_get_as_string(km, XKB_KEYMAP_FORMAT_TEXT_V1);
    size_t len = strlen(kmstr) + 1;
    int fd = memfd_create("kmap", 0);
    if (write(fd, kmstr, len) != (ssize_t)len) return 3;
    zwp_virtual_keyboard_v1_keymap(kbd, WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1, fd, len);
    wl_display_roundtrip(dpy);
    fprintf(stderr, "READY\n");

    char line[128];
    while (fgets(line, sizeof line, stdin)) {
        unsigned a, b, c, g;
        if (line[0] == 'q') break;
        if (sscanf(line, "k %u", &a) == 1) {
            key_event(kbd, a, 1);
            wl_display_flush(dpy);
            usleep(25000);
            key_event(kbd, a, 0);
        } else if (sscanf(line, "p %u", &a) == 1 || sscanf(line, "d %u", &a) == 1) {
            key_event(kbd, a, 1);
        } else if (sscanf(line, "r %u", &a) == 1 || sscanf(line, "u %u", &a) == 1) {
            key_event(kbd, a, 0);
        } else if (sscanf(line, "m %u %u %u %u", &a, &b, &c, &g) == 4) {
            zwp_virtual_keyboard_v1_modifiers(kbd, a, b, c, g);
            last_dep = a;
            last_lat = b;
            last_lock = c;
            last_grp = g;
        }
        wl_display_flush(dpy);
        wl_display_roundtrip(dpy);
    }
    zwp_virtual_keyboard_v1_destroy(kbd);
    wl_display_roundtrip(dpy);
    return 0;
}
