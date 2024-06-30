#![no_std]
#![no_main]

mod usb_communication;
mod remote_receiver;
mod persistency;

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Pio, InterruptHandler};
use embassy_usb::class::cdc_acm::State;
use embassy_futures::join::join;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::class::cdc_acm::Sender;
use embassy_rp::peripherals::{USB, FLASH, DMA_CH0};
use embassy_rp::usb::Driver;

use app::DataField;
use usb_communication::UsbCommunication;
use persistency::Persistency;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let buttons = app::Buttons::new();
    let mut remote_receiver = remote_receiver::RemoteReceiver::new(
        &mut common,
        sm0,
        p.PIN_28,
        buttons,
    );

    let mut state = State::new();
    let usb_communication = UsbCommunication::new(p.USB, &mut state);

    let mut usb = usb_communication.usb;
    let (sender, mut receiver) = usb_communication.cdc_acm_class.split();
    let sender: Mutex<CriticalSectionRawMutex, Sender<Driver<USB>>> = Mutex::new(sender);

    let receiver_fut = async {
        loop {
            let pressed_button = remote_receiver.read().await;
            {
                let mut sender = sender.lock().await;
                let _ = sender.write_packet(pressed_button).await;
                let _ = sender.write_packet(b"\n").await;
            }
        }
    };

    let echo_fut = async {
        let mut buf = [0; 64];

        struct EnterBootloaderImpl;
        impl app::EnterBootloader for EnterBootloaderImpl {
            fn call(&mut self) {
                embassy_rp::rom_data::reset_to_usb_boot(0, 0);
            }
        }

        struct PersistencyImpl {
            persistency: Persistency,
        }
        impl PersistencyImpl {
            fn new(flash: FLASH, dma_ch0: DMA_CH0) -> Self {
                Self {
                    persistency: Persistency::new(flash, dma_ch0),
                }
            }
        }
        impl app::Persistency for PersistencyImpl {
            fn store(&mut self, value: &[u8], field: DataField) {
                match field {
                    DataField::WifiSsid => self.persistency.store(value, persistency::Field::WifiSsid),
                    DataField::WifiPassword => self.persistency.store(value, persistency::Field::WifiPassword),
                    DataField::MqttHostIp => self.persistency.store(value, persistency::Field::MqttHostIp),
                    DataField::MqttBrokerUsername => self.persistency.store(value, persistency::Field::MqttBrokerUsername),
                    DataField::MqttBrokerPassword => self.persistency.store(value, persistency::Field::MqttBrokerPassword),
                }
            }

            fn read(&mut self, field: DataField) -> &[u8] {
                match field {
                    DataField::WifiSsid => self.persistency.read(persistency::Field::WifiSsid),
                    DataField::WifiPassword => self.persistency.read(persistency::Field::WifiPassword),
                    DataField::MqttHostIp => self.persistency.read(persistency::Field::MqttHostIp),
                    DataField::MqttBrokerUsername => self.persistency.read(persistency::Field::MqttBrokerUsername),
                    DataField::MqttBrokerPassword => self.persistency.read(persistency::Field::MqttBrokerPassword),
                }
            }
        }

        let persistency = PersistencyImpl::new(p.FLASH, p.DMA_CH0);
        let mut parser = app::Parser::new(EnterBootloaderImpl, persistency);

        loop {
            receiver.wait_connection().await;
            let n = match receiver.read_packet(&mut buf).await {
                Ok(n) => n,
                Err(_e) => {
                    // Handle the error
                    continue;
                }
            };
            let data = &buf[..n];
            {
                let mut sender = sender.lock().await;
                let _ = usb_communication::echo(data, &mut sender, &mut parser).await;
            }
        }
    };
    join(usb.run(), join(echo_fut, receiver_fut)).await;
}
