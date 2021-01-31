#pragma once

#ifdef DEBUG
#include <QDebug>
#define KIME_DEBUG QTextStream(stderr, QIODevice::WriteOnly)
#endif

extern "C" {
#include <kime_engine.h>
}
