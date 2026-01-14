cmake_minimum_required(VERSION 3.13)

set(PICO_BOARD pico_w)

include(pico_sdk_import.cmake)
include(pico_extras_import.cmake)

project(firmware C CXX ASM)

pico_sdk_init()

add_subdirectory(opensprig build)

add_executable(firmware main.cpp)

target_link_libraries(firmware
        pico_stdlib
        pico_cyw43_arch_none
        opensprig
)
