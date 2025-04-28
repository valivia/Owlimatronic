use core::time::Duration;
use esp_hal::{
    gpio::{AnyPin, Level},
    peripherals::RMT,
    rmt::{Channel, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator},
    time::Rate,
    Blocking,
};
use rgb::RGB8;

static CLOCK_DIVIDER: u8 = 2;

type LedChannel = Channel<Blocking, 3>;

pub struct Indicator {}
impl<'d> Indicator {
    pub fn initialize(rmt: RMT, led_pin: AnyPin) -> LedChannel {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();
        let tx_config = TxChannelConfig::default().with_clk_divider(CLOCK_DIVIDER);
        let channel = rmt.channel3.configure(led_pin, tx_config).unwrap();
        channel
    }

    pub fn set_pixel(channel: LedChannel, rgb: RGB8) -> LedChannel {
        let color: u32 = ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32);

        let ticks_per_hz = Rate::from_mhz(80) / CLOCK_DIVIDER as u32;

        // 0.4 us
        let ticks_0_state_high =
            Indicator::duration_to_ticks(ticks_per_hz, &Duration::from_nanos(400));
        // 0.85 us
        let ticks_0_state_low =
            Indicator::duration_to_ticks(ticks_per_hz, &Duration::from_nanos(850));

        // 0.8 us
        let ticks_1_state_high =
            Indicator::duration_to_ticks(ticks_per_hz, &Duration::from_nanos(800));
        // 0.45 us
        let ticks_1_state_low =
            Indicator::duration_to_ticks(ticks_per_hz, &Duration::from_nanos(450));

        let mut data = [PulseCode::empty(); 25];
        for i in 0..24 {
            let bit = (color >> (23 - i)) & 0x01 != 0;
            data[i] = if bit {
                PulseCode::new(
                    Level::High,
                    ticks_1_state_high,
                    Level::Low,
                    ticks_1_state_low,
                )
            } else {
                PulseCode::new(
                    Level::High,
                    ticks_0_state_high,
                    Level::Low,
                    ticks_0_state_low,
                )
            };
        }

        // Add reset pulse
        data[24] = PulseCode::new(
            Level::Low,
            Indicator::duration_to_ticks(ticks_per_hz, &Duration::from_micros(50)),
            Level::Low,
            0,
        );

        let transaction = channel.transmit(&data).unwrap();
        transaction.wait().unwrap()
    }

    fn duration_to_ticks(ticks_hz: Rate, duration: &Duration) -> u16 {
        let ticks = duration
            .as_nanos()
            .checked_mul(ticks_hz.as_hz() as u128)
            .ok_or_else(|| "Overflow")
            .unwrap()
            / 1_000_000_000;

        u16::try_from(ticks).unwrap()
    }
}
