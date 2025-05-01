pub mod test;
pub mod yap;

use super::Animation;

pub enum Animations {
    Yap,
    Test,
}

impl Animations {
    pub fn get_animation(&self) -> &'static Animation {
        match self {
            Animations::Test => &test::ANIMATION,
            Animations::Yap => &yap::ANIMATION,
        }
    }
}

