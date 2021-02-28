#include "immodule.h"
#include "str_buf.h"

#include <stdio.h>

static GType KIME_TYPE_IM_CONTEXT = 0;
static const guint FORWARDED_MASK = 1 << 25;

#if GTK_CHECK_VERSION(3, 98, 4)
typedef GtkWidget ClientType;
typedef GdkEvent EventType;
#else
#define GDK_ALT_MASK GDK_MOD1_MASK
typedef GdkWindow ClientType;
typedef GdkEventKey EventType;
#endif

static const guint NOT_ENGLISH_MASK =
    GDK_ALT_MASK | GDK_CONTROL_MASK | GDK_SUPER_MASK;

typedef struct KimeSignals {
  guint commit;
  guint preedit_start;
  guint preedit_changed;
  guint preedit_end;
} KimeSignals;

typedef struct KimeImContextClass {
  GtkIMContextClass parent;
  KimeSignals signals;
  KimeConfig *config;
} KimeImContextClass;

typedef struct KimeImContext {
  GtkIMContext parent;
  StrBuf buf;
  ClientType *client;
  KimeSignals signals;
  KimeInputEngine *engine;
  gboolean focus;
  gboolean preedit_visible;
  KimeConfig *config;
} KimeImContext;

#define KIME_IM_CONTEXT(var)                                                   \
  KimeImContext *ctx =                                                         \
      G_TYPE_CHECK_INSTANCE_CAST(var, KIME_TYPE_IM_CONTEXT, KimeImContext)

#define debug(...) g_log("kime", G_LOG_LEVEL_DEBUG, __VA_ARGS__)

void update_preedit(KimeImContext *ctx, gboolean visible) {
  if (ctx->preedit_visible != visible) {
    ctx->preedit_visible = visible;

    if (visible) {
      g_signal_emit(ctx, ctx->signals.preedit_start, 0);
      g_signal_emit(ctx, ctx->signals.preedit_changed, 0);
    } else {
      g_signal_emit(ctx, ctx->signals.preedit_changed, 0);
      g_signal_emit(ctx, ctx->signals.preedit_end, 0);
    }
  } else if (visible) {
    g_signal_emit(ctx, ctx->signals.preedit_changed, 0);
  }
}

void commit(KimeImContext *ctx) {
  g_signal_emit(ctx, ctx->signals.commit, 0, ctx->buf.ptr);
}

void focus_in(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("focus_in");

  ctx->focus = TRUE;
  kime_engine_update_hangul_state(ctx->engine);
}

void kime_reset(KimeImContext *ctx) {
  update_preedit(ctx, FALSE);
  kime_engine_clear_preedit(ctx->engine);
  str_buf_set_str(&ctx->buf, kime_engine_commit_str(ctx->engine));
  commit(ctx);
  kime_engine_reset(ctx->engine);
}

void reset(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("reset");

  kime_reset(ctx);
}

void focus_out(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("focus_out");
  ctx->focus = FALSE;

  kime_reset(ctx);
}

void put_event(KimeImContext *ctx, EventType *key) {
#if GTK_CHECK_VERSION(3, 98, 4)
  gtk_im_context_filter_key(
      GTK_IM_CONTEXT(ctx), gdk_event_get_event_type(key) == GDK_KEY_PRESS,
      gdk_event_get_surface(key), gdk_event_get_device(key),
      gdk_event_get_time(key), gdk_key_event_get_keycode(key),
      gdk_event_get_modifier_state(key) | FORWARDED_MASK, 0);
#else
  key->state |= FORWARDED_MASK;
  gdk_event_put(gdk_event_copy((GdkEvent *)key));
#endif
}

gboolean commit_event(KimeImContext *ctx, GdkModifierType state, guint keyval) {
  if (!(state & NOT_ENGLISH_MASK)) {
    uint32_t c = gdk_keyval_to_unicode(keyval);

    if (!g_unichar_iscntrl(c)) {
      str_buf_set_ch(&ctx->buf, c);
      commit(ctx);
      return TRUE;
    }
  }

  return FALSE;
}

