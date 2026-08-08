#![no_main]
#![cfg_attr(not(feature = "host-test"), no_std)]

use embassy_executor::{Spawner, main, task};
use embassy_rp::{
    bind_interrupts,
    pio::{self, Pio},
    peripherals::{DMA_CH0, DMA_CH1, PIO1},
};
use embassy_usb::driver::EndpointError as UsbEndPointError;
use embassy_sync::{
    mutex::Mutex,
    blocking_mutex::raw::CriticalSectionRawMutex,
};
use static_cell::StaticCell;
use defmt::unwrap;
use defmt_rtt as _;
use panic_probe as _;

use lib::{
    parser::{self, Parser},
    persistency::{self, PersistencyTrait},
    terminal::{self, Terminal},
};
use firmware::modules::{
    flash_persistency::{self, FlashPersistency},
    mqtt::{self, MQTT, WifiHw},
    // remote_receiver::RemoteReceiver,
    usb_communication::{self, UsbSender, UsbReceiver},
};

struct EnterBootloader;
impl parser::Command for EnterBootloader {
    fn cmd_str(&self) -> &'static [u8] {
        b"enter bootloader"
    }

    fn run(&self, answer: &mut [u8]) -> Result<usize, &'static str> {
        embassy_rp::rom_data::reset_to_usb_boot(0, 0);

        // Note: Probably this message won't be seen, because of immediate restart.
        let text = b"entering bootloader now";
        answer.copy_from_slice(text);
        Ok(text.len())
    }
}

type MyParser = Parser<ParserActions<'static>, EnterBootloader>;
type MyTerminal = Terminal<TerminalActions, { usb_communication::MAX_PACKET_SIZE as usize }>;

struct TerminalActions {
    usb_sender: UsbSender,
    usb_receiver: UsbReceiver,
    parser: MyParser,
}

impl TerminalActions {
    fn new(usb_sender: UsbSender, usb_receiver: UsbReceiver, parser: MyParser) -> Self {
        Self {
            usb_sender,
            usb_receiver,
            parser
        }
    }
}

impl terminal::Actions for TerminalActions{
    async fn send(&self, data: &[u8]) -> Result<(), terminal::Error> {
        self.usb_sender.send(data).await.unwrap();
        Ok(()) //TODO: proper error handling
    }

    async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, terminal::Error> {
        self.usb_receiver.read_packet(buffer).await.map_err(|e| match e {
            UsbEndPointError::BufferOverflow => terminal::Error::BufferOverflow,
            UsbEndPointError::Disabled => terminal::Error::Disconnected,
        })
    }

    async fn parse_message(&mut self, msg: &[u8], answer: &mut [u8]) -> Result<usize, &'static str>{
        self.parser.parse_message(msg, answer).await
    }
}

type MutexedPersistency = Mutex<CriticalSectionRawMutex, FlashPersistency>;

struct ParserActions<'d> {
    persistency: &'d MutexedPersistency,
}

impl<'d> ParserActions<'d> {
    pub fn new(persistency: &'d MutexedPersistency) -> Self {
        Self {
            persistency,
        }
    }

    fn id_to_key(id: parser::ValueId) -> persistency::Key {
        match id {
            parser::ValueId::WifiSsid => persistency::Key::WifiSsid,
            parser::ValueId::WifiPassword => persistency::Key::WifiPassword,
            parser::ValueId::MqttHostIp => persistency::Key::MqttHostIp,
            parser::ValueId::MqttBrokerUsername => persistency::Key::MqttBrokerUsername,
            parser::ValueId::MqttBrokerPassword => persistency::Key::MqttBrokerPassword,
        }
    }
}

impl<'d> parser::Actions for ParserActions<'d> {
    async fn store(&mut self, id: parser::ValueId, value: &[u8]) -> Result<(), &'static str> {
        let mut p = self.persistency.lock().await;
        let key = Self::id_to_key(id);
        p.store(key, value).await.map_err(|_| "internal storing error")
    }

    async fn get(&mut self, id: parser::ValueId, buffer: &mut [u8]) -> Result<usize, &'static str>  {
        let mut p = self.persistency.lock().await;
        let key = Self::id_to_key(id);

        p.read(key, buffer).await
            .map_err(|_| "internal reading error")?
            .ok_or("value not found")
    }
}


struct MqttActions<'d> {
    persistency: &'d MutexedPersistency,
}

impl<'d> MqttActions<'d> {
    pub fn new(persistency: &'d MutexedPersistency) -> Self {
        Self {
            persistency,
        }
    }
}

impl<'d> mqtt::Actions for MqttActions<'d> {
    async fn get(&mut self, id: mqtt::ValueId, buffer: &mut [u8]) -> Option<usize> {
        let key = match id {
            mqtt::ValueId::WifiSsid => persistency::Key::WifiSsid,
            mqtt::ValueId::WifiPassword => persistency::Key::WifiPassword,
            mqtt::ValueId::MqttHostIp => persistency::Key::MqttHostIp,
            mqtt::ValueId::MqttBrokerUsername => persistency::Key::MqttBrokerUsername,
            mqtt::ValueId::MqttBrokerPassword => persistency::Key::MqttBrokerPassword,
        };
        let mut p = self.persistency.lock().await;
        let result = p.read(key, buffer).await;
        if let Ok(value) = result {
            return value;
        }
        else {
            return None;
        };
    }
}


bind_interrupts!(struct DmaIrq {
    DMA_IRQ_0 =>
        embassy_rp::dma::InterruptHandler<DMA_CH0>,
        embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

bind_interrupts!(struct Pio1Irqs {
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
});

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let (usb_sender, usb_receiver) = usb_communication::create(peripherals.USB, spawner);

    let flash_persistency = flash_persistency::init(peripherals.FLASH, peripherals.DMA_CH0, DmaIrq);
    let mutexed_persistency = MutexedPersistency::new(flash_persistency);
    static MUTEXED_PERSISTENCY: StaticCell<MutexedPersistency> = StaticCell::new();
    let persistency = MUTEXED_PERSISTENCY.init(mutexed_persistency);

    let additional_command = EnterBootloader;

    let parser_actions = ParserActions::new(persistency);
    let parser = Parser::new(parser_actions, additional_command);

    let terminal_actions = TerminalActions::new(usb_sender, usb_receiver, parser);

    let terminal = MyTerminal::new(terminal_actions);

    spawner.spawn(unwrap!(run_terminal(terminal)));

    let pio = Pio::new(peripherals.PIO1, Pio1Irqs);

    let wifi_hw = WifiHw {
        pin_23: peripherals.PIN_23,
        pin_24: peripherals.PIN_24,
        pin_25: peripherals.PIN_25,
        pin_29: peripherals.PIN_29,
        pio_1: pio,
        dma_ch1: peripherals.DMA_CH1,
    };

    let my_mqtt_actions = MqttActions::new(persistency);
    let _mqtt = MQTT::new(my_mqtt_actions, wifi_hw, spawner, DmaIrq).await;
}

#[task]
async fn run_terminal(mut terminal: MyTerminal) -> ! {
    terminal.run().await
}
