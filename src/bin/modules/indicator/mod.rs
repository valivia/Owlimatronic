use controller::Indicator;
use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use esp_hal::{gpio::AnyPin, peripherals::RMT};
pub use rgb::RGB8;

pub mod controller;

pub static INDICATOR_QUEUE: Signal<CriticalSectionRawMutex, RGB8> = Signal::new();

#[embassy_executor::task]
pub async fn indicator_task(rmt: RMT<'static>, led_pin: AnyPin<'static>) {
    info!("Indicator task started");
    let mut indicator = Indicator::new(rmt, led_pin, RGB8::new(0, 0, 0));

    loop {
        let value = INDICATOR_QUEUE.wait().await;
        indicator = indicator.set_pixel(value);
    }
}
