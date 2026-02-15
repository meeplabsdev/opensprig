#!/usr/bin/env bash

if [ -d "cyw43" ] && [ "$(basename "$PWD")" != "cyw43" ]; then
  cd cyw43 || exit 1
fi

# 43439A0.bin:      0x101be000 - 0x101fe000 (0x386A5 / 0x40000)
# 43439A0_clm.bin:  0x101fe000 - 0x101ff000 (0x003D8 / 0x01000)
# nvram_rp2040.bin: 0x101ff000 - 0x10200000 (0x002E6 / 0x01000)
probe-rs download ./43439A0.bin --binary-format bin --chip RP2040 --base-address 0x101be000
probe-rs download ./43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x101fe000
probe-rs download ./nvram_rp2040.bin --binary-format bin --chip RP2040 --base-address 0x101ff000