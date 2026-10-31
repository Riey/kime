#include "immodule.h"
#include "str_buf.h"

#include <stdio.h>

static GType KIME_TYPE_IM_CONTEXT = 0;
static GObjectClass *kime_parent_class = NULL;
#if !GTK_CHECK_VERSION(3, 98, 4)
// for many buggy gtk apps
static const guint HANDLED_MASK = 1 << 25;
static const guint BYPASS_MASK = 1 << 24;
#endif

#if GTK_CHECK_VERSION(3, 98, 4)
typedef GtkWidget ClientType;
typedef GdkEvent EventType;
#else
#define GDK_ALT_MASK GDK_MOD1_MASK
typedef GdkWindow ClientType;
typedef GdkEventKey EventType;
#if GTK_CHECK_VERSION(3, 0, 0)
gboolean gdk_device_get_num_lock_state (GdkDevice *device)
{
  GdkKeymap *keymap = gdk_keymap_get_for_display(gdk_device_get_display(device));
  return gdk_keymap_get_num_lock_state(keymap);
}
#else
gboolean gdk_device_get_num_lock_state (GdkDevice *device)
{
  return FALSE;
}
#endif
#endif

static const guint NOT_ENGLISH_MASK =
    GDK_ALT_MASK | GDK_CONTROL_MASK | GDK_SUPER_MASK;

typedef struct _KeyRet {
  gboolean bypassed;
  gboolean has_preedit;
} KeyRet;

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
  GtkWidget *widget;
  KimeSignals signals;
  KimeInputEngine *engine;
  gboolean preedit_visible;

  gboolean engine_ready;
  KimeConfig *config;
} KimeImContext;

#define KIME_IM_CONTEXT(var)                                                   \
  KimeImContext *ctx =                                                         \
      G_TYPE_CHECK_INSTANCE_CAST(var, KIME_TYPE_IM_CONTEXT, KimeImContext)

#define debug(...) g_log("kime", G_LOG_LEVEL_DEBUG, __VA_ARGS__)

void update_preedit(KimeImContext *ctx) {
  KimeRustStr str = kime_engine_preedit_str(ctx->engine);

  gboolean visible = str.len != 0;
  debug("preedit(%d)", visible);

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
  // Don't commit zero size string
  if (ctx->buf.len == 0) {
    return;
  }

// hide commit string when not debug
#ifdef DEBUG
  debug("commit(%zu): `%s`", ctx->buf.len, ctx->buf.ptr);
#else
  debug("commit(%zu)", ctx->buf.len);
#endif

  g_signal_emit(ctx, ctx->signals.commit, 0, ctx->buf.ptr);

  // clear commit buffer
  ctx->buf.len = 0;
}

KeyRet process_input_result(KimeImContext *ctx, KimeInputResult ret) {
  KeyRet key_ret;
  key_ret.bypassed = (ret & KimeInputResult_CONSUMED) == 0;
  key_ret.has_preedit = (ret & KimeInputResult_HAS_PREEDIT) != 0;

  if (ret & KimeInputResult_NOT_READY) {
    ctx->engine_ready = FALSE;

    // blocking mode
    // bool engine_ready = false;
    // while (!engine_ready) {
    //   engine_ready = kime_engine_check_ready(ctx->engine);
    // }
    // ret = kime_engine_end_ready(ctx->engine);
  }

  if (ret & KimeInputResult_LANGUAGE_CHANGED) {
    kime_engine_update_layout_state(ctx->engine);
  }

  if (ret & KimeInputResult_HAS_COMMIT) {
    // The "commit" handler may drop the last reference to this context (e.g.
    // an app destroying the IM context from the handler); keep ctx and
    // ctx->engine alive until we are done with them.
    g_object_ref(ctx);
    // Snapshot the commit string and clear the engine buffer BEFORE emitting:
    // a client may call gtk_im_context_reset() from inside its "commit"
    // handler (the #562 Firefox pattern), re-entering kime_reset while this
    // emission is still on the stack. With emit-then-clear the re-entrant
    // reset would read the still-populated engine buffer and commit the same
    // string again (#563 guarded this with is_committing; #570 removed the
    // guard). With the buffer already cleared, a re-entrant reset only
    // commits the live preedit, which is the correct reset semantic.
    str_buf_set_str(&ctx->buf, kime_engine_commit_str(ctx->engine));
    kime_engine_clear_commit(ctx->engine);
    commit(ctx);
    g_object_unref(ctx);
  }

  return key_ret;
}

