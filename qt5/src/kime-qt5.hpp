#pragma once

#include <QDebug>
#define KIME_DEBUG QTextStream(stderr, QIODevice::WriteOnly)

extern "C" {
    #include <kime/kime_engine.h>
}
