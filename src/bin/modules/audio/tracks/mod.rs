#[derive(Clone, Copy, Debug)]
pub enum Tracks {
    BuboRatched1,
    BuboRatched2,
    BuboRatched3,
    BuboYap1,
    BuboYap2,
    BuboYap3,
    BuboYap4,
    BuboYap5,
    BuboYap6,
    BuboYap7,
    BuboYap8,
}

// ffmpeg -i sound.mp3 -ac 1 -ar 16000 -f s16le -c:a pcm_s16le sound.pcm

// impl format
impl Tracks {
    pub fn get_name(&self) -> &'static str {
        match self {
            Tracks::BuboRatched1 => "Bubo Ratched 1",
            Tracks::BuboRatched2 => "Bubo Ratched 2",
            Tracks::BuboRatched3 => "Bubo Ratched 3",
            Tracks::BuboYap1 => "Bubo Yap 1",
            Tracks::BuboYap2 => "Bubo Yap 2",
            Tracks::BuboYap3 => "Bubo Yap 3",
            Tracks::BuboYap4 => "Bubo Yap 4",
            Tracks::BuboYap5 => "Bubo Yap 5",
            Tracks::BuboYap6 => "Bubo Yap 6",
            Tracks::BuboYap7 => "Bubo Yap 7",
            Tracks::BuboYap8 => "Bubo Yap 8",
        }
    }
}

impl Tracks {
    pub fn get_file(&self) -> &'static [u8] {
        match self {
            Tracks::BuboRatched1 => include_bytes!("bubo_ratched_1.pcm"),
            Tracks::BuboRatched2 => include_bytes!("bubo_ratched_2.pcm"),
            Tracks::BuboRatched3 => include_bytes!("bubo_ratched_3.pcm"),
            Tracks::BuboYap1 => include_bytes!("bubo_yap_1.pcm"),
            Tracks::BuboYap2 => include_bytes!("bubo_yap_2.pcm"),
            Tracks::BuboYap3 => include_bytes!("bubo_yap_3.pcm"),
            Tracks::BuboYap4 => include_bytes!("bubo_yap_4.pcm"),
            Tracks::BuboYap5 => include_bytes!("bubo_yap_5.pcm"),
            Tracks::BuboYap6 => include_bytes!("bubo_yap_6.pcm"),
            Tracks::BuboYap7 => include_bytes!("bubo_yap_7.pcm"),
            Tracks::BuboYap8 => include_bytes!("bubo_yap_8.pcm"),
        }
    }
}
