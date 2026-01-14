#!/usr/bin/env bash
set -e

mkdir -p build
cd build

if [ -z "$1" ]; then
    cmake ".."
    make

    sudo openocd \
        -f interface/cmsis-dap.cfg \
        -f target/rp2040.cfg \
        -c "adapter speed 5000; program firmware.elf verify reset exit"
else
    cmake "../examples/$1"
    make
fi
