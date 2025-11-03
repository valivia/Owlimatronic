use defmt::info;
use embassy_time::Timer;
use esp_hal::gpio::{AnyPin, Event, Input, InputConfig, Pull};

use crate::modules::servo::{animation::ANIMATION_QUEUE, animations::AnimationType};

const TAG: &str = "[INTERACTION]";

#[embassy_executor::task]
pub async fn interaction_task(beak_pin: AnyPin<'static>) {
    info!("{} interaction task started", TAG);
    let input_button_cfg = InputConfig::default().with_pull(Pull::Up);

    // Beak
    let mut beak_button = Input::new(beak_pin, input_button_cfg);
    beak_button.listen(Event::FallingEdge);

    // Touch
    // TODO implement this when it becomes available https://github.com/esp-rs/esp-hal/issues/1905

    loop {
        beak_button.wait_for_falling_edge().await;
        Timer::after(embassy_time::Duration::from_millis(100)).await;
        if beak_button.is_high() {
            continue;
        }
        info!("{} Beak button pressed", TAG);
        ANIMATION_QUEUE.send(AnimationType::Yap).await;
    }
}
