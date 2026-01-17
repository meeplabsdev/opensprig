#ifndef OPENSPRIG_UI_WIDGET_H
#define OPENSPRIG_UI_WIDGET_H

#include <pico/types.h>
#include <screen.h>

enum Align {
  NONE = 0b0000,
  LEFT = 0b1000,
  RIGHT = 0b0100,
  TOP = 0b0010,
  BOTTOM = 0b0001,
  X_CENTER = 0b1100,
  Y_CENTER = 0b0011,
  XY_CENTER = 0b1111,
};

class Widget {
protected:
  Screen *screen;
  uint x;
  uint y;
  uint w;
  uint h;

  Widget(Screen *screen, uint x = 0, uint y = 0, uint w = FRAME_WIDTH,
         uint h = FRAME_HEIGHT);
  ~Widget();

public:
  virtual void blit();
};

#endif // OPENSPRIG_UI_WIDGET_H
