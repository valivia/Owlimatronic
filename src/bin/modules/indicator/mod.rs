use controller::Indicator;
use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use esp_hal::{gpio::AnyPin, peripherals::RMT};
pub use rgb::RGB8;

pub mod controller;

pub static INDICATOR_QUEUE: Signal<CriticalSectionRawMutex, RGB8> = Signal::new();

#[embassy_executor::task]
pub async fn indicator_task(rmt: RMT, led_pin: AnyPin) {
    info!("Indicator task started");
    let mut led_channel = Indicator::initialize(rmt, led_pin);

    loop {
        let value = INDICATOR_QUEUE.wait().await;
        led_channel = Indicator::set_pixel(led_channel, value);
    }
}
