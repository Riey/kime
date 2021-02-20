#pragma once
#include <gdk/gdk.h>
#include <gtk/gtkimmodule.h>
#include <X11/Xlib.h>
#include <kime_engine.h>

void register_module(GTypeModule *module);
GType get_kime_ty();
