//! This is the main file of the firmware.
//! The pieces are set up, connected together and started.

#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::{Spawner, main, task};
use embassy_rp::bind_interrupts;
use embassy_rp::pio::{self, Pio};
use embassy_rp::peripherals::{USB, FLASH, DMA_CH0, PIO0, PIN_28};
use embassy_rp::usb;
use embassy_usb::class::cdc_acm;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;

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
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    let mut usb_communication = UsbCommunication::new(peripherals.USB);

    let (usb_sender_mutexed, usb_receiver) = {
        let (usb_sender, usb_receiver) = usb_communication.cdc_acm_class.split();
        static USB_SENDER: StaticCell<Mutex<CriticalSectionRawMutex, cdc_acm::Sender<usb::Driver<USB>>>> = StaticCell::new();
        (USB_SENDER.init(Mutex::new(usb_sender)), usb_receiver)
    };

    spawner.spawn(handle_buttons(peripherals.PIO0, peripherals.PIN_28, usb_sender_mutexed)).unwrap();
    spawner.spawn(echo(peripherals.FLASH, peripherals.DMA_CH0, usb_receiver, usb_sender_mutexed)).unwrap();

    usb_communication.usb.run().await;
}

#[task]
async fn handle_buttons(pio: PIO0, receiver_pin: PIN_28, usb_sender: &'static Mutex<CriticalSectionRawMutex, cdc_acm::Sender<'static, usb::Driver<'static, USB>>>) {
    let Pio { common: mut pio_common, sm0: pio_sm0, .. } = Pio::new(pio, Irqs);

    let buttons = Buttons::new();

    let mut remote_receiver = RemoteReceiver::new(
        &mut pio_common,
        pio_sm0,
        receiver_pin,
        buttons,
    );

    loop {
        let pressed_button = remote_receiver.read().await;
        {
            let mut sender = usb_sender.lock().await;
            let _ = sender.write_packet(pressed_button).await;
            let _ = sender.write_packet(b"\n").await;
        }
    }
}

#[task]
async fn echo(flash: FLASH, dma_ch0: DMA_CH0, mut usb_receiver: cdc_acm::Receiver<'static, usb::Driver<'static, USB>>, usb_sender: &'static Mutex<CriticalSectionRawMutex, cdc_acm::Sender<'static, usb::Driver<'static, USB>>>) {
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

    let persistency = PersistencyImpl::new(flash, dma_ch0);
    let mut parser = Parser::new(EnterBootloaderImpl, persistency);

    loop {
        usb_receiver.wait_connection().await;
        let n = match usb_receiver.read_packet(&mut buf).await {
            Ok(n) => n,
            Err(_e) => {
                // Handle the error
                continue;
            }
        };
        let data = &buf[..n];
        {
            let mut sender = usb_sender.lock().await;
            let _ = usb_communication::echo(data, &mut sender, &mut parser).await;
        }
    }
}