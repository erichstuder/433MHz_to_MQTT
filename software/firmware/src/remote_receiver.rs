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

use embassy_rp::gpio::Pull;
use embassy_rp::pio;
use fixed::traits::ToFixed;
use embassy_rp::pio::{Common, Config, FifoJoin, Instance, PioPin, ShiftDirection, StateMachine};
use {defmt_rtt as _, panic_probe as _};

use app::Buttons;

pub struct RemoteReceiver<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
    buttons: Buttons,
}

impl<'d, T: Instance, const SM: usize> RemoteReceiver<'d, T, SM> {
    pub fn new(pio: &mut Common<'d, T>, mut sm: StateMachine<'d, T, SM>, pin: impl PioPin, buttons: Buttons) -> Self {
        let mut pin = pio.make_pio_pin(pin);
        pin.set_pull(Pull::None);
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

        let mut cfg = Config::default();
        cfg.set_in_pins(&[&pin]);
        cfg.set_jmp_pin(&pin);
        cfg.fifo_join = FifoJoin::RxOnly;
        cfg.shift_in.direction = ShiftDirection::Left;
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
