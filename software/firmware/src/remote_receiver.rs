use embassy_rp::gpio::Pull;
use embassy_rp::pio;
use fixed::traits::ToFixed;
use embassy_rp::pio::{Common, Config, FifoJoin, Instance, PioPin, ShiftDirection, StateMachine};
use {defmt_rtt as _, panic_probe as _};

pub struct RemoteReceiver<'d, T: Instance, const SM: usize> {
    sm: StateMachine<'d, T, SM>,
}

impl<'d, T: Instance, const SM: usize> RemoteReceiver<'d, T, SM> {
    pub fn new(pio: &mut Common<'d, T>, mut sm: StateMachine<'d, T, SM>, pin: impl PioPin) -> Self {
        let mut pin = pio.make_pio_pin(pin);
        pin.set_pull(Pull::None);
        sm.set_pin_dirs(pio::Direction::In, &[&pin]);

        let prg = pio_proc::pio_asm!("wait 1 pin 0", "wait 0 pin 0", "in pins, 0", "push",);

        let mut cfg = Config::default();
        cfg.set_in_pins(&[&pin]);
        cfg.fifo_join = FifoJoin::RxOnly;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.clock_divider = 1250.to_fixed(); //This should result in 125MHz / 1250 = 100kHz
        cfg.use_program(&pio.load_program(&prg.program), &[]);
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { sm }
    }

    pub async fn read(&mut self) -> &[u8] {
        // loop {
        //     match self.sm.rx().wait_pull().await {
        //         0 => return Direction::CounterClockwise,
        //         1 => return Direction::Clockwise,
        //         _ => {}
        //     }
        // }
		b"sm done"
    }
}

// pub enum Direction {
//     Clockwise,
//     CounterClockwise,
// }
