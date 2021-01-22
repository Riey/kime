#pragma once

#include <QDebug>

#ifdef DEBUG
#define KIME_DEBUG QTextStream(stderr, QIODevice::WriteOnly)
#endif

extern "C" {
#include <kime/kime_engine.h>
}
