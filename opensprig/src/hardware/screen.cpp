#include "screen.h"
#include "st7735.h"

Screen::Screen() {
  spi_init(PORT, 30000000);

  gpio_set_function(RX, GPIO_FUNC_SPI);
  gpio_set_function(TX, GPIO_FUNC_SPI);
  gpio_set_function(CLK, GPIO_FUNC_SPI);

  gpio_set_function(CS, GPIO_FUNC_SIO);
  gpio_set_dir(CS, GPIO_OUT);
  gpio_put(CS, 1); // active-low

  gpio_set_function(DC, GPIO_FUNC_SIO);
  gpio_set_dir(DC, GPIO_OUT);
  gpio_put(DC, 0); // active-low

  gpio_set_function(RST, GPIO_FUNC_SIO);
  gpio_set_dir(RST, GPIO_OUT);
  gpio_put(RST, 0); // active-low

  init_screen_tft();
  draw_flood(0);

  gpio_set_function(BACKLIGHT, GPIO_FUNC_SIO);
  gpio_set_dir(BACKLIGHT, GPIO_OUT);
  gpio_put(BACKLIGHT, 0);
}

Screen::~Screen() {
  spi_deinit(PORT);

  gpio_set_function(RX, GPIO_FUNC_NULL);
  gpio_set_function(TX, GPIO_FUNC_NULL);
  gpio_set_function(CLK, GPIO_FUNC_NULL);
  gpio_set_function(CS, GPIO_FUNC_NULL);
  gpio_set_function(DC, GPIO_FUNC_NULL);
  gpio_set_function(RST, GPIO_FUNC_NULL);
  gpio_set_function(BACKLIGHT, GPIO_FUNC_NULL);

  gpio_set_pulls(RX, false, false);
  gpio_set_pulls(TX, false, false);
  gpio_set_pulls(CLK, false, false);
  gpio_set_pulls(CS, false, false);
  gpio_set_pulls(DC, false, false);
  gpio_set_pulls(RST, false, false);
  gpio_set_pulls(BACKLIGHT, false, false);
}

void Screen::blit() {
  fill_start();
  spi_write_blocking(PORT, (uint8_t *)this->screen_buf,
                     sizeof(uint16_t) * FRAME_SIZE);
  fill_finish();
}

void Screen::set_backlight(bool enabled) { gpio_put(BACKLIGHT, enabled); }

void Screen::set_pixel(uint16_t colour, int x, int y) {
  this->screen_buf[y + (FRAME_HEIGHT * x)] = colour;
}

void Screen::draw_flood(uint16_t colour) {
  std::fill(&this->screen_buf[0], &this->screen_buf[0] + FRAME_SIZE, colour);
}

void Screen::draw_rectangle(uint16_t colour, int x, int y, int w, int h) {
  for (int dx = x; dx < x + w; dx++)
    for (int dy = y; dy < y + h; dy++)
      set_pixel(colour, dx, dy);
}

void Screen::draw_character(uint16_t colour, char character, int x, int y) {
  int index = [character]() -> int {
    size_t i = 0;
    while (chars[i] != '\0') {
      if (chars[i] == character)
        return i;
      ++i;
    }

    return -1;
  }();

  if (index == -1)
    return;

  auto c = font[index];
  for (int row = 0; row < 6; row++) {
    auto row_data = c[row];
    for (int i = 0; i < 8; i++)
      if ((row_data >> i) & 1 == 1) {
        int x0 = x + (4 - i);
        int y0 = y + row;

        if (x0 < 0 || x0 > FRAME_WIDTH || y0 < 0 || y0 > FRAME_HEIGHT)
          continue;

        set_pixel(colour, x0, y0);
      }
  }
}

void Screen::draw_text(uint16_t colour, std::string contents, int x, int y) {
  int i = 0;
  for (char c : contents) {
    draw_character(colour, c, x + i, y);
    i += 4;
  }
}

void Screen::draw_callback(std::function<uint16_t(int, int)> callback, int x,
                           int y, int w, int h) {
  for (int dx = x; dx < x + w; dx++)
    for (int dy = y; dy < y + h; dy++)
      set_pixel(callback(dx, dy), dx, dy);
}
