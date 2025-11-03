use defmt::{info, warn};
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::OutputPin,
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    peripherals::MCPWM0,
    time::Rate,
};

use crate::modules::{
    audio::AUDIO_QUEUE,
    servo::{
        animation::{FRAME_DURATION, INTERPOLATION_STEPS},
        config::{SERVO_MAX, SERVO_MIN},
    },
    util::map_range_clamped,
};

use super::{
    animation::{Animation, ANIMATION_QUEUE},
    config::SERVO_COUNT,
    easing::Easing,
    Servo,
};

pub struct ServoController {
    servos: [Servo<'static>; SERVO_COUNT],
}

use num_traits::float::FloatCore;

const TAG: &str = "[SERVO]";

impl ServoController {
    pub async fn new(
        mc_pwm: MCPWM0<'static>,
        beak_pin: impl OutputPin + 'static,
        neck_pin: impl OutputPin + 'static,
        wing_r_pin: impl OutputPin + 'static,
        wing_l_pin: impl OutputPin + 'static,
    ) -> Self {
        let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(32)).unwrap();
        let mut mcpwm = McPwm::new(mc_pwm, clock_cfg);

        // Connect operator0 to timer0
        mcpwm.operator0.set_timer(&mcpwm.timer0);
        mcpwm.operator1.set_timer(&mcpwm.timer0);

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
        let animation = ANIMATION_QUEUE.receive().await;
        self.run_animation(animation.get_animation()).await;
    }

    // Control
    fn reset_servos(&mut self) {
        info!("{} Resetting servos to default positions", TAG);
        for servo in &mut self.servos {
            let servo_config = servo.get_config();
            let position = map_range_clamped(
                servo_config.default_position as i32,
                SERVO_MIN as i32,
                SERVO_MAX as i32,
                servo_config.min_duty_cycle as i32,
                servo_config.max_duty_cycle as i32,
            );
            servo.set_timestamp(position as u16);
        }
    }

    fn release_servos(&mut self) {
        for servo in &mut self.servos {
            servo.set_timestamp(0);
        }
    }

    // Animation
    pub async fn run_animation(&mut self, animation: &Animation) {
        info!("{} Running animation with {} frames", TAG, animation.len());
        let total_frames = animation.len();

        let mut previous_servo_frame_index: [Option<usize>; SERVO_COUNT] = [None; SERVO_COUNT];
        let mut next_servo_frame_index: [Option<usize>; SERVO_COUNT] = [None; SERVO_COUNT];

        for frame_index in 0..animation.len() {
            // Play audio if present
            if let Some(frame) = &animation[frame_index] {
                if let Some(audio) = &frame.audio {
                    AUDIO_QUEUE.signal(audio.clone());
                }
            }

            // Get the frames to interpolate from
            ServoController::get_surrounding_frame_indices(
                &mut next_servo_frame_index,
                &mut previous_servo_frame_index,
                animation,
                frame_index,
            );

            if frame_index == total_frames - 1 {
                for servo_index in 0..SERVO_COUNT {
                    if let Some(target) = animation[frame_index]
                        .as_ref()
                        .unwrap()
                        .get_servo(servo_index)
                    {
                        self.servos[servo_index].set_timestamp(target.0);
                    }
                }
                break;
            }

            // Play the servo frames
            for interpolation_index in 0..INTERPOLATION_STEPS {
                // Set servo angles
                for servo_index in 0..SERVO_COUNT {
                    let from = match previous_servo_frame_index[servo_index] {
                        Some(index) => Some((
                            animation[index]
                                .as_ref()
                                .unwrap()
                                .get_servo(servo_index)
                                .unwrap()
                                .0,
                            index,
                        )),
                        None => None,
                    };

                    let to = match next_servo_frame_index[servo_index] {
                        Some(index) => {
                            let frame = animation[index]
                                .as_ref()
                                .unwrap()
                                .get_servo(servo_index)
                                .unwrap();
                            Some((frame.0, frame.1, index))
                        }
                        None => None,
                    };

                    if let (Some(from), Some(to)) = (from, to) {
                        let (from_value, from_index) = from;
                        let (to_value, easing, to_index) = to;

                        // No need to interpolate if the values are the same
                        if to_value == from_value {
                            continue;
                        }

                        let frame_span = to_index - from_index;
                        let total_keyframe_steps = INTERPOLATION_STEPS * frame_span as u32;
                        let current_frame_in_span = (frame_index - from_index) as u32;
                        let current_step =
                            current_frame_in_span * INTERPOLATION_STEPS + interpolation_index + 1;

                        let t = current_step as f32 / total_keyframe_steps as f32;
                        let target = Self::interpolate(from_value, to_value, t, &easing);

                        if (t > 1.0) || (t < 0.0) {
                            warn!("{} Interpolation out of bounds: t = {}", TAG, t);
                        }

                        self.servos[servo_index].move_to(target);
                    }
                }

                Timer::after(FRAME_DURATION).await;
            }
        }

        info!("{} Animation completed!", TAG);
        self.release_servos();
    }

