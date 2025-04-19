#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use modules::servo::ServoController;

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

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    // TODO: Spawn some tasks
    let _ = spawner;

    let mut servo_controller = ServoController::new();

    loop {
        info!("spinning to 0");

        servo_controller.move_servo(0, 0);
        servo_controller.move_servo(1, 0);
        servo_controller.move_servo(2, 0);
        servo_controller.move_servo(3, 0);

        Timer::after(Duration::from_secs(2)).await;
        info!("spinning to 90");

        servo_controller.move_servo(0, 90);
        servo_controller.move_servo(1, 90);
        servo_controller.move_servo(2, 90);
        servo_controller.move_servo(3, 90);

        Timer::after(Duration::from_secs(2)).await;
        info!("spinning to 180");

        servo_controller.move_servo(0, 180);
        servo_controller.move_servo(1, 180);
        servo_controller.move_servo(2, 180);
        servo_controller.move_servo(3, 180);

        Timer::after(Duration::from_secs(2)).await;
    }
}
