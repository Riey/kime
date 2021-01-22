#include <QtPlugin>
#include <QString>
#include <QInputMethodEvent>
#include <qpa/qplatforminputcontextplugin_p.h>

extern "C" {
    #include <kime/kime_engine.h>
}

class KimePlatformInputContextPlugin: public QPlatformInputContextPlugin {
    Q_OBJECT
    Q_PLUGIN_METADATA(IID QPlatformInputContextFactoryInterface_iid FILE "kime.json")

private:
    InputEngine *engine = nullptr;
    Config *config = nullptr;
public:
    KimePlatformInputContextPlugin();
    ~KimePlatformInputContextPlugin();
    
    QPlatformInputContext *create(const QString &key, const QStringList &param_list) Q_DECL_OVERRIDE;
};
