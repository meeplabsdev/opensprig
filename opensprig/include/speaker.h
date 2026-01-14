#ifndef OPENSPRIG_SPEAKER_H
#define OPENSPRIG_SPEAKER_H

#include <cmath>
#include <cstdint>
#include <hardware/adc.h>
#include <pico/audio.h>
#include <pico/audio_i2s.h>

#define SINE_WAVE_TABLE_LEN 2048

class Speaker {
  int16_t sine_wave_table[SINE_WAVE_TABLE_LEN];

public:
  explicit Speaker();
  ~Speaker();
  void sine(uint32_t step, uint32_t volume);
};

#endif // OPENSPRIG_SPEAKER_H
