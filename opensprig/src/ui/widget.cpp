#include "ui/widget.h"

Widget::Widget(Screen *screen, uint x, uint y, uint w, uint h)
    : screen{screen}, x{x}, y{y}, w{w}, h{h} {}

Widget::~Widget() {}

void Widget::blit() {
  // screen->blit();
}
