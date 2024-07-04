//! This module handles the communication via USB.
//! It receives telegrams and passes them to :doc:`../app/parser`.
//! It receives the answers from there and sends them out.
//!
//! .. plantuml::
//!
//!    @startuml
//!
//!    UsbCommunication -- Parser
//!
//!    @enduml

use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance};
use embassy_usb::{Builder, Config, UsbDevice};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::class::cdc_acm::Sender;

use app;

// fn init_parser() -> app::Parser<app::EnterBootloader, app::Persistency> {
//     struct EnterBootloaderImpl;
//     impl app::EnterBootloader for EnterBootloaderImpl {
//         fn call(&mut self) {
//             embassy_rp::rom_data::reset_to_usb_boot(0, 0);
//         }
//     }

//     struct PersistencyImpl;
//     impl app::Persistency for PersistencyImpl {
//         fn store_wifi_ssid(&mut self, wifi_ssid: &[u8]) {
//             flash.store_wifi_ssid(wifi_ssid);
//         }
//     }

//     app::Parser::new(EnterBootloaderImpl, PersistencyImpl)
// }

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

pub struct UsbCommunication<'a> {
    pub cdc_acm_class: CdcAcmClass<'a, Driver<'a, USB>>,
    pub usb: UsbDevice<'a, Driver<'a, USB>>,
    //parser: app::Parser<app::EnterBootloader, app::Persistency>,
}

impl<'a> UsbCommunication<'a> {
    pub fn new(usb: USB, state: &'a mut State<'a>) -> Self{
        let mut config = Config::new(0x2E8A, 0x0005); //rpi pico w default vid=0x2E8A and pid=0x0005
        config.manufacturer = Some("github.com/erichstuder");
        config.product = Some("433MHz_to_MQTT");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for windows compatibility.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        static mut DEVICE_DESCRIPTOR_BUF: [u8; 256] = [0; 256];
        static mut CONFIG_DESCRIPTOR_BUF: [u8; 256] = [0; 256];
        static mut BOS_DESCRIPTOR_BUF: [u8; 256] = [0; 256];
        static mut CONTROL_BUF: [u8; 64] = [0; 64];

        #[allow(unknown_lints)] //TODO: all this allow stuff should be removed
        let mut builder = Builder::new(
            Driver::new(usb, Irqs),
            config,
            #[allow(static_mut_refs)]
            unsafe{ &mut DEVICE_DESCRIPTOR_BUF },
            #[allow(static_mut_refs)]
            unsafe { &mut CONFIG_DESCRIPTOR_BUF },
            #[allow(static_mut_refs)]
            unsafe { &mut BOS_DESCRIPTOR_BUF },
            #[allow(static_mut_refs)]
            unsafe { &mut CONTROL_BUF },
        );

        let cdc_acm_class = CdcAcmClass::new(&mut builder, state, 64);
        let usb = builder.build();

        //let parser = init_parser();

        Self {
            cdc_acm_class,
            usb,
            //parser,
        }
    }
}

pub async fn echo<'d, T: Instance + 'd, E: app::EnterBootloader, P: app::Persistency>(data: &[u8], sender: &mut Sender<'d, Driver<'d, T>>, parser: &mut app::Parser<E, P>) -> Result<(), Disconnected> {
    //let mut parser = app::Parser::new(EnterBootloaderImpl, PersistencyImpl);
    let answer = parser.parse_message(data);
    sender.write_packet(answer).await?;
    Ok(())
}
