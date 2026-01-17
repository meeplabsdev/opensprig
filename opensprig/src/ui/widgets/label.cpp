#include "ui/widgets/label.h"

LabelWidget::LabelWidget(Screen *screen, std::string text)
    : Widget(screen), text{text} {}

LabelWidget::~LabelWidget() {}

void LabelWidget::blit() {
  int dx = 0;
  int dy = 0;

  for (char c : text) {
    if (dx >= w) {
      dx = 0;
      dy += 6;
    }

    if (dy >= h)
      return;

    if (c == '\t') {
      dx += 16;
    } else if (c == '\n') {
      dx = w;
    } else {
      screen->draw_character(RGB(255, 255, 255), c, x + dx, y + dy);
      dx += 4;
    }
  }
}
