use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_hal::{
    peripherals::{TIMG0, WIFI},
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_wifi::{
    init,
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState},
    EspWifiController,
};

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

static WIFI_SSID: &str = env!("WIFI_SSID");
static WIFI_PSK: &str = env!("WIFI_PASS");
const TAG: &str = "[WIFI]";

pub async fn wifi_init(
    spawner: Spawner,
    timer1: TIMG0<'static>,
    wifi: WIFI<'static>,
    mut rng: Rng,
) -> Stack<'static> {
    let timg0 = TimerGroup::new(timer1);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone()).unwrap()
    );

    let (controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, wifi).unwrap();

    let wifi_interface = interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("{} Waiting to get IP address...", TAG);
    loop {
        if let Some(config) = stack.config_v4() {
            info!("{} Got IP: {}", TAG, config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    return stack;
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("{} Start connection task", TAG);
    info!(
        "{} Device capabilities: {:?}",
        TAG,
        controller.capabilities()
    );
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            info!("{} Connecting to {} ({})", TAG, WIFI_SSID, WIFI_PSK);
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.into(),
                password: WIFI_PSK.into(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            controller.start_async().await.unwrap();

            info!("{} Scan", TAG);
            let networks = controller.scan_n_async(10).await.unwrap();

            for ap in networks {
                info!("{} Found AP: {:?}", TAG, ap);
            }
        }
        info!("{} About to connect...", TAG);

        match controller.connect_async().await {
            Ok(_) => info!("{} Wifi connected!", TAG),
            Err(e) => {
                error!("{} Failed to connect to wifi: {}", TAG, e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
