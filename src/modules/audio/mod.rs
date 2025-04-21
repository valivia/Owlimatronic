use std::{
    sync::mpmc::{channel, Receiver, Sender},
    thread,
};

use esp_idf_hal::{
    gpio::AnyOutputPin,
    i2s::{
        config::{Config, PdmTxClkConfig, PdmTxConfig, PdmTxGpioConfig, PdmTxSlotConfig},
        I2s, I2sDriver, I2sTx,
    },
    peripheral::Peripheral,
};
use files::AudioFiles;
use log::{info, warn};

pub mod files;

pub struct AudioController<'a> {
    driver: I2sDriver<'a, I2sTx>,
}

// WS 35
// Clock 36
// Data 37

pub struct AudioConfig {
    pub clk: AnyOutputPin,
    pub dout: AnyOutputPin,
}

impl<'a> AudioController<'a> {
    pub fn new(i2s: impl Peripheral<P = impl I2s> + 'a, config: AudioConfig) -> Self {
        let cfg = PdmTxConfig::new(
            Config::default(),
            PdmTxClkConfig::from_sample_rate_hz(16000u32),
            PdmTxSlotConfig::default(),
            PdmTxGpioConfig::default(),
        );

        let ws: Option<AnyOutputPin> = None;

        let driver = I2sDriver::new_pdm_tx(i2s, &cfg, config.clk, config.dout, ws).unwrap();

        return AudioController { driver };
    }

    pub fn start_audio_thread(controller: AudioController<'static>) -> Sender<AudioFiles> {
        let (tx, rx): (Sender<AudioFiles>, Receiver<AudioFiles>) = channel();

        thread::spawn(move || {
            let mut controller = controller;

            for audio_file in rx {
                let data = audio_file.get_file();

                // Log and play
                info!("Audio thread received");
                controller.play(data);
            }

            warn!("Audio thread exited.");
        });

        tx
    }

    pub fn play(&mut self, data: &[u8]) {
        info!("Playing audio");
        self.driver.tx_enable().unwrap();
        self.driver.write_all(data, 1000).unwrap();
        self.driver.tx_disable().unwrap();
        info!("Finished audio");
    }
}
