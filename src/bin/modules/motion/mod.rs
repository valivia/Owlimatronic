use defmt::info;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::I2C0;
use hayasen::mpu6050::{AccelRange, GyroRange};
use hayasen::mpu6050_hayasen;
use num_traits::float::FloatCore;

use crate::modules::servo::animation::ANIMATION_QUEUE;
use crate::modules::servo::animations::AnimationType;

const TAG: &str = "[MOTION]";

#[embassy_executor::task]
pub async fn motion_task(
    i2c: I2C0<'static>,
    power_pin: AnyPin<'static>,
    clock_pin: AnyPin<'static>,
    data_pin: AnyPin<'static>,
) {
    info!("{} Starting task...", TAG);
    let mut sensor_power = Output::new(power_pin, Level::High, OutputConfig::default());
    sensor_power.set_high();
    Timer::after_millis(300).await;

    let i2c = I2c::new(i2c, Config::default())
        .unwrap()
        .with_sda(data_pin)
        .with_scl(clock_pin);

    let mut sensor = match mpu6050_hayasen::create_default(i2c, 0x68) {
        Ok(s) => s,
        Err(e) => {
            match e {
                hayasen::Error::ConfigError => info!("{} Configuration error", TAG),
                hayasen::Error::I2c(_) => info!("{} I2C communication error", TAG),
                hayasen::Error::InvalidData => info!("{} Invalid data received", TAG),
                hayasen::Error::NotDetected => info!("{} Sensor not detected", TAG),
                hayasen::Error::SensorSpecific(msg) => {
                    info!("{} Sensor-specific error: {}", TAG, msg)
                }
            }
            return;
        }
    };

    sensor.setup_accelerometer(AccelRange::Range2G).unwrap();
    sensor.setup_gyroscope(GyroRange::Range250Dps).unwrap();
    sensor.set_sample_rate(16).unwrap();
    sensor.wake_up().unwrap();

    const SAMPLE_SIZE: usize = 10;
    let mut average_accel: [f32; SAMPLE_SIZE] = [0.0; SAMPLE_SIZE];
    let mut last_trigger = Instant::now();

    loop {
        Timer::after_millis(50).await;

        let (_temp, accel, _gyro) = mpu6050_hayasen::read_all(&mut sensor).unwrap();
        // accel[0]; // up down
        // accel[1]; // left right
        // accel[2]; // forward back

        //  Calculate total acceleration magnitude (subtract gravity)
        let total_accel = libm::sqrtf(accel[0].powi(2) + accel[1].powi(2) + accel[2].powi(2));
        let motion_accel = (total_accel - 1.0).abs(); // Subtract 1g gravity
        let avg_motion_accel: f32 = average_accel.iter().sum::<f32>() / SAMPLE_SIZE as f32;

        // Calculate total angular velocity
        // let total_gyro = libm::sqrtf(gyro[0].powi(2) + gyro[1].powi(2) + gyro[2].powi(2));

        // If didn't recently play animation
        if (last_trigger + Duration::from_secs(2)) < Instant::now() {
            // Picked up
            if (accel[0] - 0.96).abs() > 0.3 && avg_motion_accel < 0.1 {
                info!("{} Picked up!", TAG);
                ANIMATION_QUEUE.send(AnimationType::PickedUp).await;
                last_trigger = Instant::now();
            }
        }

        // Update moving average
        for i in (1..SAMPLE_SIZE).rev() {
            average_accel[i] = average_accel[i - 1];
        }
        average_accel[0] = motion_accel;

        // // Detect motion
        // if motion_accel > ACCEL_THRESHOLD || total_gyro > GYRO_THRESHOLD {
        //     info!(
        //         "{} Motion detected! Accel: {}g, Gyro: {}°/s",
        //         TAG, motion_accel, total_gyro
        //     );

        //     if embassy_time::Instant::now() - last_trigger_time
        //         > embassy_time::Duration::from_secs(5)
        //     {
        //         ANIMATION_QUEUE.send(AnimationType::Flap).await;
        //         last_trigger_time = embassy_time::Instant::now();
        //     }
        // }
    }
}
