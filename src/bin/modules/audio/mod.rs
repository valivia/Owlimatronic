use defmt::info;
use embassy_futures::yield_now;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_hal::{
    dma::DmaChannel0,
    dma_buffers,
    gpio::AnyPin,
    i2s::master::{DataFormat, I2s, I2sTx, Standard},
    peripherals::I2S0,
    time::Rate,
    Async,
};
use tracks::Tracks;

pub mod tracks;

pub static AUDIO_QUEUE: Channel<CriticalSectionRawMutex, Tracks, 4> = Channel::new();
static BUFFER_SIZE: usize = 4 * 4092;

#[embassy_executor::task]
pub async fn audio_task(
    i2s_peripheral: I2S0,
    dma_channel: DmaChannel0,
    clock_pin: AnyPin,
    data_pin: AnyPin,
    ws_pin: AnyPin,
) {
    let mut audio_controller =
        AudioService::new(i2s_peripheral, dma_channel, clock_pin, data_pin, ws_pin).await;
    info!("audio task started");
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
        i2s_peripheral: I2S0,
        dma_channel: DmaChannel0,
        clock_pin: AnyPin,
        data_pin: AnyPin,
        ws_pin: AnyPin,
    ) -> Self {
        let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(0, BUFFER_SIZE);

        let i2s = I2s::new(
            i2s_peripheral,
            Standard::Philips,
            DataFormat::Data16Channel16,
            Rate::from_hz(16000u32),
            dma_channel,
            &mut rx_descriptors[..],
            &mut tx_descriptors[..],
        )
        .into_async();

        let tx = i2s
            .i2s_tx
            .with_bclk(clock_pin)
            .with_dout(data_pin)
            .with_ws(ws_pin)
            .build();

        AudioService { tx, tx_buffer }
    }

    async fn run_loop(&mut self) {
        let track = AUDIO_QUEUE.receive().await;
        self.play(track).await;
    }

    async fn play(&mut self, track: Tracks) {
        let audio_data = track.get_file();
        info!(
            "Audio file ({}) loaded, length: {}",
            track.get_name(),
            audio_data.len()
        );

        // Make sure the buffer is large enough for the chunk size
        let chunk_size = self.tx_buffer.len(); // Use the buffer size as the chunk size

        // Make sure the audio data fits in the buffer chunks
        let mut pos = 0;

        while pos < audio_data.len() {
            // Calculate the end of the chunk (do not exceed the audio data length)
            let chunk_end = (pos + chunk_size).min(audio_data.len());
            let chunk = &audio_data[pos..chunk_end];

            // Copy the chunk into tx_buffer
            self.tx_buffer[..chunk.len()].copy_from_slice(chunk);

            // Perform the one-shot DMA write
            match self.tx.write_dma_async(self.tx_buffer).await {
                Ok(tx) => tx,
                Err(e) => {
                    info!("Error initializing DMA: {:?}", e);
                    return;
                }
            };

            // Move the position for the next chunk
            pos = chunk_end;

            // Optionally, yield to other tasks if needed
            yield_now().await;
        }

        info!("Finished playing the audio.");
    }
}
