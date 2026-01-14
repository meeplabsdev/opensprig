cmake_minimum_required(VERSION 3.13)

add_library(opensprig
    src/hardware/button.cpp
    src/hardware/led.cpp
    src/hardware/screen.cpp
    src/hardware/speaker.cpp
    src/hardware/storage.cpp
    src/ui/window.cpp
    src/ui/widget.cpp
    src/ui/widgets/text.cpp
)

target_compile_definitions(opensprig PRIVATE
    PICO_AUDIO_I2S_MONO_INPUT=1
    USE_AUDIO_I2S=1
    PICO_AUDIO_I2S_DATA_PIN=9
    PICO_AUDIO_I2S_CLOCK_PIN_BASE=10
)

target_include_directories(opensprig PRIVATE src/def)
target_include_directories(opensprig PUBLIC include)

add_subdirectory(src/fatfs/src build)
target_link_libraries(opensprig PUBLIC no-OS-FatFS-SD-SDIO-SPI-RPi-Pico)
target_sources(opensprig PRIVATE src/def/hw_config.cpp)

if (PICO_BOARD STREQUAL "pico_w")
    set(TARGET_LIBS pico_cyw43_arch_none)
else ()
    set(TARGET_LIBS)
endif ()

target_link_libraries(opensprig PRIVATE
    pico_stdlib
    pico_multicore
    pico_audio_i2s
    hardware_timer
    hardware_adc
    hardware_pwm
    hardware_spi
    ${TARGET_LIBS}
)
