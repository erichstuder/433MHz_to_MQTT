//! This module receives data from the remote.
//! Besides some platform specific stuff, it aggregates :doc:`../app/buttons`.
//!
//! .. plantuml::
//!
//!    @startuml
//!
//!    RemoteReceiver o-- Buttons
//!
//!    @enduml

use {defmt_rtt as _, panic_probe as _};
use embassy_rp::{gpio, pio};
use embassy_rp::pio::PioPin;
use fixed::traits::ToFixed;

use app::buttons::Buttons;

pub struct RemoteReceiver<'d, T: pio::Instance, const SM: usize> {
    sm: pio::StateMachine<'d, T, SM>,
    buttons: Buttons,
}

impl<'d, T: pio::Instance, const SM: usize> RemoteReceiver<'d, T, SM> {
    pub fn new(pio: &mut pio::Common<'d, T>, mut sm: pio::StateMachine<'d, T, SM>, pin: impl PioPin, buttons: Buttons) -> Self {
        let mut pin = pio.make_pio_pin(pin);
        pin.set_pull(gpio::Pull::None);
        sm.set_pin_dirs(pio::Direction::In, &[&pin]);

        let prg = pio_proc::pio_asm!(
            "startup:"
                "set x 31", // 31 is maximum and sufficient
            "assert_initial_low_pulse:",
                "jmp pin startup"
                "jmp x-- assert_initial_low_pulse",

            "set x 24", // one less than the number of bits to read
            "read_bits:",
                "wait 1 pin 0 [5]",
                "in pins, 1",
                "wait 0 pin 0",
                "jmp x-- read_bits",

            "push",
        );

        let mut cfg = pio::Config::default();
        cfg.set_in_pins(&[&pin]);
        cfg.set_jmp_pin(&pin);
        cfg.fifo_join = pio::FifoJoin::RxOnly;
        cfg.shift_in.direction = pio::ShiftDirection::Left;
        cfg.clock_divider = 12500.to_fixed(); // 125MHz / 12500 = 10kHz
        cfg.use_program(&pio.load_program(&prg.program), &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { sm, buttons }
    }

    pub async fn read(&mut self) -> &[u8]{
        loop {
            let value = self.sm.rx().wait_pull().await;
            if let Some(button) = self.buttons.match_button(value) {
                return button;
            }
        }
    }
}
