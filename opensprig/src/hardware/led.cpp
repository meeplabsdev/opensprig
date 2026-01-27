#include "led.h"

#include <hardware/gpio.h>
#include <hardware/pwm.h>
#include <hardware/timer.h>
#include <pico/cyw43_arch.h>

static struct repeating_timer _animate_task;
void LED::animate() {
    uint32_t now = time_us_32();
    uint32_t delta = (now - this->animated_at) / 50000;

    if (this->cur_brightness < this->brightness)
        set_absolute_brightness(this->cur_brightness +
                                delta * MAX_BRIGHTNESS * 0.1);
    else if (this->cur_brightness > this->brightness)
        set_absolute_brightness(this->cur_brightness -
                                delta * MAX_BRIGHTNESS * 0.1);

    this->animated_at = now;
}

LED::LED(LED_TYPE pin, bool pwm) {
    this->pin = pin;
    this->pwm = pwm;
    this->brightness = 0;
    this->cur_brightness = 0;
    this->animated_at = time_us_32();

    if (this->pin == PICO)
        return;

    if (this->pwm) {
        uint slice_num = pwm_gpio_to_slice_num(this->pin);

        gpio_set_function(this->pin, GPIO_FUNC_PWM);
        pwm_set_enabled(slice_num, true);

        add_repeating_timer_ms(
            50,
            [](repeating_timer *rt) -> bool {
                ((LED *)rt->user_data)->animate();
                return true;
            },
            this, &_animate_task);
    } else {
        gpio_set_function(this->pin, GPIO_FUNC_SIO);
        gpio_set_dir(this->pin, GPIO_OUT);
    }
}

LED::~LED() {
    if (this->pin == PICO)
        return;

    if (this->pwm) {
        uint slice_num = pwm_gpio_to_slice_num(this->pin);

        cancel_repeating_timer(&_animate_task);

        pwm_set_enabled(slice_num, false);
        gpio_set_function(this->pin, GPIO_FUNC_NULL);
    } else {
        gpio_set_function(this->pin, GPIO_FUNC_NULL);
        gpio_set_pulls(this->pin, false, false);
    }
}

uint LED::get_brightness() {
    if (!this->pwm) {
    }

    return this->cur_brightness * 8 / 65535;
}

void LED::set_brightness(bool level) {
    if (this->pin == PICO)
        return cyw43_arch_gpio_put(CYW43_WL_GPIO_LED_PIN, level);

    if (!this->pwm)
        return gpio_put(this->pin, level);

    this->brightness = level ? MAX_BRIGHTNESS : 0;
}

void LED::set_brightness(uint level) {
    if (!this->pwm)
        return set_brightness(level > MAX_LEVEL / 2);

    if (level > MAX_LEVEL)
        level = MAX_LEVEL;
    else if (level < 0)
        level = 0;

    this->brightness = level * MAX_BRIGHTNESS / MAX_LEVEL;
}

void LED::set_absolute_brightness(uint16_t level) {
    if (!this->pwm)
        return set_brightness(level > MAX_BRIGHTNESS / 2);

    if (level > MAX_BRIGHTNESS)
        level = MAX_BRIGHTNESS;
    else if (level < 0)
        level = 0;

    uint slice_num = pwm_gpio_to_slice_num(this->pin);
    uint channel = pwm_gpio_to_channel(this->pin);

    pwm_set_chan_level(slice_num, channel, level);
    this->cur_brightness = level;
}

void LED::blink() {
    set_brightness(true);
    add_alarm_in_ms(
        800,
        [](alarm_id_t id, void *user_data) -> long long {
            ((LED *)user_data)->set_brightness(false);
            return 0;
        },
        this, true);
}
