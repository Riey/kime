#include "input_context.hpp"

#include <QMetaEnum>
#include <QtCore/QCoreApplication>
#include <QtGui/QKeyEvent>
#include <QtGui/QTextCharFormat>
#include <QtWidgets/QApplication>

static bool APP_QUITED = false;

KimeInputContext::KimeInputContext(kime::InputEngine *engine,
                                   const kime::Config *config) {
  this->engine = engine;
  this->config = config;
  this->filter.setCtx(this);
  qApp->installEventFilter(&this->filter);
  QObject::connect(qApp, &QCoreApplication::aboutToQuit,
                   []() { APP_QUITED = true; });
}

KimeInputContext::~KimeInputContext() {
  if (!APP_QUITED) {
    qApp->removeEventFilter(&this->filter);
  } else {
#ifdef DEBUG
    KIME_DEBUG << "Remove skipped\n";
#endif
  }
}

void KimeInputContext::update(Qt::InputMethodQueries queries) {}

void KimeInputContext::commit() {}

void KimeInputContext::reset() {
#ifdef DEBUG
  KIME_DEBUG << "reset"
             << "\n";
#endif
  kime::kime_engine_clear_preedit(this->engine);
  this->commit_str(kime::kime_engine_commit_str(this->engine));
  kime::kime_engine_reset(this->engine);
}

void KimeInputContext::setFocusObject(QObject *object) {
  if (object) {
    kime::kime_engine_update_layout_state(this->engine);
  } else if (this->focus_object) {
    this->reset();
  }

  this->focus_object = object;
}

bool KimeInputContext::isValid() const { return true; }

Qt::LayoutDirection KimeInputContext::inputDirection() const {
  return Qt::LayoutDirection::LeftToRight;
}

void KimeInputContext::invokeAction(QInputMethod::Action action,
                                    int cursorPosition) {
#ifdef DEBUG
  KIME_DEBUG << "invokeAction: " << action << ", " << cursorPosition << "\n";
#endif
}

bool KimeInputContext::filterEvent(const QEvent *event) {
  if (event->type() != QEvent::KeyPress) {
    return false;
  }

  auto keyevent = static_cast<const QKeyEvent *>(event);
  auto modifiers = keyevent->modifiers();

  kime::ModifierState state = 0;

  if (modifiers.testFlag(Qt::KeyboardModifier::ControlModifier)) {
    state |= kime::ModifierState_CONTROL;
  }

  if (modifiers.testFlag(Qt::KeyboardModifier::ShiftModifier)) {
    state |= kime::ModifierState_SHIFT;
  }

  if (modifiers.testFlag(Qt::KeyboardModifier::AltModifier)) {
    state |= kime::ModifierState_ALT;
  }

  if (modifiers.testFlag(Qt::KeyboardModifier::MetaModifier)) {
    state |= kime::ModifierState_SUPER;
  }

  kime::InputResult ret = kime_engine_press_key(
      this->engine, this->config, (uint16_t)keyevent->nativeScanCode(), state);

  if (ret & kime::InputResult_LANGUAGE_CHANGED) {
    kime::kime_engine_update_layout_state(this->engine);
  }

  bool visible = !!(ret & kime::InputResult_HAS_PREEDIT);

  if (!visible) {
    // only send preedit when invisible
    // issue #425
    if (this->visible) {
#ifdef DEBUG
      KIME_DEBUG << "Clear preedit\n";
#endif
      this->preedit_str(kime::kime_engine_preedit_str(this->engine));
    }
  }

  if (ret & (kime::InputResult_HAS_COMMIT)) {
#ifdef DEBUG
    KIME_DEBUG << "Commit\n";
#endif
    commit_str(kime::kime_engine_commit_str(this->engine));
    kime::kime_engine_clear_commit(this->engine);
  }

  if (visible) {
#ifdef DEBUG
    KIME_DEBUG << "Update preedit\n";
#endif
    this->preedit_str(kime::kime_engine_preedit_str(this->engine));
  }

  this->visible = visible;

  return !!(ret & kime::InputResult_CONSUMED);
}

void KimeInputContext::preedit_str(kime::RustStr s) {
  if (!this->focus_object) {
    return;
  }

  QTextCharFormat fmt;
  fmt.setFontUnderline(true);
  QString qs = QString::fromUtf8((const char *)(s.ptr), s.len);
  this->attributes.push_back(QInputMethodEvent::Attribute{
      QInputMethodEvent::AttributeType::TextFormat, 0, qs.length(), fmt});
  QInputMethodEvent e(qs, this->attributes);
  this->attributes.clear();
  QCoreApplication::sendEvent(this->focus_object, &e);
}

void KimeInputContext::commit_str(kime::RustStr s) {
  if (!this->focus_object) {
    return;
  }

  QInputMethodEvent e;
  if (s.len) {
    e.setCommitString(QString::fromUtf8((const char *)(s.ptr), s.len));
  }
  QCoreApplication::sendEvent(this->focus_object, &e);
}

void KimeEventFilter::setCtx(KimeInputContext *ctx) { this->ctx = ctx; }

bool KimeEventFilter::eventFilter(QObject *obj, QEvent *event) {
  // QMetaEnum meta = QMetaEnum::fromType<decltype(event->type())>();
  // KIME_DEBUG << meta.valueToKey(event->type()) << "\n";
  if (event->type() == QEvent::MouseButtonPress) {
#ifdef DEBUG
    KIME_DEBUG << "Button"
               << "\n";
#endif
    this->ctx->reset();
  }

  return QObject::eventFilter(obj, event);
}
