# opensprig

<img width="1920" height="1080" alt="frame_73" src="https://github.com/user-attachments/assets/be8c0990-6044-48b6-9b5f-5439ac825ea1" />

Custom firmware for the Sprig! Makes use of the currently unused features like the SD slot built into the screen and the wireless functionality of the Pico W.

## What is it?
opensprig is a project designed to make it easier to write much lower level apps for the [sprig](https://sprig.hackclub.com/get). It currently supports the following features:
- Macro to do setup for all sprig hardware automatically
- Reading from the buttons
- Writing the the LEDs (supports PWM) and the builtin LED (no PWM)
- Writing to the screen (also supports some basic helpers, eg. flood, rectangle)
- Playing audio in the PCM format
- Reading/writing files, listing/creating/deleting directories on an SD card inserted into the screen TFT slot
- Parsing custom fonts in the BDF format
- Writing text to the screen with a custom font

It also comes with some examples to show off these features:
- blink - simple blink for the pico builtin LED
- hardware_checker - displays pressed button on screen, flashes LEDs, plays audio read from the SD card, writes "Hello, world!" to the screen using custom parsed font

## Flashing
First you must build the firmware. A script is provided to assist with this:
`./build.sh <--debug>`. This will build the firmware and all apps (optionally as a debug release) using the correct memory.x for each. The output will be in `build/thumbv6m-none-eabi/release/`, with the firmware being named `firmware` and the applications each being `name.bin`. You can flash the firmware like any normal pico firmware, for example: `probe-rs run --chip RP2040 firmware`. You will also need to place the proprietary cyw43 firmware, which can be done with `./cyw43/place.sh`. This script assumes probe-rs is being used. If this is not the case, you must place the cyw43 bins at the addresses specified inside the `place.sh` script.

The firmware is now successfully written to the pico, but booting will currently show no options on the screen. This is because no applications are present on the SD card. Prepare an SD card in the FAT32 format and create a directory `/program/`, placing the built applications (`name.bin`) here. Plug the SD into the sprig (the slot is in the top of the screen) and reboot it. The applications should be present in a list. It can be navigated using using the left dpad "up" and "down", and selected using the right dpad "left". When a program selected, it will reboot into the program. On the next reboot, it will return to the main firmware.

> Note: to use hardware checker example, the SD card must have the `/audio/believer.pcm` file, which can be obtained with the command `ffmpeg -i believer.flac -ar 24000 -ac 1 -f u16le -acodec pcm_u16le believer.pcm`.

## Development
The project is structured into a cargo workspace. The main firmware is present under `/firmware/`, and the main library under `/opensprig-rs/`. Each application is under `/parts/name/`. To build, follow the instructions above in the **Flashing** section. No AI pull requests will be accepted.

