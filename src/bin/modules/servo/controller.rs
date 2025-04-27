use defmt::info;
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::AnyPin,
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    peripherals::MCPWM0,
    time::Rate,
};

use crate::modules::{
    animation::ANIMATION_QUEUE,
    servo::config::{SERVO_MAX, SERVO_MIN},
    util::map_range_clamped,
};

use super::{config::SERVO_COUNT, Servo};

static TEST_ANIMATION: [[u16; 140]; SERVO_COUNT] = [
    [
        0, 1, 4, 14, 32, 63, 108, 172, 256, 364, 500, 636, 744, 829, 892, 938, 968, 987, 996, 1000,
        1000, 1000, 996, 987, 968, 938, 892, 829, 744, 636, 500, 365, 256, 172, 108, 63, 32, 13, 4,
        1, 0, 0, 0, 0, 1, 1, 2, 3, 4, 6, 8, 10, 14, 17, 21, 26, 32, 38, 46, 54, 63, 72, 83, 95,
        108, 122, 137, 154, 172, 191, 211, 233, 256, 281, 307, 335, 364, 396, 429, 463, 500, 537,
        571, 604, 636, 665, 693, 719, 744, 767, 789, 809, 829, 846, 863, 878, 892, 905, 917, 928,
        938, 946, 954, 962, 968, 974, 979, 983, 987, 990, 992, 994, 996, 997, 998, 999, 1000, 1000,
        1000, 1000, 1000, 1000, 996, 987, 968, 938, 892, 829, 744, 636, 500, 365, 256, 172, 108,
        63, 32, 13, 4, 1,
    ],
    [
        477, 477, 476, 475, 471, 466, 458, 447, 432, 412, 389, 365, 345, 330, 319, 311, 306, 302,
        301, 300, 300, 300, 300, 301, 302, 303, 305, 309, 313, 318, 325, 333, 343, 355, 369, 384,
        402, 423, 446, 471, 500, 529, 554, 577, 598, 616, 631, 645, 657, 667, 675, 682, 687, 691,
        695, 697, 698, 699, 700, 700, 700, 700, 700, 700, 700, 700, 699, 699, 698, 697, 696, 695,
        694, 692, 690, 688, 685, 682, 678, 675, 670, 666, 661, 655, 649, 642, 635, 627, 619, 610,
        600, 590, 581, 573, 565, 558, 551, 545, 539, 534, 530, 525, 522, 518, 515, 513, 510, 508,
        506, 505, 504, 503, 502, 501, 501, 500, 500, 500, 500, 500, 500, 500, 500, 500, 499, 499,
        498, 496, 494, 492, 489, 485, 483, 481, 479, 478, 478, 477, 477, 477,
    ],
    [
        0, 0, 1, 2, 4, 8, 14, 21, 32, 46, 63, 83, 108, 137, 172, 211, 256, 307, 364, 429, 500, 571,
        636, 693, 744, 789, 829, 863, 892, 917, 938, 954, 968, 979, 987, 992, 996, 998, 1000, 1000,
        1000, 1000, 1000, 1000, 1000, 999, 998, 997, 996, 994, 992, 990, 987, 983, 979, 974, 968,
        962, 954, 946, 938, 928, 917, 905, 892, 878, 863, 846, 829, 809, 789, 767, 744, 719, 693,
        665, 636, 604, 571, 537, 500, 463, 429, 396, 365, 335, 307, 281, 256, 233, 211, 191, 172,
        154, 137, 122, 108, 95, 83, 72, 63, 54, 46, 38, 32, 26, 21, 17, 13, 10, 8, 6, 4, 3, 2, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 1, 2, 4, 8, 14, 21, 32, 46, 63, 83, 108, 137, 172, 211, 256, 307, 364, 429, 500, 571,
        636, 693, 744, 789, 829, 863, 892, 917, 938, 954, 968, 979, 987, 992, 996, 998, 1000, 1000,
        1000, 1000, 1000, 1000, 1000, 999, 998, 997, 996, 994, 992, 990, 987, 983, 979, 974, 968,
        962, 954, 946, 938, 928, 917, 905, 892, 878, 863, 846, 829, 809, 789, 767, 744, 719, 693,
        665, 636, 604, 571, 537, 500, 463, 429, 396, 365, 335, 307, 281, 256, 233, 211, 191, 172,
        154, 137, 122, 108, 95, 83, 72, 63, 54, 46, 38, 32, 26, 21, 17, 13, 10, 8, 6, 4, 3, 2, 1,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

const KEYFRAME_DURATION: u64 = 250;
const INTERPOLATION_STEPS: u64 = 20;

const FRAME_DURATION: Duration = Duration::from_millis(KEYFRAME_DURATION / INTERPOLATION_STEPS);
pub struct ServoController<'a> {
    servos: [Servo<'a>; SERVO_COUNT],
}

impl<'a> ServoController<'a> {
    pub async fn new(
        mc_pwm: MCPWM0,
        beak_pin: AnyPin,
        neck_pin: AnyPin,
        wing_r_pin: AnyPin,
        wing_l_pin: AnyPin,
    ) -> Self {
        let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(32)).unwrap();
        let mut mcpwm = McPwm::new(mc_pwm, clock_cfg);

        // Connect operator0 to timer0
        mcpwm.operator0.set_timer(&mcpwm.timer0);

        let op0 = mcpwm.operator0;
        let op1 = mcpwm.operator1;

        // Set up the pins for the servos
        let (beak, neck) = op0.with_pins(
            beak_pin,
            PwmPinConfig::UP_ACTIVE_HIGH,
            neck_pin,
            PwmPinConfig::UP_ACTIVE_HIGH,
        );

        let (wing_r, wing_l) = op1.with_pins(
            wing_r_pin,
            PwmPinConfig::UP_ACTIVE_HIGH,
            wing_l_pin,
            PwmPinConfig::UP_ACTIVE_HIGH,
        );

        // Put the servos in an array
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

        let mut controller = ServoController { servos };

        // Set default positions
        controller.reset_servos();

        Timer::after(Duration::from_millis(500)).await;

        // Release servos
        controller.release_servos();

        controller
    }

    pub async fn run_loop(&mut self) {
        ANIMATION_QUEUE.receive().await;
        self.run_animation(TEST_ANIMATION).await;
    }

    fn reset_servos(&mut self) {
        info!("Resetting servos to default positions");
        for servo in &mut self.servos {
            let servo_config = servo.get_config();
            let position = map_range_clamped(
                servo_config.default_position as i32,
                SERVO_MIN as i32,
                SERVO_MAX as i32,
                servo_config.min as i32,
                servo_config.max as i32,
            );
            servo.set_timestamp(position as u16);
        }
    }

    fn release_servos(&mut self) {
        info!("Releasing servos");
        for servo in &mut self.servos {
            servo.set_timestamp(0);
        }
    }

    async fn run_animation(&mut self, animation: [[u16; 140]; SERVO_COUNT]) {
        info!("Running animation {}", animation);
        let total_frames = TEST_ANIMATION[0].len();

        for frame_index in 0..total_frames {
            info!("Interpolated Frame {}/{}", frame_index + 1, total_frames);

            // Set servo angles
            for servo_index in 0..SERVO_COUNT {
                let servo_track = &animation[servo_index];
                let target = servo_track[frame_index as usize];
                self.servos[servo_index].move_to(target);
            }

            Timer::after(FRAME_DURATION).await;
        }

        self.release_servos();
    }
}
