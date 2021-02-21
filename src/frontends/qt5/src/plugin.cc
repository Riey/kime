#include "plugin.hpp"
#include "input_context.hpp"

KimePlatformInputContextPlugin::KimePlatformInputContextPlugin() {
  if (kime::kime_api_version() != 1) {
    throw "Kime Engine version is mismatched!\n";
  }

  this->config = kime::kime_config_load();
  this->engine = kime::kime_engine_new(this->config);
}

KimePlatformInputContextPlugin::~KimePlatformInputContextPlugin() {
  kime::kime_engine_delete(this->engine);
  kime::kime_config_delete(this->config);
}

QPlatformInputContext *
KimePlatformInputContextPlugin::create(const QString &key,
                                       const QStringList &param_list) {
  return new KimeInputContext(this->engine, this->config);
}
