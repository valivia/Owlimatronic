#[derive(Clone, Copy, Debug)]
pub enum Tracks {
    Hoot,
    Hello,
}

// ffmpeg -i sound.mp3 -ac 1 -ar 16000 -f s16le -c:a pcm_s16le sound.pcm

// impl format
impl Tracks {
    pub fn get_name(&self) -> &'static str {
        match self {
            Tracks::Hoot => "Hoot",
            Tracks::Hello => "Hello",
        }
    }
}

impl Tracks {
    pub fn get_file(&self) -> &'static [u8] {
        match self {
            Tracks::Hoot => include_bytes!("hoot.pcm"),
            Tracks::Hello => include_bytes!("hello.pcm"),
        }
    }
}
