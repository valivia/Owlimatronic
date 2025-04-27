pub static MIN_DUTY: u32 = 500;
pub static MAX_DUTY: u32 = 2400;

pub static SERVO_MIN: u32 = 0;
pub static SERVO_MAX: u32 = 1000;

pub struct ServoConfig {
    pub name: &'static str,

    pub min: u32,
    pub max: u32,

    pub default_position: u16,
}

pub const SERVO_COUNT: usize = 4;
pub const SERVOS: [ServoConfig; SERVO_COUNT] = [
    ServoConfig {
        name: "Beak",
        min: 922,
        max: 584,
        default_position: 0,
    },
    ServoConfig {
        name: "Neck",
        min: 500,
        max: 2_400,
        default_position: 477,
    },
    ServoConfig {
        name: "Wing_R",
        min: 1_935,
        max: 1_344,
        default_position: 0,
    },
    ServoConfig {
        name: "Wing_L",
        min: 922,
        max: 1_471,
        default_position: 0,
    },
];