//! Sets up and handles the MQTT connection.

use core::str;
use defmt::{info, error, unwrap};
use embassy_executor::{task, Spawner};
use embassy_rp::{Peri, gpio, dma};
use embassy_time::{Duration, Timer};
use embassy_net;
use embassy_rp::clocks::RoscRng;
use embassy_rp::pio::Pio;
use embassy_rp::peripherals::{DMA_CH1, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
use static_cell::StaticCell;
use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
use cyw43::{aligned_bytes, JoinOptions};
use core::net::Ipv4Addr;
use rust_mqtt::types::{MqttString, MqttBinary};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use lib::persistency::{self, PersistencyTrait};

type MqttClientMutexed = Mutex<CriticalSectionRawMutex, rust_mqtt::client::Client<'static, embassy_net::tcp::TcpSocket<'static>, rust_mqtt::buffer::BumpBuffer<'static>, 8, 16, 16, 4>>;

pub struct WifiHw<'d> {
    pub pin_23: Peri<'d, PIN_23>,
    pub pin_24: Peri<'d, PIN_24>,
    pub pin_25: Peri<'d, PIN_25>,
    pub pin_29: Peri<'d, PIN_29>,
    pub pio_1: Pio<'static, PIO1>,
    pub dma_ch1: Peri<'d, DMA_CH1>,
}

pub struct MQTT {
    client_mutexed: &'static MqttClientMutexed,
}

impl MQTT {
    pub async fn new<P, I>(persistency: &'static mut P, hw: WifiHw<'static>, spawner: Spawner, irq: I) -> Option<Self>
    where
        P: PersistencyTrait,
        I: embassy_rp::interrupt::typelevel::Binding<embassy_rp::interrupt::typelevel::DMA_IRQ_0, embassy_rp::dma::InterruptHandler<DMA_CH1>> + 'static,
    {
        let (driver, mut control) = MQTT::setup_cyw43(hw, spawner, irq).await;
        let network_stack = MQTT::setup_network(driver, spawner);

        let mut wifi_ssid = [0u8; 32];
        let mut wifi_password = [0u8; 32];
        let mut mqtt_host_ip = [0u8; 32];
        let mut mqtt_broker_username = [0u8; 32];
        let mut mqtt_broker_password = [0u8; 64];

        persistency.read(persistency::Key::WifiSsid, &mut wifi_ssid).await;
        persistency.read(persistency::Key::WifiPassword, &mut wifi_password).await;
        persistency.read(persistency::Key::MqttHostIp, &mut mqtt_host_ip).await;
        persistency.read(persistency::Key::MqttBrokerUsername, &mut mqtt_broker_username).await;
        persistency.read(persistency::Key::MqttBrokerPassword, &mut mqtt_broker_password).await;

        loop {
            match control.join(str::from_utf8(&wifi_ssid).unwrap(), JoinOptions::new(&wifi_password)).await {
                Ok(_) => {
                    info!("join successful");
                    break
                },
                Err(err) => info!("join failed with status={:?}", err),
            }
        }

        info!("waiting for DHCP...");
        while !network_stack.is_config_up() {
            Timer::after_millis(100).await;
        }
        info!("DHCP is now up!");


        // TODO: ab hier wird mit dem broker verbunden
        let (ip0, ip1, ip2, ip3) = Self::parse_ip(&mqtt_host_ip).unwrap();
        let address = Ipv4Addr::new(ip0, ip1, ip2, ip3);
        let remote_endpoint = (address, 1883);

        //TODO: The following buffer sizes have mostly been taken from examples. There might be better values.
        static RX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
        let rx_buffer = RX_BUFFER.init([0; 4096]);
        static TX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
        let tx_buffer = TX_BUFFER.init([0; 4096]);
        let mut socket = embassy_net::tcp::TcpSocket::new(network_stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(100)));

        let connection = socket.connect(remote_endpoint).await;
        if let Err(e) = connection {
            error!("connect error: {:?}", e);
        }
        info!("connected to broker!");

        let mqtt_connect_options = rust_mqtt::client::options::ConnectOptions::new()
            // TODO: hier gibt es eine keep-alive funktion. vielleicht bräuchte man dann das pinging nicht mehr?
            .clean_start()
            .session_expiry_interval(rust_mqtt::config::SessionExpiryInterval::NeverEnd)
            .user_name(unwrap!(MqttString::from_str(str::from_utf8(&mqtt_broker_username).unwrap())))
            .password(unwrap!(MqttBinary::from_slice(&mqtt_broker_password)));


        static MQTT_BUMP_MEM: StaticCell<[u8; 2048]> = StaticCell::new();
        static MQTT_BUMP: StaticCell<rust_mqtt::buffer::BumpBuffer<'static>> = StaticCell::new();

        let bump_mem = MQTT_BUMP_MEM.init([0; 2048]);
        let bump = MQTT_BUMP.init(rust_mqtt::buffer::BumpBuffer::new(bump_mem));
        let client = rust_mqtt::client::Client::new(bump);

        static CLIENT_MUTEXED: StaticCell<MqttClientMutexed> = StaticCell::new();
        let client_mutexed = CLIENT_MUTEXED.init(Mutex::new(client));

        // loop { Note: At the moment we only try once to connect, due to a moved socket.
            let mut client = client_mutexed.lock().await;
            match client.connect(
                socket,
                &mqtt_connect_options,
                Some(MqttString::from_str("433MHz_to_MQTT").unwrap())
            ).await {
                Ok(info) => {
                    info!("Connected to broker with: {:?}", info);
                    // break;
                }
                Err(e) =>  {
                    error!("Other MQTT Error: {:?}", e);
                },
            }
        //     Timer::after(Duration::from_millis(2000)).await;
        // }

