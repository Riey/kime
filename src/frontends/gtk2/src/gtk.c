#include "./immodule.h"

static const GtkIMContextInfo INFO = {
    .context_id = "kime",
    .context_name = "Kime (Korean IME)",
    .domain = "kime",
    .domain_dirname = "/usr/share/locale",
    .default_locales = "ko:*",
};
static const GtkIMContextInfo *INFOS[] = {&INFO};

G_MODULE_EXPORT const gchar *g_module_check_init(GModule *module) {
  if (kime_api_version() == 1) {
    return NULL;
  } else {
    return "Engine version mismatched";
  }
}

G_MODULE_EXPORT void im_module_exit(void) {}

G_MODULE_EXPORT void im_module_init(GTypeModule *type_module) {
  g_type_module_use(type_module);
  register_module(type_module);

  kime_enable_logger_with_env();
}

G_MODULE_EXPORT void im_module_list(const GtkIMContextInfo ***contexts,
                                    int *n_contexts) {
  *contexts = INFOS;
  *n_contexts = G_N_ELEMENTS(INFOS);
}

G_MODULE_EXPORT GtkIMContext *im_module_create(const gchar *context_id) {
  if (g_strcmp0(context_id, "kime") == 0) {
    GType ty = get_kime_ty();
    if (ty) {
      gpointer obj = g_object_new(ty, NULL);
      return GTK_IM_CONTEXT(obj);
    }
  }

  return NULL;
}
