use core::net::Ipv4Addr;

use defmt::{error, info, warn};
use embassy_net::{tcp::TcpSocket, Stack};
use embassy_time::{with_timeout, Duration, Timer};
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    packet::v5::{publish_packet::QualityOfService as QoS, reason_codes::ReasonCode},
    utils::rng_generator::CountingRng,
};

use crate::modules::servo::{animation::ANIMATION_QUEUE, animations::AnimationType};

const TAG: &str = "[MQTT]";

static MQTT_USERNAME: &str = env!("MQTT_USERNAME");
static MQTT_PASSWORD: &str = env!("MQTT_PASSWORD");
static MQTT_CLIENT_ID: &str = env!("MQTT_CLIENT_ID");

fn handle_mqtt_error(e: ReasonCode) -> bool {
    match e {
        ReasonCode::NetworkError => {
            error!("{} Network Error", TAG);
            true
        }
        _ => {
            error!("{} Other Error: {:?}", TAG, e);
            false
        }
    }
}

#[embassy_executor::task]
pub async fn mqtt_init(stack: Stack<'static>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut reconnect_delay_secs = 1;
    let mut first_run = true;

    loop {
        if !stack.is_config_up() {
            stack.wait_config_up().await;
            info!("{} WIFI stack is up, connecting to MQTT broker", TAG);
        }

        if first_run {
            first_run = false;
        } else {
            Timer::after(embassy_time::Duration::from_secs(reconnect_delay_secs)).await;
        }

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Addr::new(192, 168, 1, 50), 1883);
        info!("{} connecting...", TAG);

        if let Err(error) = socket.connect(remote_endpoint).await {
            error!("{} connect error: {}", TAG, error);
            reconnect_delay_secs = (reconnect_delay_secs * 2).min(60);
            continue;
        }

        reconnect_delay_secs = 1;
        info!("{} TCP connected!", TAG);

        let mut config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );

        config.add_max_subscribe_qos(QoS::QoS1);
        config.add_username(MQTT_USERNAME);
        config.add_password(MQTT_PASSWORD);
        config.add_client_id(MQTT_CLIENT_ID);
        config.keep_alive = 120;
        config.max_packet_size = 100;

        let mut recv_buffer = [0; 256];
        let mut write_buffer = [0; 256];

        let mut client = MqttClient::<_, 5, _>::new(
            socket,
            &mut write_buffer,
            256,
            &mut recv_buffer,
            256,
            config,
        );

        info!("{} client created", TAG);

        if let Err(mqtt_error) = client.connect_to_broker().await {
            handle_mqtt_error(mqtt_error);
            continue;
        }

        info!("{} connected to broker!", TAG);

        if let Err(_) = client.subscribe_to_topic("owlimatronic/event").await {
            error!("{} Failed to subscribe to topics", TAG);
            continue;
        } else {
            info!("{} Subsribed to topics", TAG);
        }

        loop {
            let result = with_timeout(Duration::from_secs(8), client.receive_message()).await;

            match result {
                // Message received
                Ok(message) => match message {
                    Ok((topic, payload)) => {
                        info!("{} Received: {} {}", TAG, topic, payload);
                        let animation = match AnimationType::get_from_binary(&payload) {
                            Some(anim) => anim,
                            None => {
                                warn!("{} animation not found {}", TAG, payload);
                                continue;
                            }
                        };

                        ANIMATION_QUEUE.send(animation).await;
                    }
                    Err(error) => {
                        error!("{} {}", TAG, error);
                    }
                },

                // Timeout occurred
                Err(_) => {
                    if let Err(e) = client.send_ping().await {
                        if handle_mqtt_error(e) {
                            break;
                        } else {
                            continue;
                        }
                    }
                }
            }

            Timer::after(Duration::from_millis(100)).await;
        }
    }
}
