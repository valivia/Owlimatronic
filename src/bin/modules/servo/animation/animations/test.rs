use crate::modules::{
    audio::tracks::Tracks,
    servo::animation::{easing::Easing, Animation, Frame},
};

pub static ANIMATION: &Animation = &[
    Some(Frame::default()),
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: Some((300, Easing::CubicInOut)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: Some(Tracks::Hoot),
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
        beak_servo: Some((0, Easing::CubicInOut)),
        neck_servo: Some((300, Easing::CubicInOut)),
        wing_right_servo: None,
        wing_left_servo: None,
        audio: None,
    }),
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: None,
        wing_right_servo: Some((0, Easing::CubicInOut)),
        wing_left_servo: Some((0, Easing::CubicInOut)),
        audio: None,
    }),
    Some(Frame::default()),
];
