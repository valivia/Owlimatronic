#![feature(mpmc_channel)]

use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use log::info;

mod modules;
use modules::{
    animation::{AnimationPlayer, IDLE_ANIMATION},
    audio::{AudioConfig, AudioController},
    indicator::{RGB8, WS2812RMT},
    servo::ServoController,
};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    info!("Hello, world!");
    let app_config = CONFIG;

    let mut led = WS2812RMT::new(peripherals.pins.gpio21, peripherals.rmt.channel0)?;

    // Connect to wifi
    led.set_pixel(RGB8::new(0, 0, 50))?;

    // let _wifi = match wifi(
    //     app_config.wifi_ssid,
    //     app_config.wifi_psk,
    //     peripherals.modem,
    //     sysloop,
    // ) {
    //     Ok(inner) => inner,
    //     Err(err) => {
    //         // Red!
    //         led.set_pixel(RGB8::new(50, 0, 0))?;
    //         bail!("Could not connect to Wi-Fi network: {:?}", err)
    //     }
    // };

    // Audio

    let audio = AudioController::new(
        peripherals.i2s0,
        AudioConfig {
            clk: peripherals.pins.gpio36.into(),
            dout: peripherals.pins.gpio37.into(),
        },
    );

    let audio_tx = AudioController::start_audio_thread(audio);

    // Servo

    let mut servo = ServoController::new(peripherals.ledc);

    led.set_pixel(RGB8::new(0, 0, 0))?;

    loop {
        AnimationPlayer::play(&mut servo, &audio_tx, &IDLE_ANIMATION.to_vec());
        std::thread::sleep(std::time::Duration::from_secs(5));
        // info!("Setting servo to 0");
        // servo.set_angle(0, 0);
        // servo.set_angle(1, 0);
        // servo.set_angle(2, 0);
        // servo.set_angle(3, 0);
        // std::thread::sleep(std::time::Duration::from_secs(3));

        // info!("Setting servo to 90");
        // servo.set_angle(0, 90);
        // servo.set_angle(1, 90);
        // servo.set_angle(2, 90);
        // servo.set_angle(3, 90);
        // std::thread::sleep(std::time::Duration::from_millis(400));
        // audio.play(hoot);

        // info!("Setting servo to 180");
        // servo.set_angle(0, 180);
        // servo.set_angle(1, 180);
        // servo.set_angle(2, 180);
        // servo.set_angle(3, 180);
        // std::thread::sleep(std::time::Duration::from_secs(1));

        // led.set_pixel(RGB8::new(0, 50, 0))?;
        // servo.release_all_servos();
        // std::thread::sleep(std::time::Duration::from_secs(5));
        // led.set_pixel(RGB8::new(0, 0, 0))?;
    }
}
