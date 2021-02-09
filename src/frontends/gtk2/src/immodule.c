#include "immodule.h"

#include <kime_engine.h>
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
  ClientType *client;
  KimeSignals signals;
  KimeInputEngine *engine;
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

void commit(KimeImContext *ctx, uint32_t ch) {
  debug("commit: %u", ch);

  gchar buf[8] = {0};
  g_unichar_to_utf8(ch, buf);
  g_signal_emit(ctx, ctx->signals.commit, 0, &buf[0]);
}

void focus_in(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  kime_engine_update_hangul_state(ctx->engine);
}

void reset(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("reset");

  uint32_t c = kime_engine_reset(ctx->engine);

  if (!c) {
    update_preedit(ctx, FALSE);
    commit(ctx, c);
  }
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
      commit(ctx, c);
      return TRUE;
    }
  }

  return FALSE;
}

gboolean bypass(KimeImContext *ctx, EventType *key) {
  uint32_t c = kime_engine_reset(ctx->engine);

  if (!c) {
    update_preedit(ctx, FALSE);
    commit(ctx, c);
    put_event(ctx, key);
    return TRUE;
  } else {
    return FALSE;
  }
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
  debug("code %u, state %u", code, state);

  if (state & FORWARDED_MASK) {
    debug("Forwarded: %u", keyval);
    return commit_event(ctx, state, keyval);
  } else {
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

    KimeInputResult ret =
        kime_engine_press_key(ctx->engine, ctx->config, code, kime_state);

    if (ret.hangul_changed) {
      kime_engine_update_hangul_state(ctx->engine);
    }

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
      if (!commit_event(ctx, state, keyval)) {
        put_event(ctx, key);
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
      return commit_event(ctx, state, keyval);
    }
  }
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
  ctx->client = NULL;
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

#if GTK_CHECK_VERSION(3, 98, 4)
  klass->parent.set_client_widget = set_client;
#else
  klass->parent.set_client_window = set_client;
#endif
  klass->parent.reset = reset;
  klass->parent.filter_keypress = filter_keypress;
  klass->parent.get_preedit_string = get_preedit_string;
  klass->parent.focus_in = focus_in;
  klass->parent.focus_out = reset;

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
