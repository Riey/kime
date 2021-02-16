#include "plugin.hpp"
#include "input_context.hpp"

#include <QDebug>

KimePlatformInputContextPlugin::KimePlatformInputContextPlugin()
    : engine(kime::kime_engine_new()), config(kime::kime_config_load()) {
      if (kime::kime_api_version() != 1) {
        QTextStream(stderr, QIODevice::WriteOnly) << "Kime Engine version is mismatched!\n";
        return;
      }

      kime::kime_enable_logger_with_env();
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
