//! Sets up and handles the MQTT connection.

use core::str;
use core::marker::PhantomData;
use defmt::{Format, info, error, unwrap};
use embassy_executor::{task, Spawner};
use embassy_rp::{Peri, gpio, dma};

// use embassy_time::{Duration, Timer};
use embassy_time::Timer;

use embassy_net;
use embassy_rp::clocks::RoscRng;
use embassy_rp::pio::Pio;
use embassy_rp::peripherals::{DMA_CH1, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
use static_cell::StaticCell;
use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
use cyw43::{aligned_bytes, JoinOptions};
use core::net::Ipv4Addr;
use rust_mqtt::types::{MqttString, MqttBinary};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use lib::misc::parse_ip;

type MqttClient<'a> = rust_mqtt::client::Client<'a, embassy_net::tcp::TcpSocket<'a>, rust_mqtt::buffer::BumpBuffer<'a>, 8, 16, 16, 4>;

#[derive(Copy, Clone, Format)]
pub enum ValueId {
    WifiSsid,
    WifiPassword,
    MqttHostIp,
    MqttBrokerUsername,
    MqttBrokerPassword,
}

// This struct serves as a workaround for the problem that tasks cannot have generic parameters.
// For this reason the concept with Actions does not work here and the config must be updated from externally.
pub struct Config {
// TODO: sollen die member puglic sein?
    pub wifi_ssid: [u8; 32],
    pub wifi_ssid_len: Option<usize>,
    pub wifi_password: [u8; 32],
    pub wifi_password_len: Option<usize>,
    pub mqtt_host_ip: [u8; 32],
    pub mqtt_host_ip_len: Option<usize>,
    pub mqtt_broker_username: [u8; 32],
    pub mqtt_broker_username_len: Option<usize>,
    pub mqtt_broker_password: [u8; 64],
    pub mqtt_broker_password_len: Option<usize>,
}

impl Config {
    fn get_value(&self, id: ValueId, buffer: &mut [u8]) -> Option<usize> {
        match id {
            ValueId::WifiSsid => self.wifi_ssid_len.map(|len| {
                buffer[..len].copy_from_slice(&self.wifi_ssid[..len]);
                len
            }),
            ValueId::WifiPassword => self.wifi_password_len.map(|len| {
                buffer[..len].copy_from_slice(&self.wifi_password[..len]);
                len
            }),
            ValueId::MqttHostIp => self.mqtt_host_ip_len.map(|len| {
                buffer[..len].copy_from_slice(&self.mqtt_host_ip[..len]);
                len
            }),
            ValueId::MqttBrokerUsername => self.mqtt_broker_username_len.map(|len| {
                buffer[..len].copy_from_slice(&self.mqtt_broker_username[..len]);
                len
            }),
            ValueId::MqttBrokerPassword => self.mqtt_broker_password_len.map(|len| {
                buffer[..len].copy_from_slice(&self.mqtt_broker_password[..len]);
                len
            }),
        }
    }
}

static CONFIG: Mutex<CriticalSectionRawMutex, Config> = Mutex::new(Config {
    wifi_ssid: [0u8; 32],
    wifi_ssid_len: None,
    wifi_password: [0u8; 32],
    wifi_password_len: None,
    mqtt_host_ip: [0u8; 32],
    mqtt_host_ip_len: None,
    mqtt_broker_username: [0u8; 32],
    mqtt_broker_username_len: None,
    mqtt_broker_password: [0u8; 64],
    mqtt_broker_password_len: None,
});

pub struct WifiHw<'d> {
    pub pin_23: Peri<'d, PIN_23>,
    pub pin_24: Peri<'d, PIN_24>,
    pub pin_25: Peri<'d, PIN_25>,
    pub pin_29: Peri<'d, PIN_29>,
    pub pio_1: Pio<'static, PIO1>, // TODO: warum ist nur das 'static?
    pub dma_ch1: Peri<'d, DMA_CH1>,
}

#[derive(Copy, Clone)]
struct MqttPayload {
    payload: [u8; 128],
    len: usize,
}

type MessageChannel = Channel<CriticalSectionRawMutex, MqttPayload, 4>;

pub struct MQTT<I> {
    message_channel: &'static MessageChannel,
    _phantom_data: PhantomData<I>,
}

