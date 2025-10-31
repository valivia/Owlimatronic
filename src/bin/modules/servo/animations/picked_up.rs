use crate::modules::{
    audio::tracks::Tracks,
    servo::{
        animation::{Animation, Frame},
        config::{DEFAULT_NECK_POSITION, DEFAULT_WING_POSITION},
        easing::Easing,
    },
};

pub static ANIMATION: &Animation = &[
    Some(Frame {
        beak_servo: Some((1000, Easing::Linear)),
        neck_servo: Some((DEFAULT_NECK_POSITION, Easing::Linear)),
        wing_right_servo: Some((DEFAULT_WING_POSITION, Easing::Linear)),
        wing_left_servo: Some((DEFAULT_WING_POSITION, Easing::Linear)),
        audio: Some(Tracks::BuboYap1),
    }),
    Some(Frame {
        beak_servo: None,
        neck_servo: None,
        wing_right_servo: Some((1000, Easing::CubicInOut)),
        wing_left_servo: Some((1000, Easing::Linear)),
        audio: None,
    }),
    Some(Frame::default()),
];
