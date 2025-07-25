use super::animation::Animation;

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
        }
    }
}
