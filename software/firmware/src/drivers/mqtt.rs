#[cfg(not(test))]
use defmt::{unwrap, error};
use defmt::info;

#[cfg(not(test))]
use embassy_executor::{task, Spawner};

//use embassy_net::tcp::client;
//use embassy_rp::pac::xip_ctrl::regs::Stat;
use embassy_rp::pio::Pio;
use embassy_rp::peripherals::{DMA_CH1, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
#[cfg(not(test))]
use embassy_rp::gpio;
use rust_mqtt::utils::rng_generator::CountingRng;
#[cfg(not(test))]
use static_cell::StaticCell;
#[cfg(not(test))]
use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
#[cfg(not(test))]
use cyw43::JoinOptions;
#[cfg(not(test))]
use core::net::Ipv4Addr;
#[cfg(not(test))]
use core::str;
#[cfg(not(test))]
use embassy_net;
#[cfg(not(test))]
use embassy_rp::clocks::RoscRng;
#[cfg(not(test))]
use rand_core::RngCore; // Don't know why this is needed. Is it because the 'use' is missing in embassy_rp::clocks::RoscRng?
//use minimq::{ConfigBuilder, Minimq, Publication};
// use minimq::Minimq;
// use embedded_nal_async;
// use embedded_nal_async;
use rust_mqtt::client::client::MqttClient;
// use embassy_net:
#[cfg(not(test))]
use embassy_time::{Duration, Timer};

// use minimq::embedded_nal::
// use embedded_nal_async;

#[cfg(not(test))]
use crate::drivers::persistency;
#[cfg(not(test))]
use crate::PersistencyMutexed;

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub struct WifiHw {
    pub pin_23: PIN_23,
    pub pin_24: PIN_24,
    pub pin_25: PIN_25,
    pub pin_29: PIN_29,
    pub pio_1: Pio<'static, PIO1>,
    pub dma_ch1: DMA_CH1,
}

type MqttClientMutexed = Mutex<CriticalSectionRawMutex, MqttClient<'static, embassy_net::tcp::TcpSocket<'static>, 5, CountingRng>>;

pub struct MQTT {
    //rx_buffer: [u8; 4096],
    //tx_buffer: [u8; 4096],
    //recv_buffer: [u8; 150],
    //write_buffer: &'static mut[u8; 150],
    client_mutexed: &'static MqttClientMutexed,
}

impl MQTT {
    // pub fn new() -> Self {
    //     Self {
    //         socket: embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer),
    //         client: MqttClient::<_, 5, _>::new(  //was 5
    //             socket,
    //             &mut write_buffer,
    //             150,
    //             &mut recv_buffer,
    //             150,
    //             config,
    //         );,
    //     }
    // }

    //#[task]
    #[cfg(not(test))]
    pub async fn new(persistency: &'static PersistencyMutexed, mut hw: WifiHw, spawner: Spawner) -> Option<Self> {
        let fw = include_bytes!("../../../cyw43-firmware/43439A0.bin");
        let clm = include_bytes!("../../../cyw43-firmware/43439A0_clm.bin");

        let pwr = gpio::Output::new(hw.pin_23, gpio::Level::Low);
        let cs = gpio::Output::new(hw.pin_25, gpio::Level::High);
        let spi = cyw43_pio::PioSpi::new(&mut hw.pio_1.common, hw.pio_1.sm0, DEFAULT_CLOCK_DIVIDER, hw.pio_1.irq0, cs, hw.pin_24, hw.pin_29, hw.dma_ch1);

        static STATE: StaticCell<cyw43::State> = StaticCell::new();
        let state = STATE.init(cyw43::State::new());
        let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
        unwrap!(spawner.spawn(cyw43_task(runner))); //TODO: irgenwie wird hier eine andere unwrap funktion verwendet. warum?

        control.init(clm).await;
        control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

        let config = embassy_net::Config::dhcpv4(Default::default());
        let mut rng = RoscRng;
        let seed = rng.next_u64();
        static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(net_device, config, RESOURCES.init(embassy_net::StackResources::new()), seed);
        unwrap!(spawner.spawn(net_task(runner)));

        let mut wifi_ssid: [u8; 32] = ['\0' as u8; 32];
        let length = match persistency.lock().await.read(persistency::ValueId::WifiSsid, &mut wifi_ssid) {
            Ok(length) => length,
            Err(e) => {
                error!("Failed to read Wifi SSID: {:?}", e);
                return None;
            }
        };
        let wifi_ssid = str::from_utf8(&wifi_ssid[..length]).unwrap();

        let mut wifi_password: [u8; 32] = ['\0' as u8; 32];
        let length = match persistency.lock().await.read(persistency::ValueId::WifiPassword, &mut wifi_password) {
            Ok(length) => length,
            Err(e) => {
                error!("Failed to read Wifi Password: {:?}", e);
                return None;
            }
        };
        let wifi_password = &wifi_password[..length];

        let mut mqtt_host_ip: [u8; 32] = ['\0' as u8; 32];
        let length = match persistency.lock().await.read(persistency::ValueId::MqttHostIp, &mut mqtt_host_ip) {
            Ok(length) => length,
            Err(e) => {
                error!("Failed to read MQTT Host IP: {:?}", e);
                return None;
            }
        };
        let mqtt_host_ip = &mqtt_host_ip[..length];

        const MQTT_BROKER_USERNAME_LENGTH: usize = 32;
        let mut mqtt_broker_username: [u8; 32] = ['\0' as u8; MQTT_BROKER_USERNAME_LENGTH];
        let _ = match persistency.lock().await.read(persistency::ValueId::MqttBrokerUsername, &mut mqtt_broker_username) {
            Ok(length) => length,
            Err(e) => {
                error!("Failed to read MQTT Broker Username: {:?}", e);
                return None;
            }
        };
        static MQTT_BROKER_USERNAME: StaticCell<[u8; 32]> = StaticCell::new();
        let mqtt_broker_username = str::from_utf8(MQTT_BROKER_USERNAME.init(mqtt_broker_username)).unwrap().trim_end_matches('\0');

        const MQTT_BROKER_PASSWORD_LENGTH: usize = 64;
        let mut mqtt_broker_password = ['\0' as u8; MQTT_BROKER_PASSWORD_LENGTH];
        let _ = match persistency.lock().await.read(persistency::ValueId::MqttBrokerPassword, &mut mqtt_broker_password) {
            Ok(length) => length,
            Err(e) => {
                error!("Failed to read MQTT Broker Password: {:?}", e);
                return None;
            }
        };
        static MQTT_BROKER_PASSWORD: StaticCell<[u8; MQTT_BROKER_PASSWORD_LENGTH]> = StaticCell::new();
        let mqtt_broker_password = str::from_utf8(MQTT_BROKER_PASSWORD.init(mqtt_broker_password)).unwrap().trim_end_matches('\0');

        info!("ssid: {:?}", wifi_ssid);
        info!("password: {:?}", str::from_utf8(wifi_password).unwrap());
        info!("mqtt_host_ip: {:?}", str::from_utf8(mqtt_host_ip).unwrap());
        info!("mqtt_broker_username: {:?}", mqtt_broker_username);
        info!("mqtt_broker_password: {:?}", mqtt_broker_password);

        loop {
            match control.join(wifi_ssid, JoinOptions::new(wifi_password)).await {
                Ok(_) => {
                    info!("join successful");
                    break
                },
                Err(err) => {
                    info!("join failed with status={}", err.status);
                }
            }
        }

        info!("waiting for DHCP...");
        while !stack.is_config_up() {
            Timer::after_millis(100).await;
        }
        info!("DHCP is now up!");

        let mut ip = [0u8; 4];
        for (n, part) in mqtt_host_ip.split(|&b| b == b'.').enumerate() {
            if n >= ip.len() {
                error!("invalid mqtt host ip format");
                return None;
            }
            let part = str::from_utf8(part).unwrap();
            let part = part.parse::<u8>().unwrap();
            ip[n] = part;
        }
        let address = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
        let remote_endpoint = (address, 1883);

        static RX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
        let rx_buffer = RX_BUFFER.init([0; 4096]);
        static TX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
        let tx_buffer = TX_BUFFER.init([0; 4096]);
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(100))); //was 10
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
                    rust_mqtt::packet::v5::reason_codes::ReasonCode::NetworkError => {
                        error!("MQTT Network Error");
                    }
                    _ => {
                        error!("Other MQTT Error: {:?}", mqtt_error);
                    }
                },
            }
            Timer::after(Duration::from_millis(2000)).await;
        }



        unwrap!(spawner.spawn(ping_task(client_mutexed)));

        Some(Self {
            client_mutexed,
        })
    }

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
