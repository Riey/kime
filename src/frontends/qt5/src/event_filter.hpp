#pragma once

#include "input_context.hpp"
#include <vector>

class KimeEventFilter : public QObject {
  Q_OBJECT
public:
  bool eventFilter(QObject *obj, QEvent *event);
  void addCtx(KimeInputContext *ctx);
  void removeCtx(KimeInputContext *ctx);

private:
  std::vector<KimeInputContext*> ctxs;
};