gboolean on_key_input(KimeImContext *ctx, guint16 code,
                      KimeModifierState state) {
  KimeInputResult ret =
      kime_engine_press_key(ctx->engine, ctx->config, code, state);

  if (ret & KimeInputResult_LANGUAGE_CHANGED) {
    kime_engine_update_hangul_state(ctx->engine);
  }

  if (!(ret & KimeInputResult_HAS_PREEDIT)) {
    update_preedit(ctx, FALSE);
  }

  if (ret & (KimeInputResult_NEED_RESET | KimeInputResult_NEED_FLUSH)) {
    str_buf_set_str(&ctx->buf, kime_engine_commit_str(ctx->engine));
    commit(ctx);

    if (ret & KimeInputResult_NEED_RESET) {
      kime_engine_reset(ctx->engine);
    } else {
      kime_engine_flush(ctx->engine);
    }
  }

  if (ret & KimeInputResult_HAS_PREEDIT) {
    update_preedit(ctx, TRUE);
  }

  return (ret & KimeInputResult_CONSUMED) != 0;
}

gboolean filter_keypress(GtkIMContext *im, EventType *key) {
  KIME_IM_CONTEXT(im);
#if GTK_CHECK_VERSION(3, 98, 4)
  if (gdk_event_get_event_type(key) != GDK_KEY_PRESS) {
    return FALSE;
  }
  guint16 code = gdk_key_event_get_keycode(key);
  guint keyval = gdk_key_event_get_keyval(key);
  GdkModifierType state = gdk_event_get_modifier_state(key);
#else
  if (key->type != GDK_KEY_PRESS) {
    return FALSE;
  }
  guint16 code = key->hardware_keycode;
  guint keyval = key->keyval;
  GdkModifierType state = key->state;
#endif

  KimeModifierState kime_state = 0;

  if (state & GDK_SHIFT_MASK) {
    kime_state |= KimeModifierState_SHIFT;
  }

  if (state & GDK_ALT_MASK) {
    kime_state |= KimeModifierState_ALT;
  }

  if (state & GDK_CONTROL_MASK) {
    kime_state |= KimeModifierState_CONTROL;
  }

  if (state & GDK_SUPER_MASK) {
    kime_state |= KimeModifierState_SUPER;
  }

  return on_key_input(ctx, code, kime_state) ||
         commit_event(ctx, state, keyval);
}

void set_client(GtkIMContext *im, ClientType *client) {
  KIME_IM_CONTEXT(im);

  if (ctx->client) {
    g_object_unref(ctx->client);
  }

  if (client) {
    g_object_ref(client);
  }

  ctx->client = client;
}

void get_preedit_string(GtkIMContext *im, gchar **out, PangoAttrList **attrs,
                        int *cursor_pos) {
  KIME_IM_CONTEXT(im);
  KimeRustStr s = kime_engine_preedit_str(ctx->engine);

  if (out) {
    if (s.len == 0 || !ctx->preedit_visible) {
      // Nothing to display
      if (cursor_pos) {
        *cursor_pos = 0;
      }
      *out = g_strdup("");
    } else {
      gchar *g_s = g_malloc0(s.len + 1);
      g_s[s.len] = '\0';
      memcpy(g_s, s.ptr, s.len);

      if (cursor_pos) {
        *cursor_pos = g_utf8_strlen(g_s, -1);
      }
      *out = g_s;
    }
  }

  if (attrs) {
    *attrs = pango_attr_list_new();

    if (out && ctx->preedit_visible && s.len) {
      PangoAttribute *attr = pango_attr_underline_new(PANGO_UNDERLINE_SINGLE);
      attr->start_index = 0;
      attr->end_index = s.len;
      pango_attr_list_insert(*attrs, attr);
    }
  }
}

