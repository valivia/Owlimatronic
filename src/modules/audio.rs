use esp_idf_hal::{
    gpio::AnyOutputPin,
    i2s::{
        config::{Config, PdmTxClkConfig, PdmTxConfig, PdmTxGpioConfig, PdmTxSlotConfig},
        I2s, I2sDriver, I2sTx,
    },
    peripheral::Peripheral,
};

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

    pub fn play(&mut self, data: &[u8]) {
        self.driver.write(data, 1000).unwrap();
    }
}
