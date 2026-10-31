// Qt5/Qt6 QLineEdit probe for the kime e2e harness (compiled twice by cc.rs,
// once against Qt5Widgets and once against Qt6Widgets).
//
// Usage: qt_probe <outfile>
//   <outfile>          line-edit text, rewritten every 150ms
//   <outfile>.preedit  appended "p:<preedit>" / "c:<commit>" per QInputMethodEvent
//   <outfile>.commits  appended "d<depth>:<commit>" per non-empty commitString,
//                      only with KIME_PROBE_RESET_IN_COMMIT=1 (see below)
// Prints READY on stderr once the window is shown.
//
// The event filter makes preedit observable (QLineEdit::text() only shows
// committed text), which the candidate-window (#757) and plugin-loading
// (#736/#756) tests rely on.
//
// With env KIME_PROBE_RESET_IN_COMMIT=1 the line edit's inputMethodEvent
// override calls QGuiApplication::inputMethod()->reset() upon a commit — the
// #562-class pattern (a client resets the IM context from its commit path),
// the qt twin of gtk_reset_probe.py --reset-in-commit. Without the env var
// the override is pass-through and the probe behaves as before.
#include <QApplication>
#include <QFile>
#include <QInputMethod>
#include <QInputMethodEvent>
#include <QLineEdit>
#include <QTimer>
#include <cstdio>

class ImLogger : public QObject {
public:
    explicit ImLogger(const QString &path) : m_path(path) {}

protected:
    bool eventFilter(QObject *obj, QEvent *ev) override {
        if (ev->type() == QEvent::InputMethod) {
            auto *ime = static_cast<QInputMethodEvent *>(ev);
            QFile f(m_path);
            if (f.open(QIODevice::Append)) {
                f.write("p:" + ime->preeditString().toUtf8() + "\n");
                if (!ime->commitString().isEmpty()) {
                    f.write("c:" + ime->commitString().toUtf8() + "\n");
                }
            }
        }
        return QObject::eventFilter(obj, ev);
    }

private:
    QString m_path;
};

// Reset-in-commit instrument (#562-class, Q-06/Q-07): with
// KIME_PROBE_RESET_IN_COMMIT=1 every non-empty commitString is appended to
// <outfile>.commits as "d<depth>:<text>" (depth = inputMethodEvent nesting;
// d2+ = the commit was re-delivered from inside the outer delivery), and at
// depth 1 only the handler calls QGuiApplication::inputMethod()->reset()
// BEFORE letting QLineEdit process the event. kime's input context emits
// commits with the synchronous QCoreApplication::sendEvent, so the reset
// re-enters KimeInputContext::reset() while the outer emission is on the
// stack. Depth-1-only mirrors gtk_reset_probe.py: pre-fix the engine commit
// buffer is cleared only after the emission, so an always-resetting client
// would recurse without bound; capping at depth 1 shows the bug as exactly
// one duplicate line. Without the env var this class behaves exactly like
// QLineEdit.
class ProbeLineEdit : public QLineEdit {
public:
    ProbeLineEdit(const QString &commitsPath, bool resetInCommit)
        : m_commitsPath(commitsPath), m_resetInCommit(resetInCommit) {}

protected:
    void inputMethodEvent(QInputMethodEvent *ev) override {
        if (!m_resetInCommit || ev->commitString().isEmpty()) {
            QLineEdit::inputMethodEvent(ev);
            return;
        }
        ++m_depth;
        {
            // Scoped so the append flushes BEFORE the nested reset() below —
            // an open QFile buffers, and the nested depth-2 line would land
            // in the log first.
            QFile f(m_commitsPath);
            if (f.open(QIODevice::Append)) {
                f.write("d" + QByteArray::number(m_depth) + ":" +
                        ev->commitString().toUtf8() + "\n");
            }
        }
        if (m_depth == 1) {
            QGuiApplication::inputMethod()->reset();
        }
        QLineEdit::inputMethodEvent(ev);
        --m_depth;
    }

private:
    QString m_commitsPath;
    bool m_resetInCommit;
    int m_depth = 0;
};

int main(int argc, char **argv) {
    QApplication a(argc, argv);
    if (argc < 2) {
        fprintf(stderr, "usage: qt_probe <outfile>\n");
        return 1;
    }
    QString out = QString::fromLocal8Bit(argv[1]);
    ProbeLineEdit e(out + ".commits",
                    qgetenv("KIME_PROBE_RESET_IN_COMMIT") == "1");
    e.setWindowTitle("kimeprobe");
    e.resize(320, 60);
    ImLogger logger(out + ".preedit");
    e.installEventFilter(&logger);
    e.show();
    QTimer t;
    QObject::connect(&t, &QTimer::timeout, [&] {
        QFile f(out);
        if (f.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
            f.write(e.text().toUtf8());
        }
    });
    t.start(150);
    fprintf(stderr, "READY\n");
    fflush(stderr);
    return a.exec();
}
