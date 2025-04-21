use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::ledc::{self};
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::prelude::*;
use log::{debug, info};

pub struct ServoConfig {
    pub pin: i32,
    pub name: &'static str,

    pub min: u16,
    pub max: u16,

    pub default_position: u16,
}

const MIN_DUTY: u16 = 500;
const MAX_DUTY: u16 = 2400;

const SERVO_MIN: u16 = 0;
const SERVO_MAX: u16 = 1000;

pub const SERVO_COUNT: usize = 4;
pub const SERVOS: [ServoConfig; SERVO_COUNT] = [
    ServoConfig {
        pin: 16,
        name: "Beak",
        min: 40,
        max: 8,
        default_position: 0,
    },
    ServoConfig {
        pin: 15,
        name: "Neck",
        min: 0,
        max: 180,
        default_position: 86,
    },
    ServoConfig {
        pin: 14,
        name: "Wing_R",
        min: 136,
        max: 80,
        default_position: 0,
    },
    ServoConfig {
        pin: 13,
        name: "Wing_L",
        min: 40,
        max: 92,
        default_position: 0,
    },
];

pub struct ServoController<'a> {
    servos: [LedcDriver<'a>; SERVO_COUNT],
}

impl<'a> ServoController<'a> {
    pub fn new(ledc: ledc::LEDC) -> Self {
        info!("Initializing servo controller");

        // Get the peripherals
        let timer_driver = LedcTimerDriver::new(
            ledc.timer0,
            &TimerConfig::default()
                .frequency(50.Hz().into())
                .resolution(ledc::Resolution::Bits14),
        )
        .unwrap();

        info!("Registering {} servos", SERVO_COUNT);

        // Create the LEDC drivers for each servo
        let servos = [
            LedcDriver::new(ledc.channel0, &timer_driver, unsafe {
                AnyOutputPin::new(SERVOS[0].pin)
            })
            .unwrap(),
            LedcDriver::new(ledc.channel1, &timer_driver, unsafe {
                AnyOutputPin::new(SERVOS[1].pin)
            })
            .unwrap(),
            LedcDriver::new(ledc.channel2, &timer_driver, unsafe {
                AnyOutputPin::new(SERVOS[2].pin)
            })
            .unwrap(),
            LedcDriver::new(ledc.channel3, &timer_driver, unsafe {
                AnyOutputPin::new(SERVOS[3].pin)
            })
            .unwrap(),
        ];

        info!("Setting up {} servos", SERVO_COUNT);

        let mut servo_controller = Self { servos };

        servo_controller.reset_all_servos();

        return servo_controller;
    }

    // Move

    pub fn set_angle(&mut self, servo_index: usize, target_angle: u16) {
        if servo_index >= SERVO_COUNT {
            panic!("Servo index out of bounds");
        }

        let servo = &mut self.servos[servo_index];
        let config = &SERVOS[servo_index];

        let pos = ServoController::get_calibrated_angle(config, target_angle);
        let pulse_width = ServoController::map(pos, 0, 180, MIN_DUTY, MAX_DUTY);

        debug!(
            "Moving {} ({}) to {} ({} us)",
            config.name, config.pin, pos, pulse_width
        );

        servo
            .set_duty(ServoController::micros_to_ticks(pulse_width).into())
            .unwrap();
    }

    // Release

    pub fn reset_all_servos(&mut self) {
        for (i, servo) in self.servos.iter_mut().enumerate() {
            let config = &SERVOS[i];
            ServoController::reset_to_base_position(servo, config);
        }
    }

    fn reset_to_base_position(servo: &mut LedcDriver<'a>, config: &ServoConfig) {
        let pos = ServoController::get_calibrated_angle(config, config.default_position);
        let pulse_width = ServoController::map(pos, 0, 180, MIN_DUTY, MAX_DUTY);
        servo.set_duty(pulse_width.into()).unwrap();
    }

    // Release

    pub fn release_servo(servo: &mut LedcDriver<'a>) {
        servo.set_duty(0).unwrap();
    }

    pub fn release_all_servos(&mut self) {
        for (_i, servo) in self.servos.iter_mut().enumerate() {
            ServoController::release_servo(servo);
        }
    }

    // Util

    fn get_calibrated_angle(config: &ServoConfig, target_angle: u16) -> u16 {
        let pos = ServoController::map(
            target_angle.clamp(SERVO_MIN, SERVO_MAX),
            SERVO_MIN,
            SERVO_MAX,
            config.min.into(),
            config.max.into(),
        );

        return pos;
    }

    fn micros_to_ticks(us: u16) -> u16 {
        ((us as u32 * 16384) / 20_000) as u16
    }

    pub fn map(x: u16, in_min: u16, in_max: u16, out_min: u16, out_max: u16) -> u16 {
        let x = x as i32;
        let in_min = in_min as i32;
        let in_max = in_max as i32;
        let out_min = out_min as i32;
        let out_max = out_max as i32;

        if in_min == in_max {
            return out_min as u16; // Avoid division by zero
        }

        let result = (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
        result.clamp(u16::MIN as i32, u16::MAX as i32) as u16
    }
}
