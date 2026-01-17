#ifndef OPENSPRIG_STORAGE_H
#define OPENSPRIG_STORAGE_H

#include <ff.h>
#include <pico/types.h>

class Storage {
  bool mounted;
  FATFS fs;

public:
  Storage();
  ~Storage();

  bool is_ready();
  FRESULT mount();
  FRESULT unmount();
  FRESULT raw_write(FIL *file, char *text);
  FRESULT write(const char *const path, char *text);
  FRESULT raw_read(FIL *file, void *buffer, uint64_t buffer_size);
  FRESULT read(const char *const path, void *buffer, uint64_t buffer_size,
               uint64_t offset);
};

#endif // OPENSPRIG_STORAGE_H
