#pragma once
#include <gdk/gdk.h>
#include <gtk/gtk.h>
#include <gtk/gtkimmodule.h>

void register_module(GTypeModule *module);
GType get_kime_ty();
