#include "kime-qt5.hpp"

#include <QtGui/QInputMethodEvent>
#include <qpa/qplatforminputcontext.h>

class KimeInputContext : public QPlatformInputContext {
  Q_OBJECT

public:
  KimeInputContext(KimeInputEngine *engine, const KimeConfig *config);
  ~KimeInputContext();

  bool isValid() const override;
  Qt::LayoutDirection inputDirection() const override;

  void reset() override;
  void commit() override;
  void update(Qt::InputMethodQueries queries) override;
  void invokeAction(QInputMethod::Action action, int cursorPosition) override;
  bool filterEvent(const QEvent *event) override;
  void setFocusObject(QObject *object) override;

private:
  void commit_ch(char32_t ch);
  void preedit_ch(char32_t ch);

  QList<QInputMethodEvent::Attribute> attributes;
  KimeInputEngine *engine = nullptr;
  const KimeConfig *config = nullptr;
  QObject *focus_object = nullptr;
};
