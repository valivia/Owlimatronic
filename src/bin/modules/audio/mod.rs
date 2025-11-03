use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_futures::yield_now;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant};
use esp_hal::peripherals::DMA_CH0;
use esp_hal::{
    dma_buffers,
    gpio::{AnyPin, OutputPin},
    i2s::master::{DataFormat, I2s, I2sTx, Standard},
    peripherals::I2S0,
    time::Rate,
    Async,
};
use ringbuf::traits::Consumer;
use tracks::Tracks;

use crate::modules::connectivity::streamer::StreamConsumer;

pub mod tracks;

pub static AUDIO_QUEUE: Signal<CriticalSectionRawMutex, Tracks> = Signal::new();
pub static AUDIO_STREAM: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static BUFFER_SIZE: usize = 4 * 4092;

const TAG: &str = "[AUDIO]";

#[embassy_executor::task]
pub async fn audio_task(
    i2s_peripheral: I2S0<'static>,
    dma_channel: DMA_CH0<'static>,
    clock_pin: AnyPin<'static>,
    data_pin: AnyPin<'static>,
    ws_pin: AnyPin<'static>,
    stream_consumer: StreamConsumer,
) {
    let mut audio_controller = AudioService::new(
        i2s_peripheral,
        dma_channel,
        clock_pin,
        data_pin,
        ws_pin,
        stream_consumer,
    )
    .await;
    info!("{} task started", TAG);
    loop {
        audio_controller.run_loop().await;
    }
}

pub struct AudioService {
    tx: I2sTx<'static, Async>,
    tx_buffer: &'static mut [u8; BUFFER_SIZE],
    stream_consumer: StreamConsumer,
}

impl AudioService {
    pub async fn new(
        i2s_peripheral: I2S0<'static>,
        dma_channel: DMA_CH0<'static>,
        clock_pin: impl OutputPin + 'static,
        data_pin: impl OutputPin + 'static,
        ws_pin: impl OutputPin + 'static,
        stream_consumer: StreamConsumer,
    ) -> Self {
        let (_, _rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(0, BUFFER_SIZE);

        let i2s = I2s::new(
            i2s_peripheral,
            Standard::Philips,
            DataFormat::Data16Channel16,
            Rate::from_hz(16000u32),
            dma_channel,
        )
        .into_async();

        let tx = i2s
            .i2s_tx
            .with_bclk(clock_pin)
            .with_dout(data_pin)
            .with_ws(ws_pin)
            .build(tx_descriptors);

        AudioService {
            tx,
            tx_buffer,
            stream_consumer,
        }
    }

    async fn run_loop(&mut self) {
        match select(AUDIO_QUEUE.wait(), AUDIO_STREAM.wait()).await {
            Either::First(track) => {
                self.play_track(track).await;
            }
            Either::Second(_) => {
                self.stream_live().await;
            }
        }

        AUDIO_QUEUE.reset();
        AUDIO_STREAM.reset();
    }

    async fn play_track(&mut self, track: Tracks) {
        let pcm = track.get_file();
        info!(
            "{} playing local file '{}' ({} bytes)",
            TAG,
            track.get_name(),
            pcm.len()
        );

        self.play_mono_pcm(&pcm).await;
        info!("{} done playing local file", TAG);
    }

    /// Stream live mono PCM
    async fn stream_live(&mut self) {
        info!("{} starting live PCM stream", TAG);

        let start_time = Instant::now();

        loop {
            while let Some(chunk) = self.stream_consumer.try_pop() {
                self.write_mono_chunk(&chunk.data[..chunk.len]).await;
            }

            if Instant::now() - start_time > Duration::from_secs(1) {
                break;
            }

            yield_now().await;
        }

        info!("{} live stream ended", TAG);
    }

    /// Play a complete mono PCM buffer
    async fn play_mono_pcm(&mut self, pcm: &[u8]) {
        let chunk_size = self.tx_buffer.len() / 2;
        let mut pos = 0;

        while pos < pcm.len() {
            let end = (pos + chunk_size).min(pcm.len());
            let chunk = &pcm[pos..end];
            self.write_mono_chunk(chunk).await;
            pos = end;
            yield_now().await;
        }
    }

    /// Convert a mono chunk to stereo and send it to I2S via DMA
    async fn write_mono_chunk(&mut self, chunk: &[u8]) {
        // Each sample = 2 bytes (16-bit)
        let stereo_len = chunk.len() * 2;
        let stereo = &mut self.tx_buffer[..stereo_len];

        // Duplicate mono samples into left/right
        for (i, sample) in chunk.chunks_exact(2).enumerate() {
            stereo[i * 4..i * 4 + 2].copy_from_slice(sample); // Left
            stereo[i * 4 + 2..i * 4 + 4].copy_from_slice(sample); // Right
        }

        if let Err(e) = self.tx.write_dma_async(stereo).await {
            info!("{} DMA error: {:?}", TAG, e);
        }
    }
}
