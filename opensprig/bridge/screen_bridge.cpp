#include "screen.h"

extern "C" {
    Screen *Screen_new() {
        return new Screen();
    }

    void Screen_free(Screen *screen) {
        delete screen;
    }

    uint16_t *Screen_screen_buf(Screen *screen) {
        return screen->screen_buf;
    }

    void Screen_blit(Screen *screen) {
        screen->blit();
    }

    void Screen_set_backlight(Screen *screen, bool enabled) {
        screen->set_backlight(enabled);
    }

    void Screen_set_pixel(Screen *screen, uint16_t colour, int x, int y) {
        screen->set_pixel(colour, x, y);
    }

    void Screen_draw_flood(Screen *screen, uint16_t colour) {
        screen->draw_flood(colour);
    }

    void Screen_draw_rectangle(Screen *screen, uint16_t colour, int x, int y,
                               int w, int h) {
        screen->draw_rectangle(colour, x, y, w, h);
    }

    void Screen_draw_character(Screen *screen, uint16_t colour, char character,
                               int x, int y) {
        screen->draw_character(colour, character, x, y);
    }
}
