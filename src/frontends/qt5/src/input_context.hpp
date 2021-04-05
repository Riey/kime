#pragma once

#include "kime-qt5.hpp"

#include <QtGui/QInputMethodEvent>
#include <qpa/qplatforminputcontext.h>

class KimeInputContext;

class KimeEventFilter : public QObject {
  Q_OBJECT
public:
  bool eventFilter(QObject *obj, QEvent *event);
  void setCtx(KimeInputContext *ctx);

private:
  KimeInputContext *ctx = nullptr;
};

class KimeInputContext : public QPlatformInputContext {
  Q_OBJECT

public:
  KimeInputContext(kime::InputEngine *engine, const kime::Config *config);
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
  void commit_str(kime::RustStr s);
  void preedit_str(kime::RustStr s);

  bool visible = false;
  QList<QInputMethodEvent::Attribute> attributes;
  const kime::Config *config = nullptr;
  kime::InputEngine *engine = nullptr;
  QObject *focus_object = nullptr;
  KimeEventFilter filter;
};
