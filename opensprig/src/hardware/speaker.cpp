#include "speaker.h"

#include <cmath>
#include <pico/audio_i2s.h>

static struct audio_buffer_pool *audio_bufpool;

Speaker::Speaker() {
  static struct audio_format audio_format = {
      .sample_freq = 24000,
      .format = AUDIO_BUFFER_FORMAT_PCM_S16,
      .channel_count = 1,
  };

  static struct audio_buffer_format producer_format = {.format = &audio_format,
                                                       .sample_stride = 2};

  struct audio_i2s_config config = {
      .data_pin = PICO_AUDIO_I2S_DATA_PIN,
      .clock_pin_base = PICO_AUDIO_I2S_CLOCK_PIN_BASE,
      .dma_channel = 0,
      .pio_sm = 0,
  };

  audio_bufpool = audio_new_producer_pool(&producer_format, 3, 256 * 8);

  audio_i2s_setup(&audio_format, &config);
  audio_i2s_connect(audio_bufpool);
  audio_i2s_set_enabled(true);

  for (int i = 0; i < SINE_WAVE_TABLE_LEN; i++)
    this->sine_wave_table[i] =
        32767 * cosf(i * 2 * (float)(M_PI / SINE_WAVE_TABLE_LEN));
}

Speaker::~Speaker() { audio_i2s_set_enabled(false); }

void Speaker::sine(uint32_t step, uint32_t volume) {
  struct audio_buffer *buffer = take_audio_buffer(audio_bufpool, false);
  if (buffer == NULL)
    return;

  uint32_t pos_max = 0x10000 * SINE_WAVE_TABLE_LEN;
  uint32_t pos = 0;

  int16_t *samples = (int16_t *)buffer->buffer->bytes;
  for (uint i = 0; i < buffer->max_sample_count; i++) {
    samples[i] = (volume * this->sine_wave_table[pos >> 16u]) >> 8u;

    pos += step;
    if (pos >= pos_max)
      pos -= pos_max;
  }
  buffer->sample_count = buffer->max_sample_count;
  give_audio_buffer(audio_bufpool, buffer);
}
