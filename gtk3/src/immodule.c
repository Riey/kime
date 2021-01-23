#include "immodule.h"

#include <kime_engine.h>

static GType KIME_TYPE_IM_CONTEXT = 0;
static const guint FORWARDED_MASK = 1 << 25;
static const guint SKIP_MASK = GDK_MOD1_MASK | GDK_MOD2_MASK | GDK_MOD3_MASK |
                               GDK_MOD4_MASK | GDK_MOD5_MASK;

typedef struct KimeSignals {
  guint commit;
  guint preedit_start;
  guint preedit_changed;
  guint preedit_end;
} KimeSignals;

typedef struct KimeImContextClass {
  GtkIMContextClass parent;
  KimeSignals signals;
  Config *config;
} KimeImContextClass;

typedef struct KimeImContext {
  GtkIMContext parent;
  GdkWindow *client_window;
  KimeSignals signals;
  InputEngine *engine;
  gboolean preedit_visible;
  Config *config;
} KimeImContext;

#define KIME_IM_CONTEXT(var)                                                   \
  KimeImContext *ctx =                                                         \
      G_TYPE_CHECK_INSTANCE_CAST(var, KIME_TYPE_IM_CONTEXT, KimeImContext)

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

void commit(KimeImContext *ctx, uint32_t ch) {
  g_debug("commit: %u", ch);

  gchar buf[8] = {0};
  g_unichar_to_utf8(ch, buf);
  g_signal_emit(ctx, ctx->signals.commit, 0, &buf[0]);
}

void reset(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  g_debug("reset");

  uint32_t c = kime_engine_reset(ctx->engine);

  if (!c) {
    update_preedit(ctx, FALSE);
    commit(ctx, c);
  }
}

void put_event(GdkEventKey *key) {
  key->state |= FORWARDED_MASK;
  gdk_event_put(gdk_event_copy((GdkEvent *)key));
}

gboolean commit_event(KimeImContext *ctx, GdkEventKey *key) {
  // ignore LOCK or NUMLOCK
  guint state = key->state & !(GDK_LOCK_MASK | GDK_MOD2_MASK);

  if (!key->state || key->state == GDK_SHIFT_MASK) {
    uint32_t c = gdk_keyval_to_unicode(key->keyval);

    if (!g_unichar_iscntrl(c)) {
      commit(ctx, c);
      return TRUE;
    }
  }

  return FALSE;
}

gboolean bypass(KimeImContext *ctx, GdkEventKey *key) {
  uint32_t c = kime_engine_reset(ctx->engine);

  if (!c) {
    update_preedit(ctx, FALSE);
    commit(ctx, c);
    put_event(key);
    return TRUE;
  } else {
    return FALSE;
  }
}

gboolean filter_keypress(GtkIMContext *im, GdkEventKey *key) {
  KIME_IM_CONTEXT(im);

  if (key->type != GDK_KEY_PRESS) {
    return FALSE;
  } else if (key->state & FORWARDED_MASK) {
    g_debug("Forwarded: %u", key->keyval);
    return commit_event(ctx, key);
  } else if (key->state & SKIP_MASK) {
    return bypass(ctx, key);
  } else {
    ModifierState state = 0;

    if (key->state & GDK_SHIFT_MASK) {
      state |= ModifierState_SHIFT;
    }

    if (key->state & GDK_CONTROL_MASK) {
      state |= ModifierState_CONTROL;
    }

    if (key->state & GDK_MOD4_MASK) {
      state |= ModifierState_SUPER;
    }

    InputResult ret = kime_engine_press_key(ctx->engine, ctx->config,
                                            key->hardware_keycode, state);

    switch (ret.ty) {
    case Commit:
      update_preedit(ctx, FALSE);
      commit(ctx, ret.char1);
      return TRUE;
    case CommitCommit:
      update_preedit(ctx, FALSE);
      commit(ctx, ret.char1);
      commit(ctx, ret.char2);
      return TRUE;
    case CommitBypass:
      update_preedit(ctx, FALSE);
      commit(ctx, ret.char1);
      // try commit english first
      if (!commit_event(ctx, key)) {
        put_event(key);
      }
      return TRUE;
    case CommitPreedit:
      commit(ctx, ret.char1);
    case Preedit:
      update_preedit(ctx, TRUE);
      return TRUE;
    case ClearPreedit:
      update_preedit(ctx, FALSE);
      return TRUE;
    case Consume:
      return TRUE;
    case Bypass:
    default:
      return commit_event(ctx, key);
    }
  }
}

void set_client_window(GtkIMContext *im, GdkWindow *window) {
  KIME_IM_CONTEXT(im);

  if (ctx->client_window) {
    g_object_unref(ctx->client_window);
  }

  if (window) {
    g_object_ref(window);
  }

  ctx->client_window = window;
}

void get_preedit_string(GtkIMContext *im, gchar **out, PangoAttrList **attrs,
                        int *cursor_pos) {
  KIME_IM_CONTEXT(im);
  uint32_t c = 0;
  size_t str_len = 0;

  if (out) {
    c = kime_engine_preedit_char(ctx->engine);

    if (!c) {
      // Nothing to display
      if (cursor_pos) {
        *cursor_pos = 0;
      }
      *out = g_strdup("");
    } else {
      if (cursor_pos) {
        *cursor_pos = 1;
      }
      gchar *s = g_malloc0(8);
      memset(s, 0, 8);
      g_unichar_to_utf8(c, s);
      str_len = strlen(s);
      *out = s;
    }
  }

  if (attrs) {
    *attrs = pango_attr_list_new();

    if (out && c) {
      PangoAttribute *attr = pango_attr_underline_new(PANGO_UNDERLINE_SINGLE);
      attr->start_index = 0;
      attr->end_index = str_len;
      pango_attr_list_insert(*attrs, attr);

      // TODO: color
    }
  }
}

void im_context_class_finalize(KimeImContextClass *klass, gpointer _data) {
  kime_config_delete(klass->config);
}

void im_context_init(KimeImContext *ctx, KimeImContextClass *klass) {
  ctx->client_window = NULL;
  ctx->signals = klass->signals;
  ctx->engine = kime_engine_new();
  ctx->config = klass->config;
}

void im_context_finalize(GObject *obj) {
  KIME_IM_CONTEXT(obj);
  kime_engine_delete(ctx->engine);
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

  klass->parent.set_client_window = set_client_window;
  klass->parent.parent_class.finalize = im_context_finalize;
  klass->parent.reset = reset;
  klass->parent.filter_keypress = filter_keypress;
  klass->parent.get_preedit_string = get_preedit_string;
  // klass->parent.focus_in = NULL;
  klass->parent.focus_out = reset;
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
