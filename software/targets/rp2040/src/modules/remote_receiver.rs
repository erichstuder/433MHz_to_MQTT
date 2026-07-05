//! Receives data from the remote.

use embassy_rp::{gpio, pio};
use embassy_rp::pio::PioPin;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::Peri;
use fixed::traits::ToFixed;

pub struct RemoteReceiver<'d, PIO: pio::Instance, const SM: usize> {
    pio_sm: pio::StateMachine<'d, PIO, SM>,
    button_parser: ButtonParser,
}

impl<'d, PIO: pio::Instance, const SM: usize> RemoteReceiver<'d, PIO, SM> {
    pub fn new(pio: &mut pio::Common<'d, PIO>, mut pio_sm: pio::StateMachine<'d, PIO, SM>, receiver_pin: Peri<'static, impl PioPin>) -> Self {
        let mut pin = pio.make_pio_pin(receiver_pin);
        pin.set_pull(gpio::Pull::None);
        pio_sm.set_pin_dirs(pio::Direction::In, &[&pin]);

        let prg = pio_asm!(
            "startup:"
                "set x 31", // 31 is maximum and sufficient
            "assert_initial_low_pulse:",
                "jmp pin startup",
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

        Self {
            pio_sm,
            button_parser: ButtonParser::new(),
        }
    }

    pub async fn read(&mut self) -> &str {
        loop {
            let value = self.pio_sm.rx().wait_pull().await;
            if let Some(button) = self.button_parser.run(value) {
                return button;
            }
        }
    }
}

struct ButtonParser {
    last_value: Option<u32>,
}

impl ButtonParser {
    pub fn new() -> Self {
        Self {
            last_value: None,
        }
    }

    pub fn run(&mut self, value: u32) -> Option<&'static str> {
        // For more robustness a button must be received twice.
        match self.last_value {
            Some(last) if value == last => {
                match value {
                    0x017E9E90u32 => return Some("button 1"),
                    0x017E9E88u32 => return Some("button 2"),
                    0x017E9E98u32 => return Some("button 3"),
                    0x017E9E84u32 => return Some("button 4"),
                    0x017E9E94u32 => return Some("button 5"),
                    0x017E9E8Cu32 => return Some("button 6"),
                    0x017E9E9Cu32 => return Some("button 7"),
                    0x017E9E82u32 => return Some("button 8"),
                    0x017E9E92u32 => return Some("button 9"),
                    0x017E9E8Au32 => return Some("button 10"),
                    _ => return Some("undefined button"),
                }
            }
            _ => {
                self.last_value = Some(value);
                None
            }
        }
    }
}

#[cfg(test)]
#[embedded_test::tests]
mod button_parser_tests {
    use super::ButtonParser;

    const VALUES: &[(u32, &str)] = &[
        (0x017E9E90u32, "button 1"),
        (0x017E9E88u32, "button 2"),
        (0x017E9E98u32, "button 3"),
        (0x017E9E84u32, "button 4"),
        (0x017E9E94u32, "button 5"),
        (0x017E9E8Cu32, "button 6"),
        (0x017E9E9Cu32, "button 7"),
        (0x017E9E82u32, "button 8"),
        (0x017E9E92u32, "button 9"),
        (0x017E9E8Au32, "button 10"),
        (42u32, "undefined button"),
    ];

    #[test]
    fn the_same_button() {
        let mut button_parser = ButtonParser::new();

        for (value, button) in VALUES {
            // first time is expected None
            let result_button = button_parser.run(*value);
            assert_eq!(result_button, None, "expected button: {}", *button);

            // second time is expected the correct button
            let result_button = button_parser.run(*value);
            assert_eq!(result_button.unwrap(), *button, "expected button: {}", *button);

            // third time is also expected the correct button
            let result_button = button_parser.run(*value);
            assert_eq!(result_button.unwrap(), *button, "expected button: {}", *button);
        }
    }

    #[test]
    fn changing_button() {
        let mut button_parser = ButtonParser::new();

        // twice the same button results in the button
        let (value, button) = VALUES[0];
        let _ = button_parser.run(value);
        let result_button = button_parser.run(value);
        assert_eq!(result_button.unwrap(), button, "expected button: {}", button);

        // changing the button results first in None
        let (value, button) = VALUES[1];
        let result_button = button_parser.run(value);
        assert_eq!(result_button, None, "expected button: {}", button);

        // then again in the right button
        let result_button = button_parser.run(value);
        assert_eq!(result_button.unwrap(), button, "expected button: {}", button);
    }

    #[test]
    fn the_same_button_repeatedly() {
        let mut button_parser = ButtonParser::new();
        let (value, button) = VALUES[0];
        let mut result_button = None;

        for _ in 0..1_000 {
            result_button = button_parser.run(value);
        }

        assert_eq!(result_button.unwrap(), button, "expected button: {}", button);
    }
}
