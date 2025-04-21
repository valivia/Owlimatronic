#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    CubicInOut,
}

impl Easing {
    pub fn ease(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let f = t - 1.0;
                    1.0 + 4.0 * f * f * f
                }
            }
        }
    }
}
