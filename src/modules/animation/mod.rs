use std::{ops::Index, sync::mpmc::Sender, time::Duration};

use easing::Easing;
use log::info;

use crate::modules::servo::{SERVOS, SERVO_COUNT};

use super::{audio::files::AudioFiles, servo::ServoController};

pub mod easing;

type ServoKeyframe = (u16, Easing);
type AudioKeyframe = AudioFiles;

#[derive(Clone)]
pub struct Frame {
    beak_servo: Option<ServoKeyframe>,
    neck_servo: Option<ServoKeyframe>,
    wing_right_servo: Option<ServoKeyframe>,
    wing_left_servo: Option<ServoKeyframe>,
    audio: Option<AudioKeyframe>,
}

pub static IDLE_ANIMATION: &[Option<Frame>] = &[
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: Some((300, Easing::CubicInOut)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: Some(AudioFiles::Hoot),
    }),
    Some(Frame {
        beak_servo: Some((0, Easing::CubicInOut)),
        neck_servo: None,
        wing_right_servo: Some((1000, Easing::CubicInOut)),
        wing_left_servo: Some((1000, Easing::CubicInOut)),
        audio: None,
    }),
    Some(Frame {
        beak_servo: None,
        neck_servo: Some((700, Easing::CubicInOut)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: None,
    }),
    None,
    None,
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: Some((500, Easing::CubicInOut)),
        wing_right_servo: Some((0, Easing::CubicInOut)),
        wing_left_servo: Some((0, Easing::CubicInOut)),
        audio: None,
    }),
];

const KEYFRAME_DURATION: u64 = 250;
const INTERPOLATION_STEPS: u64 = 20;
const FRAME_DURATION: Duration = Duration::from_millis(KEYFRAME_DURATION / INTERPOLATION_STEPS);

type ServoTrack = Option<Vec<Option<u16>>>;

pub struct AnimationPlayer {}

impl AnimationPlayer {
    pub fn play(
        servo_controller: &mut ServoController,
        audio_player: &Sender<AudioFiles>,
        frames: &Vec<Option<Frame>>,
    ) {
        let end_point_frame = Some(Frame {
            beak_servo: Some((SERVOS[0].default_position, Easing::CubicInOut)),
            neck_servo: Some((
                ServoController::map(SERVOS[1].default_position, 0, 180, 0, 1000),
                Easing::CubicInOut,
            )),
            wing_right_servo: Some((SERVOS[2].default_position, Easing::CubicInOut)),
            wing_left_servo: Some((SERVOS[3].default_position, Easing::CubicInOut)),
            audio: None,
        });

        let mut frames = frames.clone();
        frames.insert(0, end_point_frame.clone());
        frames.push(end_point_frame);

        let smoothed_frames =
            Self::generate_interpolated_servo_frames(&frames, INTERPOLATION_STEPS as usize); // 3 = number of in-betweens

        let total_frames = ((frames.len() - 1) * INTERPOLATION_STEPS as usize) as u32;

        for frame_index in 0..total_frames {
            info!("Interpolated Frame {}/{}", frame_index + 1, total_frames);

            // Set servo angles
            for servo_index in 0..SERVO_COUNT {
                if let Some(servo_track) = &smoothed_frames[servo_index] {
                    let target = servo_track.index(frame_index as usize).clone();
                    if let Some(target) = target {
                        servo_controller.set_angle(servo_index, target);
                    }
                }
            }

            // Play audio if present
            if (frame_index + 1) % INTERPOLATION_STEPS as u32 == 0 {
                let real_frame_index = (frame_index / INTERPOLATION_STEPS as u32) as usize;
                if let Some(frame) = &frames[real_frame_index] {
                    if let Some(audio) = &frame.audio {
                        audio_player.send(audio.clone()).unwrap();
                    }
                }
            }

            std::thread::sleep(FRAME_DURATION);
        }

        info!("Animation completed!");
        std::thread::sleep(Duration::from_millis(500));
        servo_controller.release_all_servos();
    }

    fn generate_interpolated_servo_frames(
        frames: &Vec<Option<Frame>>,
        interpolation_steps: usize,
    ) -> [ServoTrack; 4] {
        let mut result: [ServoTrack; 4] = [
            Some(Vec::new()),
            Some(Vec::new()),
            Some(Vec::new()),
            Some(Vec::new()),
        ];

        let mut last_known_index: [usize; 4] = [0, 0, 0, 0];

        for frame_index in 0..frames.len() - 1 {
            for servo_index in 0..SERVO_COUNT {
                // Get current or last frame
                let current = AnimationPlayer::get_servo_keyframe_from_frame(
                    frames,
                    frame_index,
                    servo_index,
                    false,
                );

                let (from_value, from_index) = match current {
                    Some(pair) => (pair.0 .0, pair.1),
                    None => (
                        frames[last_known_index[servo_index] as usize]
                            .as_ref()
                            .unwrap()
                            .beak_servo
                            .unwrap()
                            .0,
                        last_known_index[servo_index] as usize,
                    ),
                };

                // Update last known value
                last_known_index[servo_index] = from_index;

                // Get next frame
                let to = AnimationPlayer::get_servo_keyframe_from_frame(
                    frames,
                    frame_index + 1,
                    servo_index,
                    true,
                );

                let ((to_value, easing), to_index) = match to {
                    Some(pair) => (pair.0, pair.1),
                    None => {
                        for _ in 0..interpolation_steps {
                            result[servo_index].as_mut().unwrap().push(None);
                        }
                        continue;
                    }
                };

                // Check if we need to interpolate
                if to_value == from_value {
                    // No need to interpolate if the values are the same
                    for _ in 0..interpolation_steps {
                        result[servo_index].as_mut().unwrap().push(None);
                    }
                    continue;
                }

                let frame_span = to_index - from_index;

                let looper = (0..interpolation_steps)
                    .map(|i| i + ((frame_index - from_index) * interpolation_steps));

                for step in looper {
                    // Calculate the interpolation step, taking into account there might be missing frames so we have to spread it out over these
                    let t = step as f32 / (interpolation_steps * frame_span) as f32;
                    let value = Self::interpolate(from_value, to_value, t, &easing);
                    result[servo_index].as_mut().unwrap().push(Some(value));
                }
            }
        }

        result
    }

    fn get_servo_keyframe_from_frame(
        frames: &Vec<Option<Frame>>,
        start_index: usize,
        servo_index: usize,
        forward: bool,
    ) -> Option<((u16, Easing), usize)> {
        let len = frames.len();
        let mut index = start_index;

        loop {
            if index >= len {
                break;
            }

            if let Some(frame) = &frames[index] {
                let keyframe = match servo_index {
                    0 => frame.beak_servo,
                    1 => frame.neck_servo,
                    2 => frame.wing_right_servo,
                    3 => frame.wing_left_servo,
                    _ => None,
                };

                if keyframe.is_some() {
                    return keyframe.map(|kf| (kf, index));
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
