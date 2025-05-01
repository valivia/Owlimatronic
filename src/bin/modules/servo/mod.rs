use defmt::info;
use esp_hal::{mcpwm::operator::PwmPin, peripherals::MCPWM0};
use controller::ServoController;

use crate::modules::util::map_range_clamped;

pub mod config;
pub mod controller;
pub mod animation;

#[embassy_executor::task]
pub async fn servo_task(mut controller: ServoController) {

    info!("Servo task started");

    loop {
        controller.run_loop().await;
    }
}

enum Servo<'a> {
    ServoBeak(PwmPin<'a, MCPWM0, 0, true>),
    ServoNeck(PwmPin<'a, MCPWM0, 0, false>),
    ServoWingR(PwmPin<'a, MCPWM0, 1, true>),
    ServoWingL(PwmPin<'a, MCPWM0, 1, false>),
}

impl<'a> Servo<'a> {
    pub fn set_timestamp(&mut self, value: u16) {
        match self {
            Servo::ServoBeak(pin) => pin.set_timestamp(value),
            Servo::ServoNeck(pin) => pin.set_timestamp(value),
            Servo::ServoWingR(pin) => pin.set_timestamp(value),
            Servo::ServoWingL(pin) => pin.set_timestamp(value),
        }
    }

    pub fn move_to(&mut self, value: u16) {
        let servo_config = self.get_config();
        let position = map_range_clamped(
            value as i32,
            config::SERVO_MIN as i32,
            config::SERVO_MAX as i32,
            servo_config.min_duty_cycle as i32,
            servo_config.max_duty_cycle as i32,
        );
        // info!(
        //     "Moving {} to {} ({} us)",
        //     servo_config.name, value, position
        // );
        self.set_timestamp(position as u16);
    }

    pub fn get_config(&self) -> &config::ServoConfig {
        match self {
            Servo::ServoBeak(_) => &config::SERVOS[0],
            Servo::ServoNeck(_) => &config::SERVOS[1],
            Servo::ServoWingR(_) => &config::SERVOS[2],
            Servo::ServoWingL(_) => &config::SERVOS[3],
        }
    }
}
