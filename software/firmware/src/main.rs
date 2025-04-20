//! This is the main file of the firmware.
//! The pieces are set up, connected and the tasks are spawned.
//!
//! Following is just a simple example of how to use plantuml to generate diagrams.
//! I had this idea but these diagrams rot quickly. So they won't be used.
//!
//! .. plantuml::
//!
//!    @startuml
//!
//!    module1 *-- module2
//!    module1 *-- module3
//!
//!    @enduml

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use cfg_if::cfg_if;
use {defmt_rtt as _, panic_probe as _};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod tasks;
mod drivers;

use crate::drivers::persistency::Persistency;

type PersistencyMutexed = Mutex<CriticalSectionRawMutex, Persistency>;

cfg_if! {
    if #[cfg(not(test))] {
        use embassy_executor::{Spawner, main};
        use embassy_rp::bind_interrupts;
        use embassy_rp::pio::{self, Pio};
        use embassy_rp::peripherals::USB;
        use embassy_rp::peripherals::{PIO0, PIO1};
        use embassy_rp::usb;
        use embassy_usb::class::cdc_acm;
        use static_cell::StaticCell;

        use crate::tasks::button_task;
        use crate::tasks::terminal_task;
        use crate::drivers::mqtt::{MQTT, WifiHw};
        use crate::drivers::usb_communication::UsbCommunication;

        type UsbSenderMutexed = Mutex<CriticalSectionRawMutex, cdc_acm::Sender<'static, usb::Driver<'static, USB>>>;
        type UsbReceiver = cdc_acm::Receiver<'static, usb::Driver<'static, USB>>;
    }
}


#[cfg(not(test))]
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

    spawner.spawn(terminal_task::run(persistency_mutexed, usb_receiver, usb_sender_mutexed)).unwrap();

    bind_interrupts!(struct Pio1Irqs {
        PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    });
    let pio = Pio::new(peripherals.PIO1, Pio1Irqs);
    let wifi_hw = WifiHw {
        pin_23: peripherals.PIN_23,
        pin_24: peripherals.PIN_24,
        pin_25: peripherals.PIN_25,
        pin_29: peripherals.PIN_29,
        pio_1: pio,
        dma_ch1: peripherals.DMA_CH1,
    };

    let mqtt = MQTT::new(persistency_mutexed, wifi_hw, spawner).await.unwrap();

    bind_interrupts!(struct Pio0Irqs {
        PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    });
    let pio = Pio::new(peripherals.PIO0, Pio0Irqs);
    spawner.spawn(button_task::run(pio, peripherals.PIN_28, usb_sender_mutexed, mqtt)).unwrap();

    usb_communication.usb.run().await;
}
