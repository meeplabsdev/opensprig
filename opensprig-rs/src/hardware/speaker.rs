use core::f32::consts::PI;
use embassy_rp::{
    Peri,
    dma::{self, ChannelInstance},
    interrupt::typelevel::Binding,
    pio::{Common, Instance, PioPin, StateMachine},
    pio_programs::i2s::{PioI2sOut, PioI2sOutProgram},
    spi,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use micromath::F32;
use static_cell::StaticCell;

use crate::hardware::Storage;

const SAMPLE_FREQ: u32 = 24_000;
const SINE_WAVE_TABLE_LEN: usize = 256;
const AUDIO_BUF_SAMPLES: usize = 9600;
const BYTES_PER_BUF: usize = AUDIO_BUF_SAMPLES * 2;

pub struct Speaker<'a, T: Instance, const S: usize> {
    i2s: Mutex<ThreadModeRawMutex, PioI2sOut<'a, T, S>>,
    sine_wave_table: Mutex<ThreadModeRawMutex, &'a mut [u32; SINE_WAVE_TABLE_LEN]>,
}

impl<'a, T: Instance, const S: usize> Speaker<'a, T, S> {
    pub fn new<Dma: ChannelInstance>(
        common: &mut Common<'a, T>,
        sm: StateMachine<'a, T, S>,
        data_pin: Peri<'a, impl PioPin>,
        bit_clock_pin: Peri<'a, impl PioPin>,
        lr_clock_pin: Peri<'a, impl PioPin>,
        dma: Peri<'a, Dma>,
        irq: impl Binding<Dma::Interrupt, dma::InterruptHandler<Dma>> + 'a,
    ) -> Self {
        static WAVE_TABLE: StaticCell<[u32; SINE_WAVE_TABLE_LEN]> = StaticCell::new();
        let wave_table = WAVE_TABLE.init([0u32; SINE_WAVE_TABLE_LEN]);
        for i in 0..SINE_WAVE_TABLE_LEN {
            wave_table[i] = ((1.0
                + F32::from(2.0 * PI * i as f32 / SINE_WAVE_TABLE_LEN as f32)
                    .cos()
                    .0)
                * 32767.0) as u32;
        }

        let program = PioI2sOutProgram::new(common);
        let mut i2s = PioI2sOut::new(
            common,
            sm,
            dma,
            irq,
            data_pin,
            bit_clock_pin,
            lr_clock_pin,
            SAMPLE_FREQ,
            16,
            &program,
        );

        i2s.start();

        Self {
            i2s: Mutex::new(i2s),
            sine_wave_table: Mutex::new(wave_table),
        }
    }

    pub async fn sine(&self, step: u32, volume: u32) {
        let mut pos: u32 = 0;
        let pos_max: u32 = 0x10000 * SINE_WAVE_TABLE_LEN as u32;
        let mut buf = [0u32; AUDIO_BUF_SAMPLES];

        for sample in buf.iter_mut() {
            let wave = self.sine_wave_table.lock().await[(pos >> 16) as usize];
            *sample = ((volume * wave) >> 8) as u32;

            pos = pos.wrapping_add(step);
            if pos >= pos_max {
                pos -= pos_max;
            }
        }

        self.i2s.lock().await.write(buf.as_slice()).await;
    }

    pub async fn play_pcm(
        &self,
        storage: &mut Storage<'a, impl spi::Instance>,
        path: &str,
        volume: u32,
    ) {
        let mut offset = 0usize;
        let mut buf_a = [0u32; AUDIO_BUF_SAMPLES];
        let mut buf_b = [0u32; AUDIO_BUF_SAMPLES];

        let mut done = read_buf(storage, path, offset, &mut buf_a, volume).await;
        offset += BYTES_PER_BUF;

        let mut i2s = self.i2s.lock().await;
        loop {
            let transfer = i2s.write(buf_a.as_slice());
            let next_done = read_buf(storage, path, offset, &mut buf_b, volume).await;
            offset += BYTES_PER_BUF;
            transfer.await;
            if done {
                break;
            }
            done = next_done;

            let transfer = i2s.write(buf_b.as_slice());
            let next_done = read_buf(storage, path, offset, &mut buf_a, volume).await;
            offset += BYTES_PER_BUF;
            transfer.await;
            if done {
                break;
            }
            done = next_done;
        }
    }
}

async fn read_buf(
    storage: &mut Storage<'_, impl spi::Instance>,
    path: &str,
    offset: usize,
    buf: &mut [u32; AUDIO_BUF_SAMPLES],
    volume: u32,
) -> bool {
    match storage.read::<BYTES_PER_BUF>(path, offset as u32).await {
        Ok(bytes) => {
            let mut samples = bytes
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]) as u32);

            let mut done = false;
            for sample in buf.iter_mut() {
                match samples.next() {
                    Some(wave) => *sample = (volume * wave) >> 8,
                    None => {
                        *sample = 32768;
                        done = true;
                    }
                }
            }
            done
        }
        Err(_) => {
            buf.fill(0);
            true
        }
    }
}
