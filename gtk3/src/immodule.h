#pragma once
#include <gtk/gtk.h>
#include <gdk/gdk.h>

void register_module(GTypeModule* module);
GType get_kime_ty();
