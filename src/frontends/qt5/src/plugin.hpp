#include "kime-qt5.hpp"

#include <QString>
#include <QtPlugin>
#include <qpa/qplatforminputcontextplugin_p.h>

class KimePlatformInputContextPlugin : public QPlatformInputContextPlugin {
  Q_OBJECT
  Q_PLUGIN_METADATA(IID QPlatformInputContextFactoryInterface_iid FILE
                    "kime.json")

private:
  KimeInputEngine *engine = nullptr;
  KimeConfig *config = nullptr;

public:
  KimePlatformInputContextPlugin();
  ~KimePlatformInputContextPlugin();

  QPlatformInputContext *create(const QString &key,
                                const QStringList &param_list) override;
};
