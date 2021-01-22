#include "kime-qt5.hpp"
#include <qpa/qplatforminputcontext.h>

class KimeInputContext: public QPlatformInputContext {
    Q_OBJECT
        
public:
    KimeInputContext(InputEngine *engine, Config *config);
    ~KimeInputContext();
    
private:
    InputEngine *engine;
    Config *config;
};
