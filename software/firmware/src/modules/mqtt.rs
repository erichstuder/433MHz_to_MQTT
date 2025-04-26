//! Sets up and handles the MQTT connection.

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        use defmt::{info, error};
        use embassy_executor::{task, Spawner};
        use embassy_rp::gpio;
        use embassy_time::{Duration, Timer};
        use embassy_net;
        use embassy_rp::clocks::RoscRng;
        use embassy_rp::pio::Pio;
        use embassy_rp::peripherals::{DMA_CH1, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
        use rand_core::RngCore; // Don't know why this is needed. Is it because the 'use' is missing in embassy_rp::clocks::RoscRng?
        use static_cell::StaticCell;
        use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
        use cyw43::JoinOptions;
        use core::net::Ipv4Addr;
        use heapless::String;
        use rust_mqtt::client::client::MqttClient;
        use rust_mqtt::utils::rng_generator::CountingRng;
        use embassy_sync::mutex::Mutex;
        use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

        use crate::modules::persistency::{self, PersistencyTrait};

        type MqttClientMutexed = Mutex<CriticalSectionRawMutex, MqttClient<'static, embassy_net::tcp::TcpSocket<'static>, 5, CountingRng>>;

        pub struct WifiHw {
            pub pin_23: PIN_23,
            pub pin_24: PIN_24,
            pub pin_25: PIN_25,
            pub pin_29: PIN_29,
            pub pio_1: Pio<'static, PIO1>,
            pub dma_ch1: DMA_CH1,
        }

        const MQTT_BROKER_USERNAME_LENGTH: usize = 32;
        const MQTT_BROKER_PASSWORD_LENGTH: usize = 64;

        struct Credentials {
            wifi_ssid: String<32>,
            wifi_password: String<32>,
            mqtt_host_ip: String<32>,
            mqtt_broker_username: String<MQTT_BROKER_USERNAME_LENGTH>,
            mqtt_broker_password: String<MQTT_BROKER_PASSWORD_LENGTH>,
        }
    }
}

use core::str;

pub struct MQTT {
    #[cfg(not(test))]
    client_mutexed: &'static MqttClientMutexed,
}

impl MQTT {
    #[cfg(not(test))]
    pub async fn new<P>(persistency: &'static P, mut hw: WifiHw, spawner: Spawner) -> Option<Self>
    where P: PersistencyTrait,
    {
        let fw = include_bytes!("../../../cyw43-firmware/43439A0.bin");
        let clm = include_bytes!("../../../cyw43-firmware/43439A0_clm.bin");

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
            hw.dma_ch1
        );

        static CYW43_STATE: StaticCell<cyw43::State> = StaticCell::new();
        let cyw43_state = CYW43_STATE.init(cyw43::State::new());
        let (net_device, mut control, runner) = cyw43::new(cyw43_state, pwr, spi, fw).await;
        spawner.spawn(cyw43_task(runner)).unwrap();

        control.init(clm).await;
        control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

        let config = embassy_net::Config::dhcpv4(Default::default());
        let mut rng = RoscRng;
        let seed = rng.next_u64();
        static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
        let (network_stack, network_runner) = embassy_net::new(net_device, config, RESOURCES.init(embassy_net::StackResources::new()), seed);
        spawner.spawn(net_task(network_runner)).unwrap();

        let mut credentials = Credentials {
            wifi_ssid: String::new(),
            wifi_password: String::new(),
            mqtt_host_ip: String::new(),
            mqtt_broker_username: String::new(),
            mqtt_broker_password: String::new(),
        };

        Self::get_credentials(persistency, &mut credentials).await.unwrap();

        static MQTT_BROKER_USERNAME: StaticCell<String<MQTT_BROKER_USERNAME_LENGTH>> = StaticCell::new();
        let mqtt_broker_username = MQTT_BROKER_USERNAME.init(credentials.mqtt_broker_username.clone());
        static MQTT_BROKER_PASSWORD: StaticCell<String<MQTT_BROKER_PASSWORD_LENGTH>> = StaticCell::new();
        let mqtt_broker_password = MQTT_BROKER_PASSWORD.init(credentials.mqtt_broker_password.clone());

        loop {
            match control.join(credentials.wifi_ssid.as_str(), JoinOptions::new(credentials.wifi_password.as_bytes())).await {
                Ok(_) => {
                    info!("join successful");
                    break
                },
                Err(err) => info!("join failed with status={}", err.status),
            }
        }

        info!("waiting for DHCP...");
        while !network_stack.is_config_up() {
            Timer::after_millis(100).await;
        }
        info!("DHCP is now up!");

        let (ip0, ip1, ip2, ip3) = Self::parse_ip(&credentials.mqtt_host_ip).unwrap();
        let address = Ipv4Addr::new(ip0, ip1, ip2, ip3);
        let remote_endpoint = (address, 1883);

        //TODO: The following buffer sizes have mostly been taken from examples. There might be better values.
        static RX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
        let rx_buffer = RX_BUFFER.init([0; 4096]);
        static TX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
        let tx_buffer = TX_BUFFER.init([0; 4096]);
        let mut socket = embassy_net::tcp::TcpSocket::new(network_stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(100)));
        static RECV_BUFFER: StaticCell<[u8; 150]> = StaticCell::new(); //was 80
        let recv_buffer = RECV_BUFFER.init([0; 150]);
        static WRITE_BUFFER: StaticCell<[u8; 150]> = StaticCell::new(); //was 80
        let write_buffer = WRITE_BUFFER.init([0; 150]);

        let connection = socket.connect(remote_endpoint).await;
        if let Err(e) = connection {
            error!("connect error: {:?}", e);
        }
        info!("connected to broker!");

