cmake_minimum_required(VERSION 3.13)

set(PICO_BOARD pico_w)

include(pico_sdk_import.cmake)
include(pico_extras_import.cmake)

project(firmware C CXX ASM)

pico_sdk_init()

add_subdirectory(opensprig build)

set(RUST_TARGET thumbv6m-none-eabi)
set(RUST_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust/target)

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(RUST_BUILD_FLAG --release)
    set(RUST_BUILD_MODE release)
else()
    set(RUST_BUILD_FLAG)
    set(RUST_BUILD_MODE debug)
endif()

set(RUST_LIB ${RUST_TARGET_DIR}/${RUST_TARGET}/${RUST_BUILD_MODE}/libopensprig_rs.a)

add_custom_command(
    OUTPUT ${RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR}
        cargo build ${RUST_BUILD_FLAG} --target=${RUST_TARGET}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    COMMENT "Performing build step for 'rust'"
    VERBATIM
)

add_custom_target(rust_build ALL DEPENDS ${RUST_LIB})

add_library(rust_firmware STATIC IMPORTED GLOBAL)
add_dependencies(rust_firmware rust_build)
set_target_properties(rust_firmware PROPERTIES IMPORTED_LOCATION ${RUST_LIB})

add_executable(firmware)

target_link_libraries(firmware
    rust_firmware
    opensprig
    pico_stdlib
    pico_cyw43_arch_none
)

pico_add_extra_outputs(firmware)