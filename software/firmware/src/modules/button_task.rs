//! Reads button presses and sends them out.

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(test))] {
        use embassy_executor::task;
        use embassy_rp::pio::Pio;
        use embassy_rp::peripherals::{PIO0, PIN_28};

        use crate::modules::remote_receiver::RemoteReceiver;
        use crate::modules::mqtt::MQTT;
        use crate::modules::usb_communication::UsbSender;
        use crate::modules::buttons::Buttons;
    }
}

#[cfg(not(test))]
#[task]
pub async fn run(mut pio: Pio<'static, PIO0>, receiver_pin: PIN_28, _usb_sender: &'static UsbSender, mut mqtt: MQTT) {
    // It would be nice to have generic types for pio and receiver_pin but I couldn't figure out how to do it.

    let buttons = Buttons::new();

    let mut remote_receiver = RemoteReceiver::new(
        &mut pio.common,
        pio.sm0,
        receiver_pin,
        buttons,
    );

    loop {
        let pressed_button = remote_receiver.read().await;

        mqtt.send_message(pressed_button.as_bytes()).await;

        // It can be helpful to have the pressed button printed to the console for debugging.
        // But this blocks forever if no terminal is connected.
        // let mut sender = usb_sender.lock().await;
        // let _ = sender.write_packet(pressed_button.as_bytes()).await;
        // let _ = sender.write_packet(b"\n").await;
    }
}
