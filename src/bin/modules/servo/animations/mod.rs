use super::animation::Animation;

pub mod test;
pub mod yap;

pub enum AnimationType {
    Yap,
    Test,
}

impl AnimationType {
    pub fn get_animation(&self) -> &'static Animation {
        match self {
            AnimationType::Test => &test::ANIMATION,
            AnimationType::Yap => &yap::ANIMATION,
        }
    }
}

