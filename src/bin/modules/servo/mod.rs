use defmt::info;
use esp_hal::{
    gpio::AnyPin,
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    peripherals::MCPWM0,
    time::Rate,
};

use crate::modules::util::map;

pub struct ServoConfig {
    pub pin: u8,
    pub name: &'static str,

    pub min: u16,
    pub max: u16,

    pub default_position: u16,
}

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

// impl ServoConfig {
//     pub fn get_servo<'a>(
//         &self,
//         mcpwm: &'a McPwm<MCPWM0>,
//         peripherals: &'a Peripherals,
//     ) -> Servo<'a> {
//         match self.pin {
//             16 => Servo::ServoBeak(
//                 mcpwm
//                     .operator0
//                     .with_pin_a(peripherals.GPIO16, PwmPinConfig::UP_ACTIVE_HIGH),
//             ),
//             15 => Servo::ServoNeck(
//                 mcpwm
//                     .operator0
//                     .with_pin_b(peripherals.GPIO15, PwmPinConfig::UP_ACTIVE_HIGH),
//             ),
//             14 => Servo::ServoWingR(
//                 mcpwm
//                     .operator1
//                     .with_pin_a(peripherals.GPIO14, PwmPinConfig::UP_ACTIVE_HIGH),
//             ),
//             13 => Servo::ServoWingL(
//                 mcpwm
//                     .operator1
//                     .with_pin_b(peripherals.GPIO13, PwmPinConfig::UP_ACTIVE_HIGH),
//             ),
//             _ => panic!("Invalid servo pin"),
//         }
//     }
// }

enum Servo<'a> {
    ServoBeak(esp_hal::mcpwm::operator::PwmPin<'a, MCPWM0, 0, true>),
    ServoNeck(esp_hal::mcpwm::operator::PwmPin<'a, MCPWM0, 0, false>),
    ServoWingR(esp_hal::mcpwm::operator::PwmPin<'a, MCPWM0, 1, true>),
    ServoWingL(esp_hal::mcpwm::operator::PwmPin<'a, MCPWM0, 1, false>),
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
}

pub struct ServoController<'a> {
    servos: [Servo<'a>; SERVO_COUNT],
}

impl<'a> ServoController<'a> {
    pub fn new() -> Self {
        let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(32)).unwrap();
        let mut mcpwm = McPwm::new(unsafe { MCPWM0::steal() }, clock_cfg);

        // Connect operator0 to timer0
        mcpwm.operator0.set_timer(&mcpwm.timer0);

        let op0 = mcpwm.operator0;
        let op1 = mcpwm.operator1;

        // Destrcuture mcpwm
        let (mut beak,mut neck) = op0.with_pins(
            unsafe { AnyPin::steal(SERVOS[0].pin) },
            PwmPinConfig::UP_ACTIVE_HIGH,
            unsafe { AnyPin::steal(SERVOS[1].pin) },
            PwmPinConfig::UP_ACTIVE_HIGH,
        );

        let (mut wing_r,mut wing_l) = op1.with_pins(
            unsafe { AnyPin::steal(SERVOS[2].pin) },
            PwmPinConfig::UP_ACTIVE_HIGH,
            unsafe { AnyPin::steal(SERVOS[3].pin) },
            PwmPinConfig::UP_ACTIVE_HIGH,
        );

        // Set default positions
        beak.set_timestamp(ServoController::angle_to_pulse_width(SERVOS[0].default_position));
        neck.set_timestamp(ServoController::angle_to_pulse_width(SERVOS[1].default_position));
        wing_r.set_timestamp(ServoController::angle_to_pulse_width(SERVOS[2].default_position));
        wing_l.set_timestamp(ServoController::angle_to_pulse_width(SERVOS[3].default_position));

        let servos = [
            Servo::ServoBeak(beak),
            Servo::ServoNeck(neck),
            Servo::ServoWingR(wing_r),
            Servo::ServoWingL(wing_l),
        ];

        // Start timer with timestamp values in the range of 0..=19999 and a frequency of 50 Hz
        let timer_clock_cfg = clock_cfg
            .timer_clock_with_frequency(19_999, PwmWorkingMode::Increase, Rate::from_hz(50))
            .unwrap();

        mcpwm.timer0.start(timer_clock_cfg);

        ServoController { servos }
    }

    pub fn move_servo(&mut self, servo_index: u8, target_position: u16) {
        if servo_index >= SERVO_COUNT as u8 {
            panic!("Invalid servo index");
        }

        let servo_config = &SERVOS[servo_index as usize];

        // Clamp the position
        let pos = map(
            target_position,
            0 as u16,
            180 as u16,
            servo_config.min.into(),
            servo_config.max.into(),
        );

        // Map position (0–180) to pulse width range (typically 1000us–2000us)
        let pulse_width = map(pos, 0, 180, 500, 2400);

        let servo = &mut self.servos[servo_index as usize];

        info!("Moving {} to {} ({} us)", servo_config.name, pos, pulse_width);

        servo.set_timestamp(pulse_width);
    }

    fn angle_to_pulse_width(angle: u16) -> u16 {
        // Map the angle to pulse width (in microseconds)
        let pulse_width = map(angle as u16, 0, 180, 500, 2400);
        pulse_width as u16
    }
}
