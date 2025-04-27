use defmt::info;
use esp_hal::{
    dma::DmaChannel0,
    dma_buffers,
    gpio::AnyPin,
    i2s::master::{asynch::I2sWriteDmaTransferAsync, DataFormat, I2s, Standard},
    peripherals::{DMA, I2S0},
    time::Rate,
};

#[embassy_executor::task]
pub async fn audio_task(mut audio_controller: AudioService) {
    info!("audio task started");

    loop {
        // audio_controller.run_loop().await;
    }
}

pub struct AudioService {}

impl AudioService {
    pub async fn new(
        i2s_peripheral: I2S0,
        dma_channel: DmaChannel0,
        clock_pin: AnyPin,
        data_pin: AnyPin,
    ) -> Self {
        let (tx_buffer, rx_descriptors, _, tx_descriptors) = dma_buffers!(0, 4 * 4092);

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

        info!("I2S initialized");
        let mut tx = i2s.i2s_tx.with_bclk(clock_pin).with_dout(data_pin).build();
        info!("I2S TX initialized");

        let mut buffer = include_bytes!("owl.pcm").to_vec(); // Create a mutable buffer
        let mut tx = tx.write_dma(tx_buffer).unwrap();
        info!("I2S TX circular buffer initialized");

        loop {
            // tx.(include_bytes!("owl.pcm")).await.unwrap();
        }

        AudioService {}
    }
}