void focus_in(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("focus_in");

  kime_engine_update_layout_state(ctx->engine);

  if (!ctx->engine_ready) {
    if (kime_engine_check_ready(ctx->engine)) {
      process_input_result(ctx, kime_engine_end_ready(ctx->engine));
      ctx->engine_ready = TRUE;
    }
  }
}

void kime_reset(KimeImContext *ctx) {
  kime_engine_clear_preedit(ctx->engine);
  str_buf_set_str(&ctx->buf, kime_engine_commit_str(ctx->engine));
  // Reset the engine (which clears its commit buffer) BEFORE emitting so a
  // re-entrant reset from the "commit" handler sees an empty engine and
  // becomes a no-op instead of re-committing the same string (#562; the
  // #563 is_committing guard for this was removed in #570). commit() skips
  // empty snapshots, so a bare reset emits nothing.
  kime_engine_reset(ctx->engine);
  commit(ctx);
}

void reset(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("reset");

  kime_reset(ctx);
}

void focus_out(GtkIMContext *im) {
  KIME_IM_CONTEXT(im);

  debug("focus_out");

  // Don't do reset when engine is not ready
  if (ctx->engine_ready) {
    kime_reset(ctx);
  }
}

#if !GTK_CHECK_VERSION(3, 98, 4)
// GTK3 only: re-queue the event with marker bits so the app sees it again and
// the second pass through filter_keypress finishes the work (workaround for
// buggy GTK3 apps that can't handle preedit updates mixed with commits).
// GTK4 has no gdk_event_put(); re-injecting through
// gtk_im_context_filter_key() would only re-enter this IM context without ever
// reaching the widget, so the GTK4 build handles everything synchronously in
// filter_keypress instead.
void put_event(KimeImContext *ctx, EventType *key, guint mask) {
  key->state |= mask;
  gdk_event_put((GdkEvent *)key);
}
#endif