    fn get_surrounding_frame_indices(
        next_servo_frame_index: &mut [Option<usize>; SERVO_COUNT],
        previous_servo_frame_index: &mut [Option<usize>; SERVO_COUNT],
        animation: &Animation,
        frame_index: usize,
    ) {
        for servo_index in 0..SERVO_COUNT {
            // Previous servo frame
            let mut should_fetch_previous = false;

            // if we exceeded the last known previous index, we need to fetch the previous one
            if let Some(last_known_previous_index) = previous_servo_frame_index[servo_index] {
                if last_known_previous_index < frame_index && frame_index != animation.len() - 1 {
                    should_fetch_previous = true;
                }
            }

            // if we are at the first frame, we need to fetch previous frame.
            if frame_index == 0 {
                should_fetch_previous = true;
            }

            if should_fetch_previous {
                previous_servo_frame_index[servo_index] =
                    ServoController::get_closest_servo_keyframe_index(
                        animation,
                        frame_index,
                        servo_index,
                        false,
                    );
            }

            // Next servo frame
            let mut should_fetch_next = false;

            // If we hit the last known next index, we need to fetch the next one
            if let Some(last_known_next_index) = next_servo_frame_index[servo_index] {
                if last_known_next_index == frame_index
                    && last_known_next_index != animation.len() - 1
                {
                    should_fetch_next = true;
                }
            }

            // if we are at the first frame, we need to fetch next frame.
            if frame_index == 0 {
                should_fetch_next = true;
            }

            // Get the next servo frame
            if should_fetch_next {
                next_servo_frame_index[servo_index] =
                    ServoController::get_closest_servo_keyframe_index(
                        animation,
                        frame_index + 1,
                        servo_index,
                        true,
                    );
            }
        }
    }

    fn get_closest_servo_keyframe_index(
        animation: &Animation,
        start_index: usize,
        servo_index: usize,
        forward: bool,
    ) -> Option<usize> {
        let len = animation.len();
        let mut index = start_index;

        loop {
            if index >= len {
                break;
            }

            if let Some(frame) = &animation[index] {
                let keyframe = match servo_index {
                    0 => frame.beak_servo,
                    1 => frame.neck_servo,
                    2 => frame.wing_right_servo,
                    3 => frame.wing_left_servo,
                    _ => None,
                };

                if keyframe.is_some() {
                    return Some(index);
                }
            }

            if forward {
                index += 1;
                if index >= len {
                    break;
                }
            } else {
                if index == 0 {
                    break;
                }
                index -= 1;
            }
        }

        None
    }

    fn interpolate(from: u16, to: u16, t: f32, easing: &Easing) -> u16 {
        let eased_t = easing.ease(t);
        let delta = to as f32 - from as f32;
        let interpolated_value = from as f32 + (delta * eased_t);
        interpolated_value.round() as u16
    }
}