#if !GTK_CHECK_VERSION(3, 98, 4)
// workaround click bug perhaps not occured in gtk4 see #282
GdkFilterReturn global_filter_event(GdkXEvent *xevent, GdkEvent *event,
                                    gpointer data) {
  KimeImContext *ctx = (KimeImContext *)data;
  XEvent *native_event = (XEvent *)xevent;

  if (!ctx->focus || !ctx->client) {
    return GDK_FILTER_CONTINUE;
  }

  if (native_event->type == ButtonPress) {
    kime_reset(ctx);
  } else if (native_event->type == KeyPress) {
    XKeyPressedEvent *kev = (XKeyPressedEvent *)xevent;
    KimeModifierState state = 0;

    if (kev->state & 0x1) {
      state |= KimeModifierState_SHIFT;
    }

    if (kev->state & 0x4) {
      state |= KimeModifierState_CONTROL;
    }

    if (kev->state & 0x8) {
      state |= KimeModifierState_ALT;
    }

    if (kev->state & 0x40) {
      state |= KimeModifierState_SUPER;
    }

    if (on_key_input(ctx, (guint16)kev->keycode, state)) {
      // handled
      return GDK_FILTER_REMOVE;
    }
  }
  return GDK_FILTER_CONTINUE;
}
#endif

void im_context_class_finalize(KimeImContextClass *klass, gpointer _data) {
  kime_config_delete(klass->config);
}

void im_context_init(KimeImContext *ctx, KimeImContextClass *klass) {
  ctx->buf = str_buf_new();
  ctx->client = NULL;
  ctx->focus = FALSE;
  ctx->signals = klass->signals;
  ctx->engine = kime_engine_new(klass->config);
  ctx->config = klass->config;

#if !GTK_CHECK_VERSION(3, 98, 4)
  gdk_window_add_filter(NULL, global_filter_event, ctx);
#endif
}

void im_context_finalize(GObject *obj) {
  KIME_IM_CONTEXT(obj);
  str_buf_delete(&ctx->buf);
  if (ctx->client) {
    g_object_unref(ctx->client);
    ctx->client = NULL;
  }
  kime_engine_delete(ctx->engine);

#if !GTK_CHECK_VERSION(3, 98, 4)
  gdk_window_remove_filter(NULL, global_filter_event, ctx);
#endif
}

void im_context_class_init(KimeImContextClass *klass, gpointer _data) {
  klass->signals.commit = g_signal_lookup("commit", KIME_TYPE_IM_CONTEXT);
  klass->signals.preedit_start =
      g_signal_lookup("preedit-start", KIME_TYPE_IM_CONTEXT);
  klass->signals.preedit_changed =
      g_signal_lookup("preedit-changed", KIME_TYPE_IM_CONTEXT);
  klass->signals.preedit_end =
      g_signal_lookup("preedit-end", KIME_TYPE_IM_CONTEXT);

  klass->config = kime_config_load();

#if GTK_CHECK_VERSION(3, 98, 4)
  klass->parent.set_client_widget = set_client;
#else
  klass->parent.set_client_window = set_client;
#endif
  klass->parent.reset = reset;
  klass->parent.filter_keypress = filter_keypress;
  klass->parent.get_preedit_string = get_preedit_string;
  klass->parent.focus_in = focus_in;
  klass->parent.focus_out = focus_out;

  GObjectClass *parent_class = G_OBJECT_CLASS(klass);
  parent_class->finalize = im_context_finalize;
}

static const GTypeInfo TYPE_INFO = {
    .class_size = sizeof(KimeImContextClass),
    .base_init = NULL,
    .base_finalize = NULL,
    .class_init = (GClassInitFunc)im_context_class_init,
    .class_finalize = (GClassFinalizeFunc)im_context_class_finalize,
    .class_data = NULL,
    .instance_size = sizeof(KimeImContext),
    .instance_init = (GInstanceInitFunc)im_context_init,
    .value_table = NULL};

void register_module(GTypeModule *module) {
  if (module) {
    KIME_TYPE_IM_CONTEXT = g_type_module_register_type(
        module, gtk_im_context_get_type(), "KimeIMContext", &TYPE_INFO, 0);
  } else {
    KIME_TYPE_IM_CONTEXT = g_type_register_static(
        gtk_im_context_get_type(), "KimeIMContext", &TYPE_INFO, 0);
  }
}

GType get_kime_ty() { return KIME_TYPE_IM_CONTEXT; }
