use crate::modules::{
    audio::tracks::Tracks,
    servo::{
        animation::{Animation, Frame},
        easing::Easing,
    },
};

pub static ANIMATION: &Animation = &[
    Some(Frame::default()),
    Some(Frame {
        beak_servo: None,
        neck_servo: Some((1000, Easing::Linear)),
        wing_right_servo: None,
        wing_left_servo: None,

        audio: None,
    }),
    None,
    None,
    Some(Frame {
        beak_servo: Some((0, Easing::Linear)),
        neck_servo: None,
        wing_right_servo: Some((0, Easing::Linear)),
        wing_left_servo: Some((0, Easing::Linear)),
        audio: None,
    }),
    Some(Frame {
        beak_servo: Some((1000, Easing::Linear)),
        neck_servo: None,
        wing_right_servo: Some((1000, Easing::Linear)),
        wing_left_servo: Some((1000, Easing::Linear)),
        audio: Some(Tracks::Hello),
    }),
    None,
    Some(Frame {
        beak_servo: Some((0, Easing::Linear)),
        neck_servo: None,
        wing_right_servo: Some((0, Easing::Linear)),
        wing_left_servo: Some((0, Easing::Linear)),
        audio: None,
    }),
    Some(Frame {
        beak_servo: None,
        neck_servo: Some((1000, Easing::Linear)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: None,
    }),
    Some(Frame::default()),
];
