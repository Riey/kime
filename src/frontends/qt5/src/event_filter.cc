#include "event_filter.hpp"
#include <algorithm>

void KimeEventFilter::addCtx(KimeInputContext *ctx) { this->ctxs.push_back(ctx); }
void KimeEventFilter::removeCtx(KimeInputContext *ctx) {
  auto pos = std::find(this->ctxs.begin(), this->ctxs.end(), ctx);

  if (pos != this->ctxs.end()) {
    this->ctxs.erase(pos);
  }
}

bool KimeEventFilter::eventFilter(QObject *obj, QEvent *event) {
  // QMetaEnum meta = QMetaEnum::fromType<decltype(event->type())>();
  // KIME_DEBUG << meta.valueToKey(event->type()) << "\n";
  if (event->type() == QEvent::MouseButtonPress) {
#ifdef DEBUG
    KIME_DEBUG << "Button"
               << "\n";
#endif
    for (auto ctx: this->ctxs) {
      ctx->reset();
    }
  }

  return QObject::eventFilter(obj, event);
}
