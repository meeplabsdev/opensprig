# opensprig

<img width="1920" height="1080" alt="frame_73" src="https://github.com/user-attachments/assets/be8c0990-6044-48b6-9b5f-5439ac825ea1" />

Custom firmware for the Sprig! Makes use of the currently unused features like the SD slot built into the screen and the wireless functionality of the Pico W.

Progress:

(master)
- LEDs
- buttons
- screen
- storage
- sound
- badapple! example
- helloworld example
- (This is basically fully integrated POC)

(rust)
- LEDs
- buttons
- screen
- storage
- sound
- helloworld example
- hardware checker example
- (no badapple!, but this would be easy to do)
- (This codebase is much nicer)

note: to use hardware checker example, the sd card must have the /audio/believer.pcm file, which can be obtained with the command

`ffmpeg -i believer.mp3 -ar 24000 -ac 1 -f u16le -acodec pcm_u16le believer.pcm`
