#ifndef OPENSPRIG_BUTTON_H
#define OPENSPRIG_BUTTON_H

#include <pico/types.h>

#define HOLD_TIME_US 1000000

enum BUTTON_TYPE {
  BOOTSEL,
  L_UP = 5,
  L_DOWN = 7,
  L_LEFT = 6,
  L_RIGHT = 8,
  R_UP = 12,
  R_DOWN = 14,
  R_LEFT = 13,
  R_RIGHT = 15,
};

class Button {
  int pin;
  bool pressed;
  bool held;
  uint64_t held_started;

public:
  Button(BUTTON_TYPE pin);
  ~Button();

  bool is_pressed();
  bool is_long_pressed();
  bool was_pressed();
  bool was_long_pressed();
};

#endif // OPENSPRIG_BUTTON_H
