//! This is the main file of the firmware.
//! The pieces are set up, connected together and started.

#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::{Spawner, main};
use embassy_rp::bind_interrupts;
use embassy_rp::pio::{self, Pio};
use embassy_rp::peripherals::{USB, FLASH, DMA_CH0, PIO0};
use embassy_rp::usb;
use embassy_usb::class::cdc_acm;
use embassy_futures::join::join;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod persistency;
mod remote_receiver;
mod usb_communication;

use app::buttons::Buttons;
use app::parser::{self, Parser};
use persistency::Persistency;
use remote_receiver::RemoteReceiver;
use usb_communication::UsbCommunication;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let Pio { common: mut pio_common, sm0, .. } = Pio::new(peripherals.PIO0, Irqs);

    let buttons = Buttons::new();

    let mut remote_receiver = RemoteReceiver::new(
        &mut pio_common,
        sm0,
        peripherals.PIN_28,
        buttons,
    );

    let usb_communication = UsbCommunication::new(peripherals.USB);

    let mut usb = usb_communication.usb;
    let (sender, mut receiver) = usb_communication.cdc_acm_class.split();
    let sender: Mutex<CriticalSectionRawMutex, cdc_acm::Sender<usb::Driver<USB>>> = Mutex::new(sender);

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
        impl parser::EnterBootloader for EnterBootloaderImpl {
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
        impl parser::Persistency for PersistencyImpl {
            fn store(&mut self, value: &[u8], value_id: parser::ValueId) {
                match value_id {
                    parser::ValueId::WifiSsid           => self.persistency.store(value, persistency::ValueId::WifiSsid),
                    parser::ValueId::WifiPassword       => self.persistency.store(value, persistency::ValueId::WifiPassword),
                    parser::ValueId::MqttHostIp         => self.persistency.store(value, persistency::ValueId::MqttHostIp),
                    parser::ValueId::MqttBrokerUsername => self.persistency.store(value, persistency::ValueId::MqttBrokerUsername),
                    parser::ValueId::MqttBrokerPassword => self.persistency.store(value, persistency::ValueId::MqttBrokerPassword),
                }
            }

            fn read(&mut self, value_id: parser::ValueId) -> &[u8] {
                match value_id {
                    parser::ValueId::WifiSsid           => self.persistency.read(persistency::ValueId::WifiSsid),
                    parser::ValueId::WifiPassword       => self.persistency.read(persistency::ValueId::WifiPassword),
                    parser::ValueId::MqttHostIp         => self.persistency.read(persistency::ValueId::MqttHostIp),
                    parser::ValueId::MqttBrokerUsername => self.persistency.read(persistency::ValueId::MqttBrokerUsername),
                    parser::ValueId::MqttBrokerPassword => self.persistency.read(persistency::ValueId::MqttBrokerPassword),
                }
            }
        }

        let persistency = PersistencyImpl::new(peripherals.FLASH, peripherals.DMA_CH0);
        let mut parser = Parser::new(EnterBootloaderImpl, persistency);

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
