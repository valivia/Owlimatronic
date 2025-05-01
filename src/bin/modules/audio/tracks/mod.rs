#[derive(Clone, Copy, Debug)]
pub enum Tracks {
    Hoot,
}

// ffmpeg -i owl2.mp3 -ac 1 -ar 16000 -f s16le -c:a pcm_s16le hoot.pcm

// impl format
impl Tracks {
    pub fn get_name(&self) -> &'static str {
        match self {
            Tracks::Hoot => "Hoot",
        }
    }
}

impl Tracks {
    pub fn get_file(&self) -> &'static [u8] {
        match self {
            Tracks::Hoot => include_bytes!("hoot.pcm"),
        }
    }
}