impl<I> MQTT<I>
where
    I: embassy_rp::interrupt::typelevel::Binding<embassy_rp::interrupt::typelevel::DMA_IRQ_0, embassy_rp::dma::InterruptHandler<DMA_CH1>> + 'static,
{
    pub async fn new(hw: WifiHw<'static>, spawner: Spawner, irq: I) -> Self {
        let (driver, control) = Self::setup_cyw43(hw, spawner, irq).await;
        //Self::connect_wifi(&mut actions, control, network_stack).await;
        // let client_mutexed = Self::connect_broker(&mut actions, network_stack, spawner).await;

        let address = Ipv4Addr::new(1, 2, 3, 4); // TODO: use right values
        let remote_endpoint = (address, 1883u16);

        let mqtt_connect_options = rust_mqtt::client::options::ConnectOptions::new()
            .clean_start()
            .session_expiry_interval(rust_mqtt::config::SessionExpiryInterval::NeverEnd)
            .keep_alive(rust_mqtt::config::KeepAlive::Infinite)
            .user_name(unwrap!(MqttString::from_str(str::from_utf8(b"broker_user_name").unwrap())))
            .password(unwrap!(MqttBinary::from_slice(b"broker_password")));

        static MESSAGE_CHANNEL: MessageChannel = Channel::new();
        spawner.spawn(run(spawner, remote_endpoint, mqtt_connect_options, driver, control, &MESSAGE_CHANNEL).unwrap());

        Self {
            message_channel: &MESSAGE_CHANNEL,
            _phantom_data: PhantomData,
        }
    }

    async fn setup_cyw43(mut hw: WifiHw<'static>, spawner: Spawner, irq: I) -> (cyw43::NetDriver<'static>, cyw43::Control<'static>) {
        let firmware = aligned_bytes!("../../../../cyw43-firmware/43439A0.bin");
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
        let (driver, mut control, runner) = cyw43::new(cyw43_state, pwr, spi, firmware, nvram).await;
        spawner.spawn(cyw43_task(runner).unwrap());

        control.init(clm).await;
        control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

        (driver, control)
    }

    // async fn connect_wifi(actions: &mut A, mut control: cyw43::Control<'static>, network_stack: embassy_net::Stack<'static>) {
    //     let mut wifi_ssid = [0u8; 32];
    //     let mut wifi_password = [0u8; 32];

    //     loop {
    //         let wifi_ssid_len = Self::get_valid_value(actions, ValueId::WifiSsid, &mut wifi_ssid).await;
    //         let wifi_password_len = Self::get_valid_value(actions, ValueId::WifiPassword, &mut wifi_password).await;
    //         match control.join(
    //             str::from_utf8(&wifi_ssid[..wifi_ssid_len]).unwrap(),
    //             JoinOptions::new(&wifi_password[..wifi_password_len])
    //         ).await {
    //             Ok(_) => {
    //                 info!("join successful");
    //                 break
    //             },
    //             Err(err) => {
    //                 // Leave immediately to prevent panic.
    //                 control.leave().await;
    //                 info!("join failed with status={:?}", err);
    //             },
    //         }
    //     }

    //     info!("waiting for DHCP...");
    //     while !network_stack.is_config_up() {
    //         Timer::after_millis(100).await;
    //     }
    //     info!("DHCP is now up!");
    // }

    // async fn connect_broker<'d>(actions: &mut A, network_stack: embassy_net::Stack<'static>, spawner: Spawner) -> &'static MqttClientMutexed<'static> {
    //     let mut mqtt_host_ip = [0u8; 32];
    //     let mut mqtt_broker_username = [0u8; 32];
    //     let mut mqtt_broker_password = [0u8; 64];

    //     let mqtt_host_ip_len = Self::get_valid_value(actions, ValueId::MqttHostIp, &mut mqtt_host_ip).await;
    //     let (ip0, ip1, ip2, ip3) = parse_ip(&mqtt_host_ip[..mqtt_host_ip_len]).unwrap();
    //     let address = Ipv4Addr::new(ip0, ip1, ip2, ip3);
    //     let remote_endpoint = (address, 1883u16);
    //     let mqtt_broker_username_len = Self::get_valid_value(actions, ValueId::MqttBrokerUsername, &mut mqtt_broker_username).await;
    //     let mqtt_broker_password_len = Self::get_valid_value(actions, ValueId::MqttBrokerPassword, &mut mqtt_broker_password).await;
    //     let mqtt_connect_options = rust_mqtt::client::options::ConnectOptions::new()
    //         .clean_start()
    //         .session_expiry_interval(rust_mqtt::config::SessionExpiryInterval::NeverEnd)
    //         .keep_alive(rust_mqtt::config::KeepAlive::Infinite)
    //         .user_name(unwrap!(MqttString::from_str(str::from_utf8(&mqtt_broker_username[..mqtt_broker_username_len]).unwrap())))
    //         .password(unwrap!(MqttBinary::from_slice(&mqtt_broker_password[..mqtt_broker_password_len])));

    //     static MQTT_BUMP_MEM: StaticCell<[u8; 2048]> = StaticCell::new();
    //     static MQTT_BUMP: StaticCell<rust_mqtt::buffer::BumpBuffer<'static>> = StaticCell::new();
    //     let bump_mem = MQTT_BUMP_MEM.init([0; 2048]);
    //     let bump = MQTT_BUMP.init(rust_mqtt::buffer::BumpBuffer::new(bump_mem));
    //     let mut client = rust_mqtt::client::Client::new(bump);

    //     // return client;

    //     // const BUFFER_SIZE: usize = 2048;
    //     // static RX_BUFFER: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    //     // static TX_BUFFER: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    //     // let rx_buffer = RX_BUFFER.init([0; BUFFER_SIZE]);
    //     // let tx_buffer = TX_BUFFER.init([0; BUFFER_SIZE]);

    //     loop {
    //         const BUFFER_SIZE: usize = 2048;
    //         let mut rx_buffer = [0; BUFFER_SIZE];
    //         let mut tx_buffer = [0; BUFFER_SIZE];

    //         let mut socket = embassy_net::tcp::TcpSocket::new(network_stack, &mut rx_buffer, &mut tx_buffer);
    //         socket.set_timeout(Some(embassy_time::Duration::from_secs(100)));

    //         loop {
    //             if let Err(e) = socket.connect(remote_endpoint).await {
    //                 info!("connect error: {:?}", e);
    //                 Timer::after_millis(1000).await;
    //                 continue
    //             }
    //             break
    //         };

    //         match client.connect(socket, &mqtt_connect_options, Some(MqttString::from_str("433MHz_to_MQTT").unwrap())).await {
    //             Ok(info) => {
    //                 info!("Connected to broker with: {:?}", info);
    //                 break;
    //             }
    //             Err(e) =>  {
    //                 client.abort().await;
    //                 error!("Other MQTT Error: {:?}", e);
    //                 Timer::after_millis(1000).await;
    //                 continue;
    //             }
    //         }
    //     }

    //     static CLIENT_MUTEXED: StaticCell<MqttClientMutexed> = StaticCell::new();
    //     let client_mutexed = CLIENT_MUTEXED.init(Mutex::new(client));
    //     return client_mutexed
    // }

    pub async fn set_config(&mut self, config: Config) {
        let mut cfg = CONFIG.lock().await;
        *cfg = config;
    }

    pub async fn send_message(&mut self, payload: &[u8]) {
        let mut msg = MqttPayload {
            payload: [0; 128],
            len: payload.len().min(128),
        };
        msg.payload[..msg.len].copy_from_slice(&payload[..msg.len]);
        self.message_channel.send(msg).await;
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
async fn run(
    spawner: Spawner,
    remote_endpoint: (Ipv4Addr, u16),
    mqtt_connect_options: rust_mqtt::client::options::ConnectOptions<'static>,
    driver: cyw43::NetDriver<'static>,
    mut control: cyw43::Control<'static>,
    message_channel: &'static MessageChannel,
) -> ! {
    let network_stack = setup_network(driver, spawner);

    let mut wifi_ssid = [0u8; 32];
    let mut wifi_password = [0u8; 32];

    loop {
        let wifi_ssid_len = get_valid_value(ValueId::WifiSsid, &mut wifi_ssid).await;
        let wifi_password_len = get_valid_value(ValueId::WifiPassword, &mut wifi_password).await;
        match control.join(
            str::from_utf8(&wifi_ssid[..wifi_ssid_len]).unwrap(),
            JoinOptions::new(&wifi_password[..wifi_password_len])
        ).await {
            Ok(_) => {
                info!("join successful");
                break
            },
            Err(err) => {
                // Leave immediately to prevent panic.
                control.leave().await;
                info!("join failed with status={:?}", err);
            },
        }
    }

    info!("waiting for DHCP...");
    while !network_stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

////


    const BUFFER_SIZE: usize = 2048;
    static RX_BUFFER: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    static TX_BUFFER: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
    let rx_buffer = RX_BUFFER.init([0; BUFFER_SIZE]);
    let tx_buffer = TX_BUFFER.init([0; BUFFER_SIZE]);

    loop {
        let mut bump_mem = [0; 2048];
        let mut bump = rust_mqtt::buffer::BumpBuffer::new(&mut bump_mem);
        let mut client = MqttClient::new(&mut bump);

        let mut socket = embassy_net::tcp::TcpSocket::new(network_stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(100)));

        loop {
            if let Err(e) = socket.connect(remote_endpoint).await {
                info!("connect error: {:?}", e);
                Timer::after_millis(1000).await;
                continue
            }
            break
        };

        match client.connect(socket, &mqtt_connect_options, Some(MqttString::from_str("433MHz_to_MQTT").unwrap())).await {
            Ok(info) => {
                info!("Connected to broker with: {:?}", info);
                //break;
            }
            Err(e) =>  {
                error!("Other MQTT Error: {:?}", e);
                client.abort().await;
                continue;
                // Timer::after_millis(1000).await;
            }
        }

        loop {
            let msg = message_channel.receive().await;
            let topic = unwrap!(rust_mqtt::types::TopicName::new(unwrap!(MqttString::from_str("433MHz_to_MQTT_button"))));
            let payload = &msg.payload[..msg.len];
            let publish_result = client.publish(
                &rust_mqtt::client::options::PublicationOptions::new(rust_mqtt::client::options::TopicReference::Name(topic.as_borrowed())).exactly_once(),
                payload.into(),
            ).await;

            if let Err(e) = publish_result {
                error!("MQTT publish failed: {:?}", e);
                client.abort().await;
                break;
            }
        }
    }

    // static CLIENT_MUTEXED: StaticCell<MqttClientMutexed> = StaticCell::new();
    // let client_mutexed = CLIENT_MUTEXED.init(Mutex::new(client));
    // return client_mutexed
}

fn setup_network(driver: cyw43::NetDriver<'static>, spawner: Spawner) -> embassy_net::Stack<'static>{
    let config = embassy_net::Config::dhcpv4(Default::default());
    let mut rng = RoscRng;
    let seed = rng.next_u64(); // TODO: dont know why the seed is important. couldn't it be a constant?
    static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
    let resources = RESOURCES.init(embassy_net::StackResources::new());
    let (network_stack, network_runner) = embassy_net::new(driver, config, resources, seed);
    spawner.spawn(net_task(network_runner).unwrap());
    network_stack
}

// As it often makes no sense to advance if there is no valid value, we just loop until we get a valid value.
// This helps on first startup when no values might be stored yet.
// As soon as a value is available we can advance.
async fn get_valid_value(id: ValueId, buffer: &mut [u8]) -> usize {
    loop {
        let c = CONFIG.lock().await;
        if let Some(result) = c.get_value(id, buffer) {
            return result
        }
        info!("Couldn't get {:?}", id);
        Timer::after_millis(3000).await;
    }
}



#[cfg(feature = "target-test")]
#[embedded_test::tests]
mod tests {
    use super::*;
    use crate::modules::test_setup::{Pio1Irqs, DmaIrqs};

    // Note: For the moment we do more of a dummy test. One day we might setup a runner that hosts a WiFi network with a MQTT broker.
    #[test]
    async fn test() {
        let peripherals = embassy_rp::init(Default::default());

        let pio = Pio::new(peripherals.PIO1, Pio1Irqs);

        let hw = WifiHw {
            pin_23: peripherals.PIN_23,
            pin_24: peripherals.PIN_24,
            pin_25: peripherals.PIN_25,
            pin_29: peripherals.PIN_29,
            pio_1: pio,
            dma_ch1: peripherals.DMA_CH1,
        };

        let spawner = unsafe{ Spawner::for_current_executor() }.await;

        struct MockActions;
        impl Actions for MockActions {
            async fn get(&mut self, _id: ValueId, _buffer: &mut [u8]) -> usize {0}
        }

        let (driver, _control) = MQTT::<MockActions, DmaIrqs>::setup_cyw43(hw, spawner, DmaIrqs).await;
        let _network_stack = MQTT::<MockActions, DmaIrqs>::setup_network(driver, spawner);
    }
}
