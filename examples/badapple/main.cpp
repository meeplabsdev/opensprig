#include <pico/cyw43_arch.h>
#include <screen.h>
#include <speaker.h>
#include <storage.h>

#define FOREVER while (1)

static Screen screen;
static Speaker speaker;
static Storage storage;

struct repeating_timer _screen_task;
bool screen_task(repeating_timer_t *rt) {
  screen.blit();
  return true;
}

static FIL file;
struct repeating_timer _main_task;
bool main_task(repeating_timer_t *rt) {
  // read straigt into the screen buffer
  FRESULT fr = storage.raw_read(&file, screen.screen_buf, FRAME_SIZE * 2);

  if (fr != FR_OK) {
    f_close(&file);
    return false;
  }

  return true;
}

int main() {
  screen = Screen();
  speaker = Speaker();
  storage = Storage();

  add_repeating_timer_ms(-82, screen_task, NULL, &_screen_task);

  storage.read("logo.bin", screen.screen_buf, FRAME_SIZE * 2, 0);
  screen.set_backlight(true);
  speaker.sine(0x800000, 16);

  sleep_ms(800);

  FIL badapple_file;
  f_open(&badapple_file, "badapple.bin", FA_READ);
  add_repeating_timer_ms(-50, main_task, NULL, &_main_task);

  FOREVER tight_loop_contents();
}
