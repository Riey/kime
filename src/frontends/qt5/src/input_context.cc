#include "input_context.hpp"

#include <QMetaEnum>
#include <QtGui/QGuiApplication>
#include <QtGui/QKeyEvent>
#include <QtGui/QTextCharFormat>

KimeInputContext::KimeInputContext(kime::InputEngine *engine,
                                   const kime::Config *config) {
  this->engine = engine;
  this->config = config;
}

void KimeInputContext::update(Qt::InputMethodQueries queries) {}

void KimeInputContext::commit() {
  // On focus-out Qt calls commit() *before* setFocusObject(nullptr), so the
  // engine_ready guard there never gets a chance: an unconditional reset()
  // here kills the just-spawned candidate window and discards the syllable it
  // is converting (issue #779, residual of #757).
  if (!this->engine_ready) {
    return;
  }

  this->reset();
}

void KimeInputContext::reset() {
#ifdef DEBUG
  KIME_DEBUG << "reset"
             << "\n";
#endif
  kime::kime_engine_clear_preedit(this->engine);
  // Snapshot the pending commit text and reset the engine (which clears its
  // commit buffer) BEFORE emitting: commit_str delivers through the
  // synchronous QCoreApplication::sendEvent, and a client may call
  // QInputMethod::reset() from inside its inputMethodEvent() (the #562-class
  // pattern, the qt twin of the gtk immodule bug). With emit-then-reset the
  // re-entrant reset would re-read the still-populated engine buffer and
  // commit the same string again. With the engine already reset, a
  // re-entrant reset only commits the live preedit, which is the correct
  // reset semantic. commit_str skips empty snapshots, so a bare reset emits
  // nothing.
  kime::RustStr s = kime::kime_engine_commit_str(this->engine);
  QString text = QString::fromUtf8((const char *)(s.ptr), s.len);
  kime::kime_engine_reset(this->engine);
  this->commit_str(text);
}

void KimeInputContext::setFocusObject(QObject *object) {
  if (object) {
    kime::kime_engine_update_layout_state(this->engine);
    if (!this->engine_ready) {
      if (kime::kime_engine_check_ready(this->engine)) {
        kime::InputResult ret = kime::kime_engine_end_ready(this->engine);
        this->process_input_result(ret);
        this->engine_ready = true;
      }
    }
  } else if (this->focus_object && this->engine_ready) {
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

  bool numlock = modifiers.testFlag(Qt::KeyboardModifier::KeypadModifier);

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
      this->engine, this->config, (uint16_t)keyevent->nativeScanCode(), numlock, state);

  return this->process_input_result(ret);
}

void KimeInputContext::preedit_str(kime::RustStr s) {
  this->focus_object = qApp->focusObject();
  if (!this->focus_object) {
    return;
  }

  QTextCharFormat fmt;
  fmt.setFontUnderline(true);
  QString qs = QString::fromUtf8((const char *)(s.ptr), s.len);
  this->attributes.push_back(QInputMethodEvent::Attribute{
      QInputMethodEvent::AttributeType::TextFormat,
      0, static_cast<int>(qs.length()), fmt
  });
  QInputMethodEvent e(qs, this->attributes);
  this->attributes.clear();
  QCoreApplication::sendEvent(this->focus_object, &e);
}

void KimeInputContext::commit_str(const QString &s) {
  this->focus_object = qApp->focusObject();
  if (!this->focus_object) {
    return;
  }
  // Nothing to commit: don't send an empty QInputMethodEvent (callers
  // snapshot the engine buffer, which is empty on a bare reset).
  if (s.isEmpty()) {
    return;
  }

  QInputMethodEvent e;
  e.setCommitString(s);
  QCoreApplication::sendEvent(this->focus_object, &e);
}

bool KimeInputContext::process_input_result(kime::InputResult ret) {
  if (ret & kime::InputResult_NOT_READY) {
    // The engine is waiting for the candidate window process (e.g. hanja
    // conversion). Mark it not ready so losing focus to that window does
    // not reset the engine and kill the popup (issue #757).
    this->engine_ready = false;
  }

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
    // Snapshot the commit string and clear the engine buffer BEFORE emitting:
    // sendEvent is synchronous, so a widget calling QInputMethod::reset()
    // from its inputMethodEvent() (the #562-class pattern) re-enters reset()
    // while this emission is still on the stack. With emit-then-clear that
    // re-entrant reset would read the still-populated engine buffer and
    // deliver the same string twice.
    kime::RustStr s = kime::kime_engine_commit_str(this->engine);
    QString text = QString::fromUtf8((const char *)(s.ptr), s.len);
    kime::kime_engine_clear_commit(this->engine);
    commit_str(text);
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
