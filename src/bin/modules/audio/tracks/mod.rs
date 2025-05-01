#[derive(Clone, Copy)]
pub enum Tracks {
    Hoot,
}

impl Tracks {
    pub fn get_file(&self) -> &'static [u8] {
        match self {
            Tracks::Hoot => include_bytes!("owl.pcm"),
        }
    }
}
