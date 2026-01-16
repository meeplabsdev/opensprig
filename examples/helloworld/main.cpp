#include <pico/cyw43_arch.h>
#include <screen.h>

#define FOREVER while (1)

int main() {
  Screen screen = Screen();

  screen.set_backlight(true);
  screen.draw_text(RGB(255, 255, 255), "Hello, World!", 6, 6);
  screen.blit();

  FOREVER tight_loop_contents();
}
