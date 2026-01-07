#pragma once

#include "kime-qt5.hpp"

#include <QString>
#include <QtPlugin>
#include <qpa/qplatforminputcontextplugin_p.h>

#ifndef KIME_QT_IID
#define KIME_QT_IID "org.qt-project.Qt.QPlatformInputContextFactoryInterface"
#endif

class KimePlatformInputContextPlugin : public QPlatformInputContextPlugin {
  Q_OBJECT
  Q_PLUGIN_METADATA(IID KIME_QT_IID FILE "kime.json")

private:
  kime::InputEngine *engine = nullptr;
  kime::Config *config = nullptr;

public:
  KimePlatformInputContextPlugin();
  ~KimePlatformInputContextPlugin();

  QPlatformInputContext *create(const QString &key,
                                const QStringList &param_list) override;
};