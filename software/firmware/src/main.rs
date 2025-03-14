//! This is the main file of the firmware.
//! The pieces are set up, connected together and started.

#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
// use defmt::{unwrap, info};
use embassy_executor::{Spawner, main};
use embassy_rp::bind_interrupts;
use embassy_rp::pio::{self, Pio};
// use embassy_rp::peripherals::{USB, DMA_CH1, PIO0, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::peripherals::{USB, PIO0, PIO1};
use embassy_rp::usb;
// use embassy_rp::gpio;
use embassy_usb::class::cdc_acm;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;
// use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
// use cyw43::JoinOptions;

mod tasks;
mod drivers;

use crate::tasks::button_task;
use crate::tasks::terminal_task;
use crate::tasks::mqtt_task;
use crate::drivers::usb_communication::UsbCommunication;
use crate::drivers::persistency::Persistency;

bind_interrupts!(struct Pio1Irqs {
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
});

type UsbSenderMutexed = Mutex<CriticalSectionRawMutex, cdc_acm::Sender<'static, usb::Driver<'static, USB>>>;
type UsbReceiver = cdc_acm::Receiver<'static, usb::Driver<'static, USB>>;
type PersistencyMutexed = Mutex<CriticalSectionRawMutex, Persistency>;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    let mut usb_communication = UsbCommunication::new(peripherals.USB);

    // Multiple writers to USB, so it is mutexed and made static to be shared between tasks.
    let (usb_sender_mutexed, usb_receiver) = {
        let (usb_sender, usb_receiver) = usb_communication.cdc_acm_class.split();
        static USB_SENDER: StaticCell<UsbSenderMutexed> = StaticCell::new();
        (USB_SENDER.init(Mutex::new(usb_sender)), usb_receiver)
    };

    // Multiple writers to persistency, so it is mutexed and made static to be shared between tasks.
    let persistency_mutexed = {
        let persistency = Persistency::new(peripherals.FLASH, peripherals.DMA_CH0);
        static PERSISTENCY: StaticCell<PersistencyMutexed> = StaticCell::new();
        PERSISTENCY.init(Mutex::new(persistency))
    };

    bind_interrupts!(struct Pio0Irqs {
        PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    });
    let pio = Pio::new(peripherals.PIO0, Pio0Irqs);
    spawner.spawn(button_task::run(pio, peripherals.PIN_28, usb_sender_mutexed)).unwrap();

    spawner.spawn(terminal_task::run(persistency_mutexed, usb_receiver, usb_sender_mutexed)).unwrap();

    // let wifi_hw = WifiHw {
    //     pin_23: peripherals.PIN_23,
    //     pin_24: peripherals.PIN_24,
    //     pin_25: peripherals.PIN_25,
    //     pin_29: peripherals.PIN_29,
    //     pio_1: peripherals.PIO1,
    //     dma_ch1: peripherals.DMA_CH1,
    // };
    //spawner.spawn(mqtt(spawner, wifi_hw)).unwrap();
    spawner.spawn(mqtt_task::run()).unwrap();

    usb_communication.usb.run().await;
}
