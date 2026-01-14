#ifndef OPENSPRIG_SCREEN_H
#define OPENSPRIG_SCREEN_H

#include <cstdint>
#include <font.h>
#include <functional>
#include <string>

#define BACKLIGHT 17

#define FRAME_WIDTH 160
#define FRAME_HEIGHT 128
#define FRAME_SIZE FRAME_WIDTH *FRAME_HEIGHT

class Screen {
public:
  uint16_t screen_buf[FRAME_SIZE];

  explicit Screen();
  ~Screen();
  void blit();
  void set_backlight(bool enabled);
  void set_pixel(uint16_t colour, int x, int y);
  void draw_flood(uint16_t colour);
  void draw_rectangle(uint16_t colour, int x, int y, int w, int h);
  void draw_character(uint16_t colour, char character, int x, int y);
  void draw_text(uint16_t colour, std::string contents, int x, int y);
  void draw_callback(std::function<uint16_t(int, int)> callback, int x, int y,
                     int w, int h);
};

static uint16_t RGB(uint8_t r, uint8_t g, uint8_t b) {
  r = (uint8_t)((float)((float)r / 255.0f) * 31.0f);
  b = (uint8_t)((float)((float)b / 255.0f) * 31.0f);
  g = (uint8_t)((float)((float)g / 255.0f) * 63.0f);

  return ((r & 0b11111000) << 8) | ((b & 0b11111100) << 3) | (g >> 3);
}

#endif // OPENSPRIG_SCREEN_H
