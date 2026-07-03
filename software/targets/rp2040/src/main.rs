#![no_main]
#![cfg_attr(not(feature = "host-test"), no_std)]
#![deny(unsafe_code)]

use embassy_executor::{Spawner, main, task};
use embassy_rp as _;
use embassy_usb::driver::EndpointError as UsbEndPointError;
use static_cell::StaticCell;
use defmt::unwrap;
use defmt_rtt as _;
use panic_probe as _;

use lib::parser::{self, Parser};
use lib::terminal::{self, Terminal};
use firmware::modules::{
    usb_communication::{self, UsbSender, UsbReceiver},
    flash_persistency::{self, FlashPersistency},
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

type MyParser = Parser<'static, FlashPersistency, EnterBootloader>;
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

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let (usb_sender, usb_receiver) = usb_communication::create(peripherals.USB, spawner);

    static PERSISTENCY: StaticCell<FlashPersistency> = StaticCell::new();
    let persistency = PERSISTENCY.init(flash_persistency::init(peripherals.FLASH, peripherals.DMA_CH0));

    let additional_command = EnterBootloader;

    let parser = Parser::new(persistency, additional_command);

    let terminal_actions = TerminalActions::new(usb_sender, usb_receiver, parser);

    let terminal = MyTerminal::new(terminal_actions);

    spawner.spawn(unwrap!(run_terminal(terminal)));
}

#[task]
async fn run_terminal(mut terminal: MyTerminal) -> ! {
    terminal.run().await
}
