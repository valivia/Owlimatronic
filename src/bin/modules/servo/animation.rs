use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Duration;

use crate::modules::audio::tracks::Tracks;

use super::{
    animations::AnimationType,
    config::{DEFAULT_BEAK_POSITION, DEFAULT_NECK_POSITION, DEFAULT_WING_POSITION},
    easing::Easing,
};

pub const KEYFRAME_DURATION: Duration = Duration::from_millis(250);
pub const INTERPOLATION_STEPS: u32 = 20;

pub const FRAME_DURATION: Duration =
    Duration::from_millis(KEYFRAME_DURATION.as_millis() / INTERPOLATION_STEPS as u64);

pub static ANIMATION_QUEUE: Channel<CriticalSectionRawMutex, AnimationType, 4> = Channel::new();

type ServoKeyframe = (u16, Easing);
type AudioKeyframe = Tracks;

#[derive(Clone)]
pub struct Frame {
    pub beak_servo: Option<ServoKeyframe>,
    pub neck_servo: Option<ServoKeyframe>,
    pub wing_right_servo: Option<ServoKeyframe>,
    pub wing_left_servo: Option<ServoKeyframe>,
    pub audio: Option<AudioKeyframe>,
}

impl Frame {
    pub fn get_servo(&self, servo: usize) -> Option<ServoKeyframe> {
        match servo {
            0 => self.beak_servo,
            1 => self.neck_servo,
            2 => self.wing_right_servo,
            3 => self.wing_left_servo,
            _ => None,
        }
    }

    pub const fn default() -> Self {
        Self {
            beak_servo: Some((DEFAULT_BEAK_POSITION, Easing::Linear)),
            neck_servo: Some((DEFAULT_NECK_POSITION, Easing::Linear)),
            wing_right_servo: Some((DEFAULT_WING_POSITION, Easing::Linear)),
            wing_left_servo: Some((DEFAULT_WING_POSITION, Easing::Linear)),
            audio: None,
        }
    }

    pub const fn empty() -> Self {
        Self {
            beak_servo: None,
            neck_servo: None,
            wing_right_servo: None,
            wing_left_servo: None,
            audio: None,
        }
    }
}

pub type Animation = [Option<Frame>];
