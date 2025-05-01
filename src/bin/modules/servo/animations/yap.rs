use crate::modules::{
    audio::tracks::Tracks,
    servo::{
        animation::{Animation, Frame}, config::DEFAULT_NECK_POSITION, easing::Easing
    },
};

pub static ANIMATION: &Animation = &[
    Some(Frame::default()),
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: Some((DEFAULT_NECK_POSITION + 100, Easing::CubicInOut)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: Some(Tracks::Hoot),
    }),
    Some(Frame {
        beak_servo: Some((0, Easing::CubicInOut)),
        neck_servo: None,
        wing_right_servo: None,
        wing_left_servo: None,
        audio: None,
    }),
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: Some((DEFAULT_NECK_POSITION - 100, Easing::CubicInOut)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: None,
    }),
    Some(Frame::default()),
];
