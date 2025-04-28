use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_time::Timer;
use esp_hal::{
    gpio::{AnyPin, Event, Input, InputConfig, Pull, WakeEvent},
    system::software_reset,
};
use rgb::RGB8;

use crate::modules::indicator::INDICATOR_QUEUE;

#[derive(PartialEq, Copy, Clone)]
pub enum SystemMode {
    Play,
    Mailbox,
    Off,
}

pub async fn initialize_mode(mode_pin_1: AnyPin, mode_pin_2: AnyPin) -> SystemMode {
    let input_button_cfg = InputConfig::default().with_pull(Pull::Up);
    let play_state = Input::new(mode_pin_1, input_button_cfg);
    let mut common_state = Input::new(mode_pin_2, input_button_cfg);

    let system_mode = match (play_state.is_low(), common_state.is_low()) {
        (true, true) => SystemMode::Play,
        (false, true) => SystemMode::Mailbox,
        (false, false) => SystemMode::Off,
        _ => software_reset(),
    };

    match system_mode {
        SystemMode::Play => {
            info!("System mode: Play");
            INDICATOR_QUEUE.signal(RGB8::new(0, 255, 0));
        }
        SystemMode::Mailbox => {
            info!("System mode: Mailbox");
            INDICATOR_QUEUE.signal(RGB8::new(0, 0, 255));
        }
        SystemMode::Off => {
            info!("off");
            INDICATOR_QUEUE.signal(RGB8::new(0, 0, 0));
            Timer::after(embassy_time::Duration::from_millis(100)).await;

            common_state.listen(Event::LowLevel);
            common_state
                .wakeup_enable(true, WakeEvent::LowLevel)
                .unwrap();
            common_state.wait_for_low().await;

            info!("turning on");

            // restart
            software_reset();
        }
    }

    Spawner::for_current_executor()
        .await
        .spawn(mode_task(play_state, common_state, system_mode))
        .unwrap();

    system_mode
}

#[embassy_executor::task]
pub async fn mode_task(
    mut play_state: Input<'static>,
    mut common_state: Input<'static>,
    mode: SystemMode,
) {
    loop {
        match mode {
            SystemMode::Play => {
                play_state.wait_for_rising_edge().await;
                Timer::after(embassy_time::Duration::from_millis(10)).await;
                if play_state.is_low() {
                    info!("exit play debounced");
                    continue;
                }
                info!("exited play mode");
                software_reset();
            }
            SystemMode::Mailbox => {
                select(
                    play_state.wait_for_falling_edge(),
                    common_state.wait_for_rising_edge(),
                )
                .await;
                Timer::after(embassy_time::Duration::from_millis(10)).await;
                if common_state.is_low() && play_state.is_high() {
                    info!("Exit mailbox debounced");
                    continue;
                }
                info!("exited mailbox mode");
                software_reset();
            }
            SystemMode::Off => {}
        }
    }
}
