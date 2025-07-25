use crate::modules::servo::{
    animation::{Animation, Frame},
    easing::Easing,
};

pub static ANIMATION: &Animation = &[
    Some(Frame::default()),
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: None,
        wing_right_servo: Some((1000, Easing::Linear)),
        wing_left_servo: Some((1000, Easing::Linear)),
        audio: None,
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(Frame {
        beak_servo: Some((1000, Easing::CubicInOut)),
        neck_servo: None,
        wing_right_servo: Some((1000, Easing::Linear)),
        wing_left_servo: Some((1000, Easing::Linear)),
        audio: None,
    }),
    Some(Frame::default()),
];
