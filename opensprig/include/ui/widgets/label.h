#ifndef OPENSPRIG_UI_WIDGETS_TEXT_H
#define OPENSPRIG_UI_WIDGETS_TEXT_H

#include <screen.h>
#include <ui/widget.h>

class LabelWidget : Widget {
  std::string text;

public:
  LabelWidget(Screen *screen, std::string text);
  ~LabelWidget();

  void blit() override;
};

#endif // OPENSPRIG_UI_WIDGETS_TEXT_H
