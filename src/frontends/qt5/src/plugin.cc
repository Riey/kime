#include "plugin.hpp"
#include "input_context.hpp"

#include <QDebug>

KimePlatformInputContextPlugin::KimePlatformInputContextPlugin()
    : engine(kime_engine_new()), config(kime_config_load()) {}

KimePlatformInputContextPlugin::~KimePlatformInputContextPlugin() {
  kime_engine_delete(this->engine);
  kime_config_delete(this->config);
}

QPlatformInputContext *
KimePlatformInputContextPlugin::create(const QString &key,
                                       const QStringList &param_list) {
  return new KimeInputContext(this->engine, this->config);
}
