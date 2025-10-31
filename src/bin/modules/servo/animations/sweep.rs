use crate::modules::{audio::tracks::Tracks, servo::{
    animation::{Animation, Frame},
    easing::Easing,
}};

const EASING: Easing = Easing::CubicInOut;

pub static ANIMATION: &Animation = &[
    Some(Frame::default()),
    Some(Frame {
        beak_servo: Some((0, EASING)),
        neck_servo: Some((0, EASING)),
        wing_right_servo: Some((0, EASING)),
        wing_left_servo: Some((0, EASING)),
        audio: Some(Tracks::BuboYap1),
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
        audio: Some(Tracks::BuboYap1),
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
