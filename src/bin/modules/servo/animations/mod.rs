use super::animation::Animation;

pub mod picked_up;
pub mod hello;
pub mod panic;
pub mod shocked;
pub mod sweep;
pub mod test;
pub mod yap;

pub enum AnimationType {
    Yap,
    Hello,
    Test,
    Sweep,
    Panic,
    Shocked,
    PickedUp,
}

impl AnimationType {
    pub fn get_animation(&self) -> &'static Animation {
        match self {
            AnimationType::Test => &test::ANIMATION,
            AnimationType::Yap => &yap::ANIMATION,
            AnimationType::Hello => &hello::ANIMATION,
            AnimationType::Sweep => &sweep::ANIMATION,
            AnimationType::Panic => &panic::ANIMATION,
            AnimationType::Shocked => &shocked::ANIMATION,
            AnimationType::PickedUp => &picked_up::ANIMATION,
        }
    }

    pub fn get_from_binary(payload: &[u8]) -> Option<AnimationType> {
        match payload {
            b"shocked" => Some(AnimationType::Shocked),
            b"hello" => Some(AnimationType::Hello),
            b"sweep" => Some(AnimationType::Sweep),
            b"panic" => Some(AnimationType::Panic),
            b"yap" => Some(AnimationType::Yap),
            b"pick_up" => Some(AnimationType::PickedUp),
            _ => None,
        }
    }
}
