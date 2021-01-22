#include <QtCore/QtGlobal>
#include <QString>
#include <QInputMethodEvent>
#include <qpa/qplatforminputcontextplugin_p.h>

#include <kime/kime_engine.h>

class KimePlatformInputContextPlugin: public QPlatformInputContextPlugin {
private:
    InputEngine *engine = nullptr;
    Config *config = nullptr;
public:
    KimePlatformInputContextPlugin();
    ~KimePlatformInputContextPlugin();
    
    QPlatformInputContext *create(const QString &key, const QStringList &param_list) override;
};
