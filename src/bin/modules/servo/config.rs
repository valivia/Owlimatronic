pub static SERVO_MIN: u32 = 0;
pub static SERVO_MAX: u32 = 1000;

pub struct ServoConfig {
    pub name: &'static str,

    pub min_duty_cycle: u32,
    pub max_duty_cycle: u32,

    pub default_position: u16,
}

pub const SERVO_COUNT: usize = 4;
pub const DEFAULT_NECK_POSITION: u16 = 477;
pub const DEFAULT_WING_POSITION: u16 = 0;
pub const DEFAULT_BEAK_POSITION: u16 = 0;

pub const SERVOS: [ServoConfig; SERVO_COUNT] = [
    ServoConfig {
        name: "Beak",
        min_duty_cycle: 922,
        max_duty_cycle: 584,
        default_position: DEFAULT_BEAK_POSITION,
    },
    ServoConfig {
        name: "Neck",
        min_duty_cycle: 500,
        max_duty_cycle: 2_400,
        default_position: DEFAULT_NECK_POSITION,
    },
    ServoConfig {
        name: "Wing_R",
        min_duty_cycle: 1_935,
        max_duty_cycle: 1_344,
        default_position: DEFAULT_WING_POSITION,
    },
    ServoConfig {
        name: "Wing_L",
        min_duty_cycle: 922,
        max_duty_cycle: 1_471,
        default_position: DEFAULT_WING_POSITION,
    },
];
