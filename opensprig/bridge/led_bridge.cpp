#include "led.h"

extern "C" {
    LED *LED_new(LED_TYPE pin, bool pwm) {
        return new LED(pin, pwm);
    }

    void LED_free(LED *led) {
        delete led;
    }

    uint LED_get_brightness(LED *led) {
        return led->get_brightness();
    }

    void LED_set_brightness_bool(LED *led, bool level) {
        led->set_brightness(level);
    }

    void LED_set_brightness_uint(LED *led, uint level) {
        led->set_brightness(level);
    }

    void LED_set_absolute_brightness(LED *led, uint16_t level) {
        led->set_absolute_brightness(level);
    }

    void LED_blink(LED *led) {
        led->blink();
    }
}
