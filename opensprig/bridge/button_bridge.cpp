#include "button.h"

extern "C" {
    Button *Button_new(BUTTON_TYPE pin) {
        return new Button(pin);
    }

    void Button_free(Button *button) {
        delete button;
    }

    bool Button_is_pressed(Button *button) {
        return button->is_pressed();
    }

    bool Button_is_long_pressed(Button *button) {
        return button->is_long_pressed();
    }

    bool Button_was_pressed(Button *button) {
        return button->was_pressed();
    }

    bool Button_was_long_pressed(Button *button) {
        return button->was_long_pressed();
    }
}