        let mut config = rust_mqtt::client::client_config::ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            rust_mqtt::utils::rng_generator::CountingRng(20000),
        );
        config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);
        config.add_client_id("433MHz_to_MQTT");
        config.add_username(mqtt_broker_username);
        config.add_password(mqtt_broker_password);
        config.max_packet_size = 150; //was 100

        let client = MqttClient::<_, 5, _>::new(
            socket,
            write_buffer,
            150,
            recv_buffer,
            150,
            config,
        );
        static CLIENT_MUTEXED: StaticCell<MqttClientMutexed> = StaticCell::new();
        let client_mutexed = CLIENT_MUTEXED.init(Mutex::new(client));

        loop {
            let mut client = client_mutexed.lock().await;
            match client.connect_to_broker().await {
                Ok(()) => {
                    info!("Connected to broker 555");
                    break;
                }
                Err(mqtt_error) => match mqtt_error {
                    rust_mqtt::packet::v5::reason_codes::ReasonCode::NetworkError => error!("MQTT Network Error"),
                    _ => error!("Other MQTT Error: {:?}", mqtt_error),
                },
            }
            Timer::after(Duration::from_millis(2000)).await;
        }

        spawner.spawn(ping_task(client_mutexed)).unwrap();

        Some(Self {
            client_mutexed,
        })
    }

    // TODO: Test for this function as soon as PersistencyMutexed can easily be mocked, if ever.
    #[cfg(not(test))]
    async fn get_credentials<P>(persistency: &P, credentials: &mut Credentials) -> Result<(), &'static str>
    where P: PersistencyTrait,
    {

        let mut wifi_ssid = ['\0' as u8; 32];
        match persistency.read(persistency::ValueId::WifiSsid, &mut wifi_ssid).await {
            Ok(_) => credentials.wifi_ssid.push_str(str::from_utf8(&wifi_ssid).unwrap().trim_end_matches('\0')).unwrap(),
            Err(e) => return Err(e),
        };

        let mut wifi_password = ['\0' as u8; 32];
        match persistency.read(persistency::ValueId::WifiPassword, &mut wifi_password).await {
            Ok(_) => credentials.wifi_password.push_str(str::from_utf8(&wifi_password).unwrap().trim_end_matches('\0')).unwrap(),
            Err(e) => return Err(e),
        };

        let mut mqtt_host_ip = ['\0' as u8; 32];
        match persistency.read(persistency::ValueId::MqttHostIp, &mut mqtt_host_ip).await {
            Ok(_) => credentials.mqtt_host_ip.push_str(str::from_utf8(&mqtt_host_ip).unwrap().trim_end_matches('\0')).unwrap(),
            Err(e) => return Err(e),
        };

        let mut mqtt_broker_username = ['\0' as u8; 32];
        match persistency.read(persistency::ValueId::MqttBrokerUsername, &mut mqtt_broker_username).await {
            Ok(_) => credentials.mqtt_broker_username.push_str(str::from_utf8(&mqtt_broker_username).unwrap().trim_end_matches('\0')).unwrap(),
            Err(e) => return Err(e),
        };

        let mut mqtt_broker_password = ['\0' as u8; 64];
        match persistency.read(persistency::ValueId::MqttBrokerPassword, &mut mqtt_broker_password).await {
            Ok(_) => credentials.mqtt_broker_password.push_str(str::from_utf8(&mqtt_broker_password).unwrap().trim_end_matches('\0')).unwrap(),
            Err(e) => return Err(e),
        };

        info!("ssid: {:?}", credentials.wifi_ssid);
        info!("password: {:?}", credentials.wifi_password);
        info!("mqtt_host_ip: {:?}", credentials.mqtt_host_ip);
        info!("mqtt_broker_username: {:?}", credentials.mqtt_broker_username);
        info!("mqtt_broker_password: {:?}", credentials.mqtt_broker_password);

        Ok(())
    }

    fn parse_ip(mqtt_host_ip: &str) -> Option<(u8, u8, u8, u8)> {
        let mut ip = [0u8; 4];
        let mut count = 0;
        for (n, part) in mqtt_host_ip.as_bytes().split(|&b| b == b'.').enumerate() {
            if n >= ip.len() {
                #[cfg(not(test))]
                error!("invalid mqtt host ip format");
                return None
            }
            let part = str::from_utf8(part).unwrap();
            let part = part.parse::<u8>().unwrap();
            ip[n] = part;
            count += 1;
        }
        if count != 4 {
            #[cfg(not(test))]
            error!("invalid mqtt host ip format");
            return None
        }
        Some((ip[0], ip[1], ip[2], ip[3]))
    }

    #[cfg(not(test))]
    pub async fn send_message(&mut self, payload: &[u8]) {
        let mut client = self.client_mutexed.lock().await;
        let result = client.send_message("433", payload, rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1, false).await;
        match result {
            Ok(()) => info!("message sent"),
            Err(mqtt_error) => info!("message NOT sent: {:?}", mqtt_error),
        }
    }
}

#[cfg(not(test))]
#[task]
async fn cyw43_task(runner: cyw43::Runner<'static, gpio::Output<'static>, cyw43_pio::PioSpi<'static, PIO1, 0, DMA_CH1>>) -> ! {
    runner.run().await
}

#[cfg(not(test))]
#[task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[cfg(not(test))]
#[task]
async fn ping_task(client: &'static MqttClientMutexed) -> ! {
    loop {
        Timer::after(Duration::from_secs(30)).await;

        let mut client = client.lock().await;
        let result = client.send_ping().await;
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
