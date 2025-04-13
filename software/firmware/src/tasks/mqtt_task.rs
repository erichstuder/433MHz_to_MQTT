use defmt::{unwrap, info, error};
use embassy_executor::{task, Spawner};
use embassy_rp::pio::Pio;
use embassy_rp::peripherals::{DMA_CH1, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::gpio;
use static_cell::StaticCell;
use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
use cyw43::JoinOptions;
use core::net::Ipv4Addr;
use core::str;
use embassy_net;
use embassy_rp::clocks::RoscRng;
use rand_core::RngCore; // Don't know why this is needed. Is it because the 'use' is missing in embassy_rp::clocks::RoscRng?
//use minimq::{ConfigBuilder, Minimq, Publication};
// use minimq::Minimq;
// use embedded_nal_async;
// use embedded_nal_async;
use rust_mqtt::client::client::MqttClient;
// use embassy_net:
use embassy_time::{Duration, Timer};

// use minimq::embedded_nal::
// use embedded_nal_async;

use crate::drivers::persistency;
use crate::PersistencyMutexed;

pub struct WifiHw {
    pub pin_23: PIN_23,
    pub pin_24: PIN_24,
    pub pin_25: PIN_25,
    pub pin_29: PIN_29,
    pub pio_1: Pio<'static, PIO1>,
    pub dma_ch1: DMA_CH1,
}

#[task]
pub async fn run(persistency: &'static PersistencyMutexed, mut hw: WifiHw, spawner: Spawner) {
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
            return;
        }
    };
    let wifi_ssid = str::from_utf8(&wifi_ssid[..length]).unwrap();

    let mut wifi_password: [u8; 32] = ['\0' as u8; 32];
    let length = match persistency.lock().await.read(persistency::ValueId::WifiPassword, &mut wifi_password) {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read Wifi Password: {:?}", e);
            return;
        }
    };
    let wifi_password = &wifi_password[..length];

    let mut mqtt_host_ip: [u8; 32] = ['\0' as u8; 32];
    let length = match persistency.lock().await.read(persistency::ValueId::MqttHostIp, &mut mqtt_host_ip) {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read MQTT Host IP: {:?}", e);
            return;
        }
    };
    let _mqtt_host_ip = &mqtt_host_ip[..length];

    let mut mqtt_broker_username: [u8; 32] = ['\0' as u8; 32];
    let length = match persistency.lock().await.read(persistency::ValueId::MqttBrokerUsername, &mut mqtt_broker_username) {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read MQTT Broker Username: {:?}", e);
            return;
        }
    };
    let mqtt_broker_username = &mqtt_broker_username[..length];

    let mut mqtt_broker_password: [u8; 32] = ['\0' as u8; 32];
    let length = match persistency.lock().await.read(persistency::ValueId::MqttBrokerPassword, &mut mqtt_broker_password) {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read MQTT Broker Password: {:?}", e);
            return;
        }
    };
    let _mqtt_broker_password = &mqtt_broker_password[..length];

    info!("ssid: {:?}", wifi_ssid);
    info!("password: {:?}", str::from_utf8(wifi_password).unwrap());

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


    //mqtt

    // broker: 192.168.1.105
    let address = Ipv4Addr::new(192, 168, 1, 105);
    let remote_endpoint = (address, 1883);


    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(100))); //was 10
    let mut recv_buffer = [0; 150]; //was 80
    let mut write_buffer = [0; 150]; //was 80

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
    config.add_username(str::from_utf8(mqtt_broker_username).unwrap());
    config.add_password("myPassword");
    config.max_packet_size = 150; //was 100


    let mut client = MqttClient::<_, 5, _>::new(  //was 5
        socket,
        &mut write_buffer,
        150,
        &mut recv_buffer,
        150,
        config,
    );

    loop {
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

    client.send_message("433", "hello from down here".as_bytes(), rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1, false).await.unwrap();
    info!("message sent");





    // minimq::embedded_nal::TcpClientStack::
    // let mut mqtt: Minimq<'_, _, _, minimq::broker::IpBroker> = Minimq::new(
    //     stack,
    //     embedded-time::Clock::default(),
    //     ConfigBuilder::new(localhost.into(), &mut buffer).client_id("test").unwrap(),
    // );

}

#[task]
async fn cyw43_task(runner: cyw43::Runner<'static, gpio::Output<'static>, cyw43_pio::PioSpi<'static, PIO1, 0, DMA_CH1>>) -> ! {
    runner.run().await
}

#[task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}
