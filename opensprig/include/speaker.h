#ifndef OPENSPRIG_SPEAKER_H
#define OPENSPRIG_SPEAKER_H

#include <pico/types.h>

#define SINE_WAVE_TABLE_LEN 2048

class Speaker {
  int16_t sine_wave_table[SINE_WAVE_TABLE_LEN];

public:
  Speaker();
  ~Speaker();

  void sine(uint32_t step, uint32_t volume);
};

#endif // OPENSPRIG_SPEAKER_H
