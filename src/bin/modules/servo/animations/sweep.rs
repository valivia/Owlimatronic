use crate::modules::servo::{
    animation::{Animation, Frame},
    easing::Easing,
};

const EASING: Easing = Easing::CubicInOut;

pub static ANIMATION: &Animation = &[
    Some(Frame::default()),
    Some(Frame {
        beak_servo: Some((0, EASING)),
        neck_servo: Some((0, EASING)),
        wing_right_servo: Some((0, EASING)),
        wing_left_servo: Some((0, EASING)),
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
    Some(Frame {
        beak_servo: Some((1000, EASING)),
        neck_servo: Some((1000, EASING)),
        wing_right_servo: Some((1000, EASING)),
        wing_left_servo: Some((1000, EASING)),
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
    Some(Frame {
        beak_servo: Some((0, EASING)),
        neck_servo: Some((0, EASING)),
        wing_right_servo: Some((0, EASING)),
        wing_left_servo: Some((0, EASING)),
        audio: None,
    }),
    Some(Frame::default()),
];
