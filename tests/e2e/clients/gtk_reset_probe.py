#!/usr/bin/env python3
"""GTK3 re-entrant-reset probe for the kime e2e harness (#562 shape).

Usage: gtk_reset_probe.py <outprefix> [--reset-in-commit]

Gtk.Entry/TextView hide their im-context, so this client creates a
Gtk.IMMulticontext directly on a plain toplevel Gtk.Window (set_client_window
on the toplevel's GdkWindow), forwards raw key events with filter_keypress —
both passes: kime's GTK3 immodule re-queues events with marker bit 1<<25
(HANDLED_MASK) via gdk_event_put and expects the client to filter them again —
and appends:

  <out>.commits   one line per "commit" signal:      d<depth>:<text>
                  (depth = commit-handler nesting; d2+ = re-entrant emission)
  <out>.preedit   one line per preedit-changed:      p:<preedit>
  <out>.events    key/handled trace + reset markers

With --reset-in-commit the commit handler calls im.reset() — the pattern of
kime#562 (Firefox resets the IM context from its commit path). The reset is
guarded to depth 1: process_input_result clears the engine commit buffer only
AFTER the signal emission, so an unguarded handler would recurse without bound
(each nested kime_reset re-reads the still-uncleared commit string).
Depth-1-only still exposes the bug as exactly one duplicate line.

Prints CONTEXT_ID:<id> (which slave GTK picked — must be "kime") and READY on
stderr once the window is mapped and the im-context focused.
"""
import sys

import gi

gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
from gi.repository import Gdk, Gtk

out = sys.argv[1]
do_reset = '--reset-in-commit' in sys.argv[2:]


def log(suffix, line):
    # Single append per line, matching gtk_probe.py's preedit log convention
    # (the harness only ever parses whole lines).
    with open(out + suffix, 'a') as f:
        f.write(line + '\n')


im = Gtk.IMMulticontext()
depth = 0


def on_commit(ctx, text):
    global depth
    depth += 1
    log('.commits', 'd%d:%s' % (depth, text))
    if do_reset and depth == 1:
        log('.events', 'reset-call')
        ctx.reset()
        log('.events', 'reset-returned')
    depth -= 1


def on_preedit_changed(ctx):
    s, _attrs, _pos = ctx.get_preedit_string()
    log('.preedit', 'p:%s' % s)


im.connect('commit', on_commit)
im.connect('preedit-changed', on_preedit_changed)

win = Gtk.Window(title='kimeprobe')
win.set_default_size(320, 240)
win.add_events(Gdk.EventMask.KEY_PRESS_MASK | Gdk.EventMask.KEY_RELEASE_MASK)


def on_key(_w, event):
    handled = im.filter_keypress(event)
    log('.events', 'key:%s code=%d state=0x%x handled=%s' %
        (Gdk.keyval_name(event.keyval), event.hardware_keycode,
         int(event.state), handled))
    return handled


win.connect('key-press-event', on_key)
win.connect('key-release-event', on_key)


def on_map(w, _ev):
    im.set_client_window(w.get_window())
    im.focus_in()
    # context id proves which slave GTK picked (must be "kime")
    sys.stderr.write('CONTEXT_ID:%s\n' % im.get_context_id())
    sys.stderr.write('READY\n')
    sys.stderr.flush()
    return False


win.connect('map-event', on_map)
win.connect('destroy', Gtk.main_quit)
win.show_all()
Gtk.main()
