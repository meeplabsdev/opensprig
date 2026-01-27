#include "speaker.h"

extern "C" {
  Speaker *Speaker_new() {
    return new Speaker();
  }

  void Speaker_free(Speaker *speaker) {
    delete speaker;
  }

  void Speaker_sine(Speaker *speaker, uint32_t step, uint32_t volume) {
    speaker->sine(step, volume);
  }
}
