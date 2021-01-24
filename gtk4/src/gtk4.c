#include "../../gtk2/src/immodule.h"

G_MODULE_EXPORT void g_io_module_load(GIOModule *module) {
  GTypeModule* type_module = G_TYPE_MODULE(module);
  g_type_module_use(type_module);

  if (!get_kime_ty()) {
    register_module(type_module);
    g_io_extension_point_implement(GTK_IM_MODULE_EXTENSION_POINT_NAME, get_kime_ty(), "kime", 50);
  }
}

G_MODULE_EXPORT char** g_io_module_query(void) {
  char *eps[] = {
    GTK_IM_MODULE_EXTENSION_POINT_NAME,
    NULL,
  };
  return g_strdupv(eps);
}

G_MODULE_EXPORT void g_io_module_unload(GIOModule *module) {

}
