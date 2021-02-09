#pragma once

#ifdef DEBUG
#include <QDebug>
#define KIME_DEBUG QTextStream(stderr, QIODevice::WriteOnly)
#endif

#include <kime_engine.hpp>