        spawner.spawn(unwrap!(ping_task(client_mutexed)));

        Some(Self {
            client_mutexed,
        })
    }

    async fn setup_cyw43<I>(mut hw: WifiHw<'static>, spawner: Spawner, irq: I) -> (cyw43::NetDriver<'static>, cyw43::Control<'static>)
    where
        // TODO: this is used in muliple places => make its own type?
        I: embassy_rp::interrupt::typelevel::Binding<embassy_rp::interrupt::typelevel::DMA_IRQ_0, embassy_rp::dma::InterruptHandler<DMA_CH1>> + 'static,
    {
        let fw = aligned_bytes!("../../../../cyw43-firmware/43439A0.bin");
        let clm = aligned_bytes!("../../../../cyw43-firmware/43439A0_clm.bin");
        let nvram = aligned_bytes!("../../../../cyw43-firmware/nvram_rp2040.bin");

        let pwr = gpio::Output::new(hw.pin_23, gpio::Level::Low);
        let cs = gpio::Output::new(hw.pin_25, gpio::Level::High);

        let spi = cyw43_pio::PioSpi::new(
            &mut hw.pio_1.common,
            hw.pio_1.sm0,
            DEFAULT_CLOCK_DIVIDER,
            hw.pio_1.irq0,
            cs,
            hw.pin_24,
            hw.pin_29,
            dma::Channel::new(hw.dma_ch1, irq)
        );

        static CYW43_STATE: StaticCell<cyw43::State> = StaticCell::new();
        let cyw43_state = CYW43_STATE.init(cyw43::State::new());
        let (net_device, mut control, runner) = cyw43::new(cyw43_state, pwr, spi, fw, nvram).await;
        spawner.spawn(unwrap!(cyw43_task(runner)));

        control.init(clm).await;
        control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

        (net_device, control)
    }

    fn setup_network(driver: cyw43::NetDriver<'static>, spawner: Spawner) -> embassy_net::Stack<'static>{
        let config = embassy_net::Config::dhcpv4(Default::default());
        let mut rng = RoscRng;
        let seed = rng.next_u64(); // TODO: dont know why the seed is important. couldn't it be a constant?
        static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
        let (network_stack, network_runner) = embassy_net::new(driver, config, RESOURCES.init(embassy_net::StackResources::new()), seed);
        spawner.spawn(unwrap!(net_task(network_runner)));
        network_stack
    }

    fn parse_ip(mqtt_host_ip: &[u8]) -> Option<(u8, u8, u8, u8)> {
        let mut ip = [0u8; 4];
        let mut count = 0;
        for (n, part) in mqtt_host_ip.split(|&b| b == b'.').enumerate() {
            if n >= ip.len() {
                error!("invalid mqtt host ip format");
                return None
            }
            let part = str::from_utf8(part).unwrap();
            let part = part.parse::<u8>().unwrap();
            ip[n] = part;
            count += 1;
        }
        if count != 4 {
            error!("invalid mqtt host ip format");
            return None
        }
        Some((ip[0], ip[1], ip[2], ip[3]))
    }

    pub async fn send_message(&mut self, payload: &[u8]) {
        let mut client = self.client_mutexed.lock().await;
        let topic = unwrap!(rust_mqtt::types::TopicName::new(unwrap!(MqttString::from_str("433MHz_to_MQTT_button"))));
        unwrap!(client.publish(
            &rust_mqtt::client::options::PublicationOptions::new(rust_mqtt::client::options::TopicReference::Name(topic.as_borrowed())).exactly_once(),
            payload.into()
        ).await);


    }
}

#[task]
async fn cyw43_task(runner: cyw43::Runner<'static, cyw43::SpiBus<gpio::Output<'static>, cyw43_pio::PioSpi<'static, PIO1, 0>>>) -> ! {
    runner.run().await
}

#[task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[task]
async fn ping_task(client: &'static MqttClientMutexed) -> ! {
    loop {
        Timer::after(Duration::from_secs(30)).await;

        let mut client = client.lock().await;
        let result = client.ping().await;
        match result {
            Ok(()) => info!("ping sent"),
            Err(mqtt_error) => info!("ping NOT sent: {:?}", mqtt_error),
        }
    }
}

#[cfg(test)]
mod test_for_parse_ip {
    use super::MQTT;
    use std::panic;

    #[test]
    fn pass() {
        let mqtt_host_ip = "123.55.6.2";
        let (ip0, ip1, ip2, ip3) = MQTT::parse_ip(mqtt_host_ip).unwrap();
        assert_eq!(ip0, 123);
        assert_eq!(ip1, 55);
        assert_eq!(ip2, 6);
        assert_eq!(ip3, 2);
    }

    #[test]
    fn too_long() {
        let mqtt_host_ip = "123.55.6.2.42";
        match MQTT::parse_ip(mqtt_host_ip) {
            Some(_) => assert!(false, "Expected None, but got Some"),
            None => assert!(true),
        }
    }

    #[test]
    fn too_short() {
        let mqtt_host_ip = "123.55.6";
        match MQTT::parse_ip(mqtt_host_ip) {
            Some(_) => assert!(false, "Expected None, but got Some"),
            None => assert!(true),
        }
    }

    #[test]
    fn letters() {
        let mqtt_host_ip = "123.55.6.X";
        let result = panic::catch_unwind(|| { MQTT::parse_ip(mqtt_host_ip) });
        match result {
            Ok(_) => assert!(false, "Expected panic, but got Ok"),
            Err(_) => assert!(true),
        }
    }
}
