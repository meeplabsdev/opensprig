#include "button.h"

// https://github.com/raspberrypi/pico-examples/blob/master/picoboard/button/button.c
bool __no_inline_not_in_flash_func(get_bootsel_button)() {
  bool state = false;
  uint32_t flags = save_and_disable_interrupts();

  hw_write_masked(&ioqspi_hw->io[1].ctrl,
                  GPIO_OVERRIDE_LOW << IO_QSPI_GPIO_QSPI_SS_CTRL_OEOVER_LSB,
                  IO_QSPI_GPIO_QSPI_SS_CTRL_OEOVER_BITS);

  for (volatile int i = 0; i < 1000; ++i)
    ;
  state = !(sio_hw->gpio_hi_in & (1u << 1));

  hw_write_masked(&ioqspi_hw->io[1].ctrl,
                  GPIO_OVERRIDE_NORMAL << IO_QSPI_GPIO_QSPI_SS_CTRL_OEOVER_LSB,
                  IO_QSPI_GPIO_QSPI_SS_CTRL_OEOVER_BITS);

  restore_interrupts(flags);
  return state;
}

Button::Button(BUTTON_TYPE pin) {
  this->pin = pin;
  this->pressed = false;
  this->held = false;

  if (this->pin == BOOTSEL)
    return;

  gpio_set_function(this->pin, GPIO_FUNC_SIO);
  gpio_set_dir(this->pin, GPIO_IN);
  gpio_pull_up(this->pin);
}

Button::~Button() {
  if (this->pin == BOOTSEL)
    return;

  gpio_set_function(this->pin, GPIO_FUNC_NULL);
  gpio_set_pulls(this->pin, false, false);
}

bool Button::is_pressed() {
  if (this->pin == BOOTSEL)
    return get_bootsel_button();

  return !gpio_get(this->pin);
}

bool Button::is_long_pressed() {
  const bool cur_pressed = is_pressed();

  if (cur_pressed && !this->held_started)
    this->held_started = time_us_32();
  else if (!cur_pressed && this->held_started)
    this->held_started = 0;

  if (!this->held_started)
    return false;

  return (time_us_32() - this->held_started) > HOLD_TIME_US;
}

bool Button::was_pressed() {
  const bool cur_pressed = is_pressed();

  if (cur_pressed && this->pressed)
    return false;

  this->pressed = cur_pressed;
  return cur_pressed;
}

bool Button::was_long_pressed() {
  const bool cur_long_pressed = is_long_pressed();

  if (cur_long_pressed && this->held)
    return false;

  this->held = cur_long_pressed;
  return cur_long_pressed;
}
