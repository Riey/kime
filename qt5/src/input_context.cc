#include "input_context.hpp"

#include <QtCore/QCoreApplication>
#include <QtGui/QKeyEvent>

KimeInputContext::KimeInputContext(InputEngine *engine, const Config *config) : engine(engine), config(config)
{
}

KimeInputContext::~KimeInputContext() {}

void KimeInputContext::update(Qt::InputMethodQueries queries)
{
    KIME_DEBUG << "update: " << queries << "\n";
}

void KimeInputContext::commit()
{
    KIME_DEBUG << "commit"
               << "\n";
}

void KimeInputContext::reset()
{
    KIME_DEBUG << "reset"
               << "\n";

    auto ch = static_cast<char32_t>(kime_engine_reset(this->engine));

    if (ch)
    {
        commit_ch(ch);
    }
}

void KimeInputContext::setFocusObject(QObject *object)
{
    if (object)
    {
        // set focus
        this->focus_object = object;
    }
    else
    {
        // unset focus
        this->focus_object = object;
        this->reset();
    }
}

bool KimeInputContext::isValid() const
{
    return true;
}

Qt::LayoutDirection KimeInputContext::inputDirection() const
{
    return Qt::LayoutDirection::LeftToRight;
}

void KimeInputContext::invokeAction(QInputMethod::Action action, int cursorPosition)
{
    KIME_DEBUG << "invokeAction: " << action << ", " << cursorPosition << "\n";
}

bool KimeInputContext::filterEvent(const QEvent *event)
{
    if (event->type() != QEvent::KeyPress)
    {
        return false;
    }

    auto keyevent = static_cast<const QKeyEvent *>(event);
    auto modifiers = keyevent->modifiers();

    ModifierState state = 0;

    if (modifiers.testFlag(Qt::KeyboardModifier::ControlModifier))
    {
        state |= ModifierState_CONTROL;
    }

    if (modifiers.testFlag(Qt::KeyboardModifier::ShiftModifier))
    {
        state |= ModifierState_SHIFT;
    }

    if (modifiers.testFlag(Qt::KeyboardModifier::MetaModifier))
    {
        state |= ModifierState_SUPER;
    }

    InputResult ret = kime_engine_press_key(this->engine, this->config, (uint16_t)keyevent->nativeScanCode(), state);

    KIME_DEBUG << "ty: " << ret.ty << "char1: " << (QChar)ret.char1 << "char2: " << (QChar)ret.char2 << "\n";

    switch (ret.ty)
    {
    case InputResultType::Bypass:
        return false;
    case InputResultType::Consume:
        return true;
    case InputResultType::ClearPreedit:
        commit_ch(U'\0');
        return true;
    case InputResultType::Commit:
        commit_ch(ret.char1);
        return true;
    case InputResultType::CommitPreedit:
        commit_ch(ret.char1);
        preedit_ch(ret.char2);
        return true;
    case InputResultType::Preedit:
        preedit_ch(ret.char1);
        return true;
    case InputResultType::CommitCommit:
        commit_ch(ret.char1);
        commit_ch(ret.char2);
        return true;
    case InputResultType::CommitBypass:
        commit_ch(ret.char1);
        return false;

    default:
        return false;
    }
}

void KimeInputContext::preedit_ch(char32_t ch)
{
    QInputMethodEvent e(QString::fromUcs4(&ch, 1), this->attributes);
    QCoreApplication::sendEvent(this->focus_object, &e);
}

void KimeInputContext::commit_ch(char32_t ch)
{
    QInputMethodEvent e;
    e.setCommitString(QString::fromUcs4(&ch, 1));
    QCoreApplication::sendEvent(this->focus_object, &e);
}
