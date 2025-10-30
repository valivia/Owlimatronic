use defmt::info;
use embassy_futures::yield_now;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_hal::peripherals::DMA_CH0;
use esp_hal::{
    dma_buffers,
    gpio::{AnyPin, OutputPin},
    i2s::master::{DataFormat, I2s, I2sTx, Standard},
    peripherals::I2S0,
    time::Rate,
    Async,
};
use tracks::Tracks;

pub mod tracks;

pub static AUDIO_QUEUE: Channel<CriticalSectionRawMutex, Tracks, 4> = Channel::new();
static BUFFER_SIZE: usize = 4 * 4092;

const TAG: &str = "[AUDIO]";

#[embassy_executor::task]
pub async fn audio_task(
    i2s_peripheral: I2S0<'static>,
    dma_channel: DMA_CH0<'static>,
    clock_pin: AnyPin<'static>,
    data_pin: AnyPin<'static>,
    ws_pin: AnyPin<'static>,
) {
    let mut audio_controller =
        AudioService::new(i2s_peripheral, dma_channel, clock_pin, data_pin, ws_pin).await;
    info!("{} task started", TAG);
    loop {
        audio_controller.run_loop().await;
    }
}

pub struct AudioService {
    tx: I2sTx<'static, Async>,
    tx_buffer: &'static mut [u8; BUFFER_SIZE],
}

impl AudioService {
    pub async fn new(
        i2s_peripheral: I2S0<'static>,
        dma_channel: DMA_CH0<'static>,
        clock_pin: impl OutputPin + 'static,
        data_pin: impl OutputPin + 'static,
        ws_pin: impl OutputPin + 'static,
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

        AudioService { tx, tx_buffer }
    }

    async fn run_loop(&mut self) {
        let track = AUDIO_QUEUE.receive().await;
        self.play(track).await;
    }

    async fn play(&mut self, track: Tracks) {
        let audio_data = track.get_file();
        info!(
            "{} file ({}) loaded, length: {}",
            TAG,
            track.get_name(),
            audio_data.len()
        );

        let chunk_size = self.tx_buffer.len() / 2;

        let mut pos = 0;

        while pos < audio_data.len() {
            let chunk_end = (pos + chunk_size).min(audio_data.len());
            let mono_chunk = &audio_data[pos..chunk_end];

            // Convert mono -> stereo
            let stereo_chunk = &mut self.tx_buffer[..mono_chunk.len() * 2];
            for (i, sample) in mono_chunk.chunks_exact(2).enumerate() {
                // copy 16-bit sample into left and right channels
                stereo_chunk[i * 4..i * 4 + 2].copy_from_slice(sample); // left
                stereo_chunk[i * 4 + 2..i * 4 + 4].copy_from_slice(sample); // right
            }

            // Write DMA
            match self.tx.write_dma_async(stereo_chunk).await {
                Ok(tx) => tx,
                Err(e) => {
                    info!("Error initializing DMA: {:?}", e);
                    return;
                }
            };

            pos = chunk_end;
            yield_now().await;
        }

        info!("{} Finished playing.", TAG);
    }
}
