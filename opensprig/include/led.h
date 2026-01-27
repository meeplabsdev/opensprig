#ifndef OPENSPRIG_LED_H
#define OPENSPRIG_LED_H

#include <pico/types.h>

#define MAX_LEVEL 100u
#define MAX_BRIGHTNESS 8192u

enum LED_TYPE {
    PICO,
    STATUS = 28,
    NETWORK = 4,
};

class LED {
    int pin;
    bool pwm;
    uint16_t brightness;
    uint16_t cur_brightness;
    uint32_t animated_at;

    void animate();

  public:
    LED(LED_TYPE pin, bool pwm);
    ~LED();

    uint get_brightness();
    void set_brightness(bool level);
    void set_brightness(uint level);
    void set_absolute_brightness(uint16_t level);
    void blink();
};

#endif // OPENSPRIG_LED_H
