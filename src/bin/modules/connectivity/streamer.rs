use core::{mem::MaybeUninit, net::Ipv4Addr};

use defmt::{error, info};
use embassy_futures::yield_now;
use embassy_net::{tcp::TcpSocket, Stack};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use ringbuf::{storage::Owning, traits::Producer, wrap::caching::Caching, SharedRb};

use crate::modules::audio::AUDIO_STREAM;

const TAG: &str = "[STREAMER]";

pub static STREAMER_TRIGGER: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub const AUDIO_CHUNK_SIZE: usize = 2 * 4092;

#[derive(PartialEq, Copy, Clone)]
pub struct AudioChunk {
    pub data: [u8; AUDIO_CHUNK_SIZE],
    pub len: usize,
}

pub static STREAM_SIZE: usize = 8;
// pub type StreamChunk = [u8; 2048];
pub type StreamRingBuffer = SharedRb<Owning<[MaybeUninit<AudioChunk>; STREAM_SIZE]>>;
pub type StreamConsumer = Caching<&'static StreamRingBuffer, false, true>;
pub type StreamProducer = Caching<&'static StreamRingBuffer, true, false>;

#[embassy_executor::task]
pub async fn streamer_init(stack: Stack<'static>, mut stream: StreamProducer) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut reconnect_delay_secs = 1;
    let mut first_run = true;

    info!("{} Starting", TAG);

    loop {
        // Wait for streamer trigger
        STREAMER_TRIGGER.wait().await;

        // Ensure network stack is up
        if !stack.is_config_up() {
            stack.wait_config_up().await;
            info!("{} WIFI stack is up", TAG);
        }

        if first_run {
            first_run = false;
        } else {
            Timer::after(Duration::from_secs(reconnect_delay_secs)).await;
        }

        // Create and connect TCP socket
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Addr::new(192, 168, 1, 50), 9000);
        info!("{} connecting...", TAG);

        if let Err(e) = socket.connect(remote_endpoint).await {
            error!("{} connect error: {}", TAG, e);
            reconnect_delay_secs = (reconnect_delay_secs * 2).min(60);
            continue;
        }

        reconnect_delay_secs = 1;
        info!("{} TCP connected!", TAG);

        AUDIO_STREAM.signal(true);

        let mut buffer = [0u8; 1024];
        let mut send_buffer = [0u8; AUDIO_CHUNK_SIZE];
        let mut buffer_pos = 0;

        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    info!("{} stream ended", TAG);
                    break;
                }
                Ok(n) => {
                    let mut read_pos = 0;

                    while read_pos < n {
                        // Copy as much as needed to fill send_buffer
                        let to_copy = (AUDIO_CHUNK_SIZE - buffer_pos).min(n - read_pos);
                        send_buffer[buffer_pos..buffer_pos + to_copy]
                            .copy_from_slice(&buffer[read_pos..read_pos + to_copy]);

                        buffer_pos += to_copy;
                        read_pos += to_copy;

                        // If buffer full, push to ring buffer
                        if buffer_pos == AUDIO_CHUNK_SIZE {
                            let chunk = AudioChunk {
                                data: send_buffer,
                                len: AUDIO_CHUNK_SIZE,
                            };

                            try_send(&mut stream, chunk).await;

                            buffer_pos = 0; // Reset for next chunk
                        }
                    }
                }
                Err(e) => {
                    error!("{} read error: {}", TAG, e);
                    break;
                }
            }
        }

        // Flush any leftover bytes at the end of the stream
        if buffer_pos > 0 {
            let chunk = AudioChunk {
                data: send_buffer,
                len: buffer_pos, // actual bytes read
            };

            try_send(&mut stream, chunk).await;
        }

        let _ = socket.close();
    }
}

async fn try_send(stream: &mut StreamProducer, mut chunk: AudioChunk) {
    loop {
        match stream.try_push(chunk) {
            Ok(_) => break,
            Err(returned_chunk) => {
                yield_now().await;
                chunk = returned_chunk;
            }
        }
    }
}
