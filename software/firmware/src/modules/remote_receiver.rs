//! Receives data from the remote.

use cfg_if::cfg_if;
use {defmt_rtt as _, panic_probe as _};

cfg_if! {
    if #[cfg(not(test))] {
        use embassy_rp::{gpio, pio};
        use embassy_rp::pio::PioPin;
        use embassy_rp::pio::program::pio_asm;
        use fixed::traits::ToFixed;
    }
}

#[cfg(not(test))]
pub struct RemoteReceiver<'d, PIO: pio::Instance, const SM: usize> {
    pio_sm: pio::StateMachine<'d, PIO, SM>,
    button_parser: ButtonParser,
}

#[cfg(not(test))]
impl<'d, PIO: pio::Instance, const SM: usize> RemoteReceiver<'d, PIO, SM> {
    pub fn new(pio: &mut pio::Common<'d, PIO>, mut pio_sm: pio::StateMachine<'d, PIO, SM>, receiver_pin: impl PioPin) -> Self {
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
    value_cnt: u8,
}

impl ButtonParser {
    pub fn new() -> Self {
        Self {
            last_value: None,
            value_cnt: 0,
        }
    }

    pub fn run(&mut self, value: u32) -> Option<&'static str> {
        match self.last_value {
            Some(last) if value == last => {
                self.value_cnt += 1;
            }
            _ => {
                self.value_cnt = 1;
                self.last_value = Some(value);
            }
        }

        if self.value_cnt >= 2 {
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
        None
    }
}

#[cfg(test)]
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
}
