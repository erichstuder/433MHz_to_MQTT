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

mod tasks;
mod drivers;

cfg_if! {
    if #[cfg(not(test))] {
        use embassy_executor::{Spawner, main};
        use embassy_rp::bind_interrupts;
        use embassy_rp::pio::{self, Pio};
        use embassy_rp::peripherals::{PIO0, PIO1};
        use static_cell::StaticCell;

        use crate::tasks::button_task;
        use crate::tasks::terminal;
        use crate::drivers::mqtt::{MQTT, WifiHw};
        use crate::drivers::usb_communication::{self, UsbSender};
        use crate::drivers::persistency::Persistency;
        use crate::drivers::parser::Parser;
    }
}

#[cfg(not(test))]
#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let (usb_receiver, usb_sender) = usb_communication::create(peripherals.USB, spawner);
    static USB_SENDER: StaticCell<UsbSender> = StaticCell::new();
    let usb_sender = USB_SENDER.init(usb_sender);

    static PERSISTENCY: StaticCell<Persistency> = StaticCell::new();
    let persistency = PERSISTENCY.init(Persistency::new(peripherals.FLASH, peripherals.DMA_CH0));

    let parser = Parser::new(persistency);

    spawner.spawn(terminal::run(usb_receiver, usb_sender, parser)).unwrap();

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

    let mqtt = MQTT::new(persistency, wifi_hw, spawner).await.unwrap();

    bind_interrupts!(struct Pio0Irqs {
        PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    });
    let pio = Pio::new(peripherals.PIO0, Pio0Irqs);
    spawner.spawn(button_task::run(pio, peripherals.PIN_28, usb_sender, mqtt)).unwrap();
}
