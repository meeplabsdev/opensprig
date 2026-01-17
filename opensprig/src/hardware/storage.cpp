#include "storage.h"

#include <cstring>

Storage::Storage() {}

Storage::~Storage() {}

bool Storage::is_ready() {
  if (!this->mounted)
    return false;

  FIL file;
  if (f_open(&file, ".sprig", FA_CREATE_ALWAYS | FA_WRITE) != FR_OK)
    return false;

  f_close(&file);

  return true;
}

FRESULT Storage::mount() {
  FRESULT fr = f_mount(&this->fs, "", 1);
  if (fr == FR_OK)
    this->mounted = true;

  return fr;
}

FRESULT Storage::unmount() {
  FRESULT fr = f_unmount("");
  if (fr == FR_OK)
    this->mounted = false;

  return fr;
}

FRESULT Storage::raw_write(FIL *file, char *text) {
  UINT bytes_written;
  return f_write(file, text, strlen(text), &bytes_written);
}

FRESULT Storage::write(const char *const path, char *text) {
  if (!is_ready())
    return FR_NOT_READY;

  FIL file;
  FRESULT fr = f_open(&file, path, FA_CREATE_ALWAYS | FA_WRITE);
  if (fr != FR_OK)
    return fr;

  fr = raw_write(&file, text);

  f_close(&file);
  return fr;
}

FRESULT Storage::raw_read(FIL *file, void *buffer, uint64_t buffer_size) {
  UINT bytes_read = 0;
  return f_read(file, buffer, buffer_size, &bytes_read);
}

FRESULT Storage::read(const char *const path, void *buffer,
                      uint64_t buffer_size, uint64_t offset) {
  if (!is_ready())
    return FR_NOT_READY;

  FIL file;
  FRESULT fr = f_open(&file, path, FA_READ);
  if (fr != FR_OK)
    return fr;

  uint64_t file_size = f_size(&file);
  if (offset >= file_size)
    return FR_INVALID_PARAMETER;

  uint64_t current_pos = f_tell(&file);
  if (current_pos != offset) {
    FRESULT fr = f_lseek(&file, offset);
    if (fr != FR_OK)
      f_close(&file);
    return fr;
  }

  fr = raw_read(&file, buffer, buffer_size);

  f_close(&file);
  return fr;
}