gboolean commit_event(KimeImContext *ctx, GdkModifierType state, guint keyval) {
  // Try english commit directly(for apps which can't handle this e.g. gedit)
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

KeyRet on_key_input(KimeImContext *ctx, guint16 code, bool numlock, KimeModifierState state) {
  KimeInputResult ret =
      kime_engine_press_key(ctx->engine, ctx->config, code, numlock, state);

#if DEBUG
  debug("(%d, %d, %d)", code, state, ret);
#endif

  return process_input_result(ctx, ret);
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
  GdkDevice* device = gdk_event_get_device(key);
#else
  if (key->type != GDK_KEY_PRESS) {
    return FALSE;
  }
  guint16 code = key->hardware_keycode;
  guint keyval = key->keyval;
  GdkModifierType state = key->state;
  GdkDevice* device = gdk_event_get_device((GdkEvent*)key);
#endif

  // A signal handler may drop the last reference to this context (e.g.
  // Inkscape destroys the IM context from its "commit" handler during tool
  // teardown); keep it alive while signals are emitted and its fields are
  // still used.
  g_object_ref(ctx);
  gboolean ret;

#if !GTK_CHECK_VERSION(3, 98, 4)
  // delayed event
  if (state & HANDLED_MASK) {
    // preedit change can't mixed with commit
    update_preedit(ctx);

    if (state & BYPASS_MASK) {
      ret = commit_event(ctx, state, keyval);
    } else {
      ret = TRUE;
    }

    g_object_unref(ctx);
    return ret;
  }
#endif

  bool numlock = gdk_device_get_num_lock_state(device) == TRUE;

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

  KeyRet key_ret = on_key_input(ctx, code, numlock, kime_state);

  if (ctx->preedit_visible || key_ret.has_preedit) {
#if GTK_CHECK_VERSION(3, 98, 4)
    // The GTK3-style deferral would swallow bypassed keys (Enter, Tab,
    // arrows) here: gtk_im_context_filter_key() never reaches the widget and
    // the outer `TRUE` marks the event handled. The deferral only works
    // around GTK3 app quirks, so update the preedit synchronously and let the
    // widget handle bypassed control keys itself.
    update_preedit(ctx);

    if (key_ret.bypassed) {
      ret = commit_event(ctx, state, keyval);
    } else {
      ret = TRUE;
    }
#else
    guint mask = HANDLED_MASK;

    if (key_ret.bypassed) {
      mask |= BYPASS_MASK;
    }

    // need change on next event
    put_event(ctx, key, mask);

    // debug("trip: preedit cur(%d) will(%d))", ctx->preedit_visible, key_ret.has_preedit);

    // never return `FALSE` here
    ret = TRUE;
#endif
  } else if (key_ret.bypassed) {
    // debug("commit_event");
    ret = commit_event(ctx, state, keyval);
  } else {
    // debug("consume");
    ret = TRUE;
  }

  g_object_unref(ctx);
  return ret;
}

GtkWidget *client_get_widget(ClientType *client) {
#if GTK_CHECK_VERSION(3, 98, 4)
  return client;
#else
  while (client) {
    gpointer user_data;
    gdk_window_get_user_data(client, &user_data);
    if (user_data)
      return user_data;
    client = gdk_window_get_parent(client);
  }
  return NULL;
#endif
}

gboolean client_button_press(GtkWidget *widget, GdkEvent *event,
                             gpointer user_data) {
  debug("button");
  KimeImContext *ctx = (KimeImContext *)user_data;
  kime_reset(ctx);

  return FALSE;
}

void set_client(GtkIMContext *im, ClientType *client) {
  KIME_IM_CONTEXT(im);
  GtkWidget *widget = client_get_widget(client);

  if (ctx->widget) {
    g_signal_handlers_disconnect_by_func(ctx->widget,
                                         (GCallback)client_button_press, ctx);
    g_object_unref(ctx->widget);
  }
  if (widget) {
    g_signal_connect(widget, "button-press-event",
                     (GCallback)client_button_press, ctx);
    g_object_ref(widget);
  }
  ctx->widget = widget;
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

void im_context_class_finalize(KimeImContextClass *klass, gpointer _data) {
  kime_config_delete(klass->config);
}

void im_context_init(KimeImContext *ctx, KimeImContextClass *klass) {
  ctx->buf = str_buf_new();
  ctx->widget = NULL;
  ctx->preedit_visible = FALSE;
  ctx->engine_ready = TRUE;
  ctx->signals = klass->signals;
  ctx->engine = kime_engine_new(klass->config);
  ctx->config = klass->config;
}

void im_context_finalize(GObject *obj) {
  KIME_IM_CONTEXT(obj);
  str_buf_delete(&ctx->buf);
  if (ctx->widget) {
    // set_client(NULL) may never be called; drop our handler here so a
    // widget outliving this context can't call back into freed memory.
    g_signal_handlers_disconnect_by_func(ctx->widget,
                                         (GCallback)client_button_press, ctx);
    g_object_unref(ctx->widget);
    ctx->widget = NULL;
  }
  kime_engine_delete(ctx->engine);
  kime_parent_class->finalize(obj);
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

  kime_parent_class = G_OBJECT_CLASS(g_type_class_peek_parent(klass));

  GObjectClass *gobject_class = G_OBJECT_CLASS(klass);
  gobject_class->finalize = im_context_finalize;
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
