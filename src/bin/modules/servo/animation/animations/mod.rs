pub mod test;

use test::TEST_ANIMATION;

use super::Animation;

pub enum Animations {
    Test,
}

impl Animations {
    pub fn get_animation(&self) -> &'static Animation {
        match self {
            Animations::Test => &TEST_ANIMATION,
        }
    }
}

