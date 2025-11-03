#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use esp_alloc::HeapStats;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use modules::audio::audio_task;
use modules::connectivity::wifi::wifi_init;
use modules::indicator::indicator_task;
use modules::interaction::interaction_task;
use modules::mode::{SystemMode, initialize_mode};
use modules::servo::controller::ServoController;
use modules::servo::servo_task;
use ringbuf::{StaticRb, traits::*};
use static_cell::StaticCell;

use crate::modules::connectivity::mqtt::mqtt_init;
use crate::modules::connectivity::streamer::{
    AudioChunk, STREAM_SIZE, StreamRingBuffer, streamer_init,
};
use crate::modules::motion::motion_task;
use crate::modules::servo::animation::ANIMATION_QUEUE;
use crate::modules::servo::animations::AnimationType;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

mod modules;

static STREAM_RING_BUFFER: StaticCell<StreamRingBuffer> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    // Indicator
    spawner
        .spawn(indicator_task(peripherals.RMT, peripherals.GPIO21.into()))
        .unwrap();

    // Mode
    let system_mode = initialize_mode(spawner, peripherals.GPIO8, peripherals.GPIO9).await;

    // Servos
    let servo_controller = ServoController::new(
        peripherals.MCPWM0,
        peripherals.GPIO16,
        peripherals.GPIO15,
        peripherals.GPIO14,
        peripherals.GPIO13,
    )
    .await;

    spawner.spawn(servo_task(servo_controller)).unwrap();

    // Interaction
    let task = interaction_task(peripherals.GPIO6.into());
    spawner.spawn(task).unwrap();

    let stream_ring_buffer =
        STREAM_RING_BUFFER.init(StaticRb::<AudioChunk, STREAM_SIZE>::default());
    let (stream_producer, stream_consumer) = stream_ring_buffer.split_ref();

    // Audio
    let task = audio_task(
        peripherals.I2S0,
        peripherals.DMA_CH0,
        peripherals.GPIO36.into(),
        peripherals.GPIO37.into(),
        peripherals.GPIO35.into(),
        stream_consumer,
    );

    spawner.spawn(task).unwrap();

    match system_mode {
        SystemMode::Mailbox => {
            // Wifi
            let wifi_stack =
                wifi_init(spawner, peripherals.WIFI).await;

            // MQTT
            spawner.spawn(mqtt_init(wifi_stack.clone())).ok();
            spawner
                .spawn(streamer_init(wifi_stack, stream_producer))
                .ok();
        }
        SystemMode::Play => {
            ANIMATION_QUEUE.send(AnimationType::Yap).await;
            // Accelerometer / Gyroscope
            let task = motion_task(
                peripherals.I2C0,
                peripherals.GPIO41.into(),
                peripherals.GPIO40.into(),
                peripherals.GPIO39.into(),
            );

            spawner.spawn(task).unwrap();
        }
        SystemMode::Off => (),
    }

    let stats: HeapStats = esp_alloc::HEAP.stats();
    info!("{}", stats);
}