use core::time::Duration;
use esp_hal::{
    gpio::{AnyPin, Level},
    peripherals::RMT,
    rmt::{Channel, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator},
    time::Rate,
    Blocking,
};
use rgb::RGB8;

const CLOCK_DIVIDER: u8 = 2;
const CLOCK_RATE: Rate = Rate::from_mhz(80);
const TICKS_PER_HZ: Rate = Rate::from_mhz(CLOCK_RATE.as_mhz() / CLOCK_DIVIDER as u32);

type LedChannel = Channel<Blocking, 3>;

pub struct Indicator(LedChannel);

impl Indicator {
    fn reset_pulse() -> u32 {
        PulseCode::new(
            Level::Low,
            Indicator::duration_to_ticks(TICKS_PER_HZ, &Duration::from_micros(50)),
            Level::Low,
            0,
        )
    }

    fn high_pulse() -> u32 {
        PulseCode::new(
            Level::High,
            Indicator::duration_to_ticks(TICKS_PER_HZ, &Duration::from_nanos(800)),
            Level::Low,
            Indicator::duration_to_ticks(TICKS_PER_HZ, &Duration::from_nanos(450)),
        )
    }

    fn low_pulse() -> u32 {
        PulseCode::new(
            Level::High,
            Indicator::duration_to_ticks(TICKS_PER_HZ, &Duration::from_nanos(400)),
            Level::Low,
            Indicator::duration_to_ticks(TICKS_PER_HZ, &Duration::from_nanos(850)),
        )
    }

    pub fn new(rmt: RMT, led_pin: AnyPin, initial_color: RGB8) -> Self {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();
        let tx_config = TxChannelConfig::default().with_clk_divider(CLOCK_DIVIDER);
        let channel = rmt.channel3.configure(led_pin, tx_config).unwrap();
        let reset_pulse = [Self::reset_pulse()];
        let transaction = channel.transmit(&reset_pulse).unwrap();
        Self(transaction.wait().unwrap()).set_pixel(initial_color)
    }

    #[must_use]
    pub fn set_pixel(self, rgb: RGB8) -> Self {
        // :huh: wsb2812 leds are GRB, not RGB but this works?
        let color: u32 = ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32);

        let mut data = [PulseCode::empty(); 25];
        for i in 0..24 {
            let bit = (color >> (23 - i)) & 0x01 != 0;
            data[i] = if bit {
                Self::high_pulse()
            } else {
                Self::low_pulse()
            };
        }

        // Add reset pulse
        data[24] = Self::reset_pulse();

        let transaction = self.0.transmit(&data).unwrap();
        Self(transaction.wait().unwrap())
    }

    fn duration_to_ticks(ticks_rate: Rate, duration: &Duration) -> u16 {
        let ticks = duration
            .as_nanos()
            .checked_mul(ticks_rate.as_hz() as u128)
            .ok_or("Overflow")
            .unwrap()
            / 1_000_000_000;

        u16::try_from(ticks).unwrap()
    }
}
