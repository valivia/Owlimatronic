use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::ledc::{self};
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::prelude::*;
use log::info;

struct ServoConfig {
    pub pin: i32,
    pub name: &'static str,

    pub min: u16,
    pub max: u16,

    pub default_position: u16,
}

const MIN_DUTY: u16 = 500;
const MAX_DUTY: u16 = 2400;

const SERVO_COUNT: usize = 4;
const SERVOS: [ServoConfig; SERVO_COUNT] = [
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
        let mut servos = [
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

        // Set the default position for each servo
        for (i, servo) in servos.iter_mut().enumerate() {
            let config = &SERVOS[i];
            let pos = ServoController::get_calibrated_angle(i, config.default_position);
            let pulse_width = ServoController::map(pos, 0, 180, MIN_DUTY, MAX_DUTY);
            servo.set_duty(pulse_width.into()).unwrap();
        }

        Self { servos }
    }

    pub fn set_angle(&mut self, servo_index: usize, target_angle: u16) {
        if servo_index >= SERVO_COUNT {
            panic!("Servo index out of bounds");
        }

        let servo = &mut self.servos[servo_index];
        let config = &SERVOS[servo_index];

        let pos = ServoController::get_calibrated_angle(servo_index, target_angle);
        let pulse_width = ServoController::map(pos, 0, 180, MIN_DUTY, MAX_DUTY);

        info!(
            "Moving {} ({}) to {} ({} us)",
            config.name, config.pin, pos, pulse_width
        );

        servo
            .set_duty(ServoController::micros_to_ticks(pulse_width).into())
            .unwrap();
    }

    pub fn release_servo(&mut self, servo_index: usize) {
        if servo_index >= SERVO_COUNT {
            panic!("Servo index out of bounds");
        }

        let servo = &mut self.servos[servo_index];

        servo.set_duty(0).unwrap();
    }

    fn get_calibrated_angle(servo_index: usize, target_angle: u16) -> u16 {
        if servo_index >= SERVO_COUNT {
            panic!("Servo index out of bounds");
        }

        let config = &SERVOS[servo_index];
        let pos = ServoController::map(
            target_angle,
            0 as u16,
            180 as u16,
            config.min.into(),
            config.max.into(),
        );

        return pos;
    }

    fn micros_to_ticks(us: u16) -> u16 {
        ((us as u32 * 16384) / 20_000) as u16
    }

    fn map(x: u16, in_min: u16, in_max: u16, out_min: u16, out_max: u16) -> u16 {
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
