#!/usr/bin/env python3
"""GTK3/GTK4 text probe for the kime e2e harness.

Usage: gtk_probe.py <3|4> <outfile> [--textview]

Every 150ms:
  <outfile>          committed widget text (truncated rewrite)
  <outfile>.cursor   current cursor offset in characters (truncated rewrite)
On every preedit change:
  <outfile>.preedit  one appended line "p:<preedit string>"

Prints READY on stderr once the window is presented and the widget focused.

--textview uses a Gtk.TextView instead of a Gtk.Entry — Entries cannot take
newlines/tabs, which the bypassed-key tests (#606/#613, PR #775) need.
GTK3 TextView has no preedit-changed signal; the probe logs that to stderr
and still dumps buffer + cursor.
"""
import os
import sys

ver = sys.argv[1]
out = sys.argv[2]
textview = '--textview' in sys.argv[3:]

import gi


def log_preedit(s):
    with open(out + '.preedit', 'a') as f:
        f.write('p:%s\n' % s)


def hook_preedit(widget):
    try:
        widget.connect('preedit-changed', lambda w, s: log_preedit(s))
    except Exception as e:
        sys.stderr.write('no preedit-changed: %r\n' % e)


def atomic_write(path, data):
    # Atomic replace so readers never observe a truncated/empty window
    # (a plain truncate-then-write raced the Rust BufferWatcher).
    tmp = path + '.tmp'
    with open(tmp, 'w') as f:
        f.write(data)
    os.replace(tmp, path)


def dump_files(text, cursor):
    atomic_write(out, text)
    atomic_write(out + '.cursor', str(cursor))
    return True


def ready():
    sys.stderr.write('READY\n')
    sys.stderr.flush()


if ver == '3':
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk, GLib
    import os
    style = os.environ.get('PROBE_PREEDIT_STYLE')
    if style:
        s = Gtk.Settings.get_default()
        s.set_property('gtk-im-preedit-style', getattr(Gtk.IMPreeditStyle, style.upper()))
        s.set_property('gtk-im-status-style', Gtk.IMStatusStyle.NOTHING)
    win = Gtk.Window(title='kimeprobe')
    win.set_default_size(320, 240)
    if textview:
        widget = Gtk.TextView()
        buf = widget.get_buffer()
        hook_preedit(widget)  # logs 'no preedit-changed' on GTK3

        def dump():
            return dump_files(
                buf.get_text(buf.get_start_iter(), buf.get_end_iter(), True),
                buf.props.cursor_position)
    else:
        widget = Gtk.Entry()
        hook_preedit(widget)

        def dump():
            return dump_files(widget.get_text(), widget.get_position())
    win.add(widget)
    win.connect('destroy', Gtk.main_quit)
    GLib.timeout_add(150, dump)
    win.show_all()
    widget.grab_focus()
    ready()
    Gtk.main()
else:
    gi.require_version('Gtk', '4.0')
    from gi.repository import Gtk, GLib, Gio
    # NON_UNIQUE: without it GApplication reaches the session bus (via
    # $XDG_RUNTIME_DIR/bus even under env_clear), and a concurrently running
    # probe elsewhere becomes the primary instance — this process then
    # delegates activation and exits 0 without ever printing READY.
    app = Gtk.Application(application_id='kime.probe',
                          flags=Gio.ApplicationFlags.NON_UNIQUE)

    def activate(app):
        win = Gtk.ApplicationWindow(application=app, title='kimeprobe')
        win.set_default_size(320, 240)
        if textview:
            widget = Gtk.TextView()
            buf = widget.get_buffer()
            hook_preedit(widget)  # GTK4 TextView has preedit-changed

            def dump():
                return dump_files(
                    buf.get_text(buf.get_start_iter(), buf.get_end_iter(), True),
                    buf.props.cursor_position)
        else:
            widget = Gtk.Entry()
            # preedit-changed lives on the GtkText delegate in GTK4
            try:
                hook_preedit(widget.get_delegate())
            except Exception:
                hook_preedit(widget)

            def dump():
                return dump_files(widget.get_text(), widget.get_position())
        win.set_child(widget)
        GLib.timeout_add(150, dump)
        win.present()
        widget.grab_focus()
        ready()

    app.connect('activate', activate)
    app.run(None)
