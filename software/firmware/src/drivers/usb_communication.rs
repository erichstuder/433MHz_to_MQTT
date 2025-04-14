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
use embassy_rp::usb;
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{self, CdcAcmClass};
use embassy_usb::driver::EndpointError as UsbEndpointError;
use static_cell::StaticCell;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

type UsbDriver = usb::Driver<'static, USB>;

pub struct UsbDisconnected {}

impl From<UsbEndpointError> for UsbDisconnected {
    fn from(val: UsbEndpointError) -> Self {
        match val {
            UsbEndpointError::BufferOverflow => panic!("Buffer overflow"),
            UsbEndpointError::Disabled => UsbDisconnected {},
        }
    }
}

pub struct UsbCommunication {
    pub cdc_acm_class: CdcAcmClass<'static, UsbDriver>,
    pub usb: UsbDevice<'static, UsbDriver>,
}

pub const MAX_PACKET_SIZE: u8 = 64;

impl UsbCommunication {
    pub fn new(usb: USB) -> Self{
        let mut config = embassy_usb::Config::new(0x2E8A, 0x0005); //rpi pico w default vid=0x2E8A and pid=0x0005
        config.manufacturer = Some("github.com/erichstuder");
        config.product = Some("433MHz_to_MQTT");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = MAX_PACKET_SIZE;

        // Required for windows compatibility.
        // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config.composite_with_iads = true;

        static DEVICE_DESCRIPTOR_BUF: StaticCell<[u8; 256]> = StaticCell::new();
        let device_descriptor_buf = DEVICE_DESCRIPTOR_BUF.init([0; 256]);

        static CONFIG_DESCRIPTOR_BUF: StaticCell<[u8; 256]> = StaticCell::new();
        let config_descriptor_buf = CONFIG_DESCRIPTOR_BUF.init([0; 256]);

        static BOS_DESCRIPTOR_BUF: StaticCell<[u8; 256]> = StaticCell::new();
        let bos_descriptor_buf = BOS_DESCRIPTOR_BUF.init([0; 256]);

        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
        let control_buf = CONTROL_BUF.init([0; 64]);

        let mut builder = embassy_usb::Builder::new(
            usb::Driver::new(usb, Irqs),
            config,
            device_descriptor_buf,
            config_descriptor_buf,
            bos_descriptor_buf,
            control_buf,
        );

        static CDC_ACM_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
        let cdc_acm_state = CDC_ACM_STATE.init(cdc_acm::State::new());
        let cdc_acm_class = CdcAcmClass::new(&mut builder, cdc_acm_state, 64);

        let usb = builder.build();

        Self {
            cdc_acm_class,
            usb,
        }
    }
}
