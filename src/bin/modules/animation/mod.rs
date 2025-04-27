use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

pub static ANIMATION_QUEUE: Channel<CriticalSectionRawMutex, u8, 4> = Channel::new();