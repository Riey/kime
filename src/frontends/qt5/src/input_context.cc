#include "input_context.hpp"

#include <QtCore/QCoreApplication>
#include <QtGui/QKeyEvent>

KimeInputContext::KimeInputContext(kime::InputEngine *engine,
                                   const kime::Config *config)
    : engine(engine), config(config) {}

KimeInputContext::~KimeInputContext() {}

void KimeInputContext::update(Qt::InputMethodQueries queries) {}

void KimeInputContext::commit() {}

void KimeInputContext::reset() {
#ifdef DEBUG
  KIME_DEBUG << "reset"
             << "\n";
#endif
  auto ch = static_cast<char32_t>(kime_engine_reset(this->engine));

  if (ch) {
    commit_ch(ch);
  }
}

void KimeInputContext::setFocusObject(QObject *object) {
  if (object) {
    // set focus
    kime_engine_update_hangul_state(this->engine);
    this->focus_object = object;
  } else {
    // unset focus
    this->focus_object = object;
    this->reset();
  }
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

  if (ret.hangul_changed) {
    kime::kime_engine_update_hangul_state(this->engine);
  }

#ifdef DEBUG
  KIME_DEBUG << "ty: " << (uint32_t) ret.ty << "char1: " << (QChar)ret.char1
             << "char2: " << (QChar)ret.char2 << "\n";
#endif

  switch (ret.ty) {
  case kime::InputResultType::Bypass:
    return false;
  case kime::InputResultType::Consume:
    return true;
  case kime::InputResultType::ClearPreedit:
    commit_ch(U'\0');
    return true;
  case kime::InputResultType::Commit:
    commit_ch(ret.char1);
    return true;
  case kime::InputResultType::CommitPreedit:
    commit_ch(ret.char1);
    preedit_ch(ret.char2);
    return true;
  case kime::InputResultType::Preedit:
    preedit_ch(ret.char1);
    return true;
  case kime::InputResultType::CommitCommit:
    commit_ch(ret.char1);
    commit_ch(ret.char2);
    return true;
  case kime::InputResultType::CommitBypass:
    commit_ch(ret.char1);
    return false;

  default:
    return false;
  }
}

void KimeInputContext::preedit_ch(char32_t ch) {
  assert(ch != U'\0');
  QInputMethodEvent e(QString::fromUcs4(&ch, 1), this->attributes);
  QCoreApplication::sendEvent(this->focus_object, &e);
}

void KimeInputContext::commit_ch(char32_t ch) {
  QInputMethodEvent e;
  if (ch) {
    e.setCommitString(QString::fromUcs4(&ch, 1));
  }
  QCoreApplication::sendEvent(this->focus_object, &e);
}
