#[derive(Clone, Copy)]
pub enum AudioFiles {
    Hoot,
}

impl AudioFiles {
    pub fn get_file(&self) -> &'static [u8] {
        match self {
            AudioFiles::Hoot => include_bytes!("owl.pcm"),
        }
    }
}
