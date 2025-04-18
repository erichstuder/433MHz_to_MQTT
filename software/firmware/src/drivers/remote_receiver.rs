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

use cfg_if::cfg_if;
use {defmt_rtt as _, panic_probe as _};

cfg_if! {
    if #[cfg(not(test))] {
        use embassy_rp::{gpio, pio};
        use embassy_rp::pio::PioPin;
        use embassy_rp::pio::program::pio_asm;
        use fixed::traits::ToFixed;
        use app::buttons::Buttons;
    }
}

#[cfg(not(test))]
pub struct RemoteReceiver<'d, PIO: pio::Instance, const SM: usize> {
    pio_sm: pio::StateMachine<'d, PIO, SM>,
    buttons: Buttons,
}

#[cfg(not(test))]
impl<'d, PIO: pio::Instance, const SM: usize> RemoteReceiver<'d, PIO, SM> {
    pub fn new(pio: &mut pio::Common<'d, PIO>, mut pio_sm: pio::StateMachine<'d, PIO, SM>, receiver_pin: impl PioPin, buttons: Buttons) -> Self {
        let mut pin = pio.make_pio_pin(receiver_pin);
        pin.set_pull(gpio::Pull::None);
        pio_sm.set_pin_dirs(pio::Direction::In, &[&pin]);

        let prg = pio_asm!(
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
        pio_sm.set_config(&cfg);
        pio_sm.set_enable(true);
        Self { pio_sm, buttons }
    }

    pub async fn read(&mut self) -> &str{
        loop {
            let value = self.pio_sm.rx().wait_pull().await;
            if let Some(button) = self.buttons.match_button(value) {
                return button
            }
        }
    }
}
