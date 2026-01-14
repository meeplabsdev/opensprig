cmake_minimum_required(VERSION 3.13)

set(PICO_BOARD pico_w)

include(../../pico_sdk_import.cmake)
include(../../pico_extras_import.cmake)

project(badapple C CXX ASM)

pico_sdk_init()

add_subdirectory(../../opensprig build)

add_executable(badapple main.cpp)

target_link_libraries(badapple
    pico_stdlib
    pico_cyw43_arch_none
    opensprig
)

pico_set_linker_script(badapple ${CMAKE_CURRENT_LIST_DIR}/../../memmap.ld)
target_link_options(badapple PRIVATE
    "-Wl,--print-memory-usage"
)

# Choose source and destination for standard input and output:
#   See 4.1. Serial input and output on Raspberry Pi Pico in Getting started with Raspberry Pi Pico (https://datasheets.raspberrypi.org/pico/getting-started-with-pico.pdf)
#   and 2.7.1. Standard Input/Output (stdio) Support in Raspberry Pi Pico C/C++ SDK (https://datasheets.raspberrypi.org/pico/raspberry-pi-pico-c-sdk.pdf):
pico_enable_stdio_uart(badapple 1)
pico_enable_stdio_usb(badapple 1)

pico_add_extra_outputs(badapple)
