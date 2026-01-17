/* Credits to https://github.com/adwuard/Picocalc_SD_Boot, where *
 * a large amount of the application loading logic came from.    */

#include "led.h"
#include "screen.h"
#include "speaker.h"
#include "storage.h"
#include <hardware/clocks.h>
#include <hardware/flash.h>
#include <hardware/regs/m0plus.h>
#include <hardware/sync.h>
#include <hardware/timer.h>
#include <hardware/watchdog.h>
#include <pico/cyw43_arch.h>
#include <pico/stdio.h>

#define VTOR_OFFSET M0PLUS_VTOR_OFFSET
#define MAX_RAM 0x20040000
#define SD_BOOT_FLASH_OFFSET (256 * 1024)
#define MAX_APP_SIZE (PICO_FLASH_SIZE_BYTES - SD_BOOT_FLASH_OFFSET)

#define FOREVER while (1)

static Screen screen;
static Storage storage;

int line = 1;
void log(uint16_t colour, std::string contents) {
  screen.draw_text(colour, contents, 6, line * 6);
  screen.blit();
  line++;
  if (line > 20) {
    screen.draw_flood(RGB(0, 0, 0));
    line = 1;
  }
}

void log(uint16_t colour, const char *fmt, ...) {
  char buffer[37];
  va_list args;
  va_start(args, fmt);
  vsnprintf(buffer, sizeof(buffer), fmt, args);
  va_end(args);
  log(colour, std::string(buffer));
}

void log(const char *fmt, ...) {
  char buffer[37];
  va_list args;
  va_start(args, fmt);
  vsnprintf(buffer, sizeof(buffer), fmt, args);
  va_end(args);
  log(RGB(255, 255, 255), std::string(buffer));
}

void err(const char *fmt, ...) {
  char buffer[37];
  va_list args;
  va_start(args, fmt);
  vsnprintf(buffer, sizeof(buffer), fmt, args);
  va_end(args);
  log(RGB(255, 100, 100), std::string(buffer));
}

static bool __not_in_flash_func(load_program)(const char *filename) {
  FIL fp;
  FRESULT fr;

  fr = f_open(&fp, filename, FA_READ);
  if (fr != FR_OK) {
    err("Failed to open %s: %s", filename, strerror(errno));
    return false;
  }

  FSIZE_t file_size = f_size(&fp);
  log("File size: %lld", file_size);

  if (file_size <= 0) {
    err("Invalid size: %lld", file_size);
    f_close(&fp);
    return false;
  }

  if (file_size > MAX_APP_SIZE) {
    err("File too large: %lld > %d", file_size, MAX_APP_SIZE);
    f_close(&fp);
    return false;
  }

  if (f_lseek(&fp, 0) == -1) {
    err("Seeking: %s", strerror(errno));
    f_close(&fp);
    return false;
  }

  size_t program_size = 0;
  uint8_t buffer[FLASH_SECTOR_SIZE] = {0};
  size_t len = 0;

  // Program flash in FLASH_SECTOR_SIZE chunks
  while (f_read(&fp, buffer, sizeof(buffer), &len) == FR_OK) {
    if (len == 0)
      break;

    // Ensure we don't write beyond the application area
    if ((program_size + len) > MAX_APP_SIZE) {
      err("Write beyond app area detected");
      f_close(&fp);
      return false;
    }

    uint32_t ints = save_and_disable_interrupts();
    flash_range_erase(SD_BOOT_FLASH_OFFSET + program_size, len);
    flash_range_program(SD_BOOT_FLASH_OFFSET + program_size, buffer, len);
    restore_interrupts(ints);

    program_size += len;
    screen.draw_rectangle(RGB(255, 255, 255), 6, 6 * line + 1,
                          148 * program_size / file_size, 4);
    screen.blit();
  }

  line += 1;
  log("Successfully loaded application");
  f_close(&fp);
  return true;
}

void __not_in_flash_func(launch_application_from)(uint32_t *app_location) {
  // https://vanhunteradams.com/Pico/Bootloader/Bootloader.html
  uint32_t *new_vector_table = app_location;
  volatile uint32_t *vtor = (uint32_t *)(PPB_BASE + VTOR_OFFSET);
  *vtor = (uint32_t)new_vector_table;
  asm volatile("msr msp, %0\n"
               "bx %1\n"
               :
               : "r"(new_vector_table[0]), "r"(new_vector_table[1])
               :);
}

static bool is_valid_application(uint32_t *app_location) {
  // Check that the initial stack pointer is within a plausible RAM region
  // (assumed range for Pico: 0x20000000 to 0x20040000)
  uint32_t stack_pointer = app_location[0];
  if (stack_pointer < 0x20000000 || stack_pointer > MAX_RAM) {
    return false;
  }

  // Check that the reset vector is within the valid flash application area
  uint32_t reset_vector = app_location[1];
  if (reset_vector < (0x10000000 + SD_BOOT_FLASH_OFFSET) ||
      reset_vector > (0x10000000 + PICO_FLASH_SIZE_BYTES)) {
    return false;
  }
  return true;
}

int main() {
  timer_hw->dbgpause = 0;

  screen = Screen();
  storage = Storage();

  screen.set_backlight(true);

  FRESULT fr_mount = storage.mount();
  if (!storage.is_ready()) {
    log("SD card not detected.");
    log(RGB(200, 200, 200), "Insert an SD card and reboot.");
    log("");
    log(RGB(255, 100, 100), "MOUNT " + std::to_string(fr_mount));

    screen.draw_text(RGB(200, 200, 200), "NO_SD", 6, 110);
    screen.blit();

    return -1;
  }

  if (!stdio_init_all() || !set_sys_clock_khz(270000, true)) {
    err("Init failed");
    return -1;
  }

  if (cyw43_arch_init_with_country(21067)) {
    err("WiFi init failed");
    return -1;
  }

  bool load_success = load_program("program.bin");
  uint32_t *app_location = (uint32_t *)(XIP_BASE + SD_BOOT_FLASH_OFFSET);
  bool has_valid_app = is_valid_application(app_location);

  if (load_success || has_valid_app) {
    sleep_ms(100);
    launch_application_from(app_location);
  } else {
    err("Error when loading app");
    sleep_ms(2000);
    watchdog_reboot(0, 0, 0);
  }

  FOREVER tight_loop_contents();
}
