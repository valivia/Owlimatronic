#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use modules::audio::{audio_task, AudioService};
use modules::connectivity::wifi::wifi_init;
use modules::interaction::interaction_task;
use modules::mode::{initialize_mode, SystemMode};
use modules::servo::controller::ServoController;
use modules::servo::servo_task;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

mod modules;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    let rng = Rng::new(peripherals.RNG);

    info!("Embassy initialized!");

    // Mode
    let system_mode = initialize_mode(peripherals.GPIO8.into(), peripherals.GPIO9.into()).await;

    // Indicator

    // Servos
    let servo_controller = ServoController::new(
        peripherals.MCPWM0,
        peripherals.GPIO16.into(),
        peripherals.GPIO15.into(),
        peripherals.GPIO14.into(),
        peripherals.GPIO13.into(),
    )
    .await;

    spawner.spawn(servo_task(servo_controller)).unwrap();

    // Interaction
    spawner
        .spawn(interaction_task(peripherals.GPIO6.into()))
        .unwrap();

    // Audio
    // let audio_controller = AudioService::new(
    //     peripherals.I2S0,
    //     peripherals.DMA_CH0,
    //     peripherals.GPIO36.into(),
    //     peripherals.GPIO37.into(),
    // ).await;

    // spawner.spawn(audio_task(audio_controller)).unwrap();

    // Wifi
    if system_mode == SystemMode::Mailbox {
        let wifi_stack = wifi_init(
            spawner,
            peripherals.TIMG0,
            peripherals.RADIO_CLK,
            peripherals.WIFI,
            rng.clone(),
        )
        .await;
    }
}
