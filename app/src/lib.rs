/*
* Everything in this library shall be runnable on any target, but at least on:
* - thumbv6m-none-eabi
* - x86_64-unknown-linux-gnu
*/

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EnterBootloader {
    fn call(&mut self);
}

pub struct Parser<E: EnterBootloader> {
    enter_bootloader: E,
}

impl<E: EnterBootloader> Parser<E> {
    pub fn new(enter_bootloader: E) -> Self {
        Self {
            enter_bootloader,
        }
    }

    pub fn parse_message(&mut self, msg: &[u8]) -> &[u8] {
        if msg.eq(b"enter bootloader") {
            self.enter_bootloader.call();
            // Note: probably this message won't be seen, because of immediate restart.
            b"entering bootloader now"
        }
        else {
            b"nothing to parse"
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    //use mockall::predicate::*;

    #[test]
    fn test_enter_bootloader() {
        //let mut mock_send_message = MockSendMessage::new();
        let mut mock_enter_bootloader = MockEnterBootloader::new();

        // mock_send_message.expect_call()
        //     .with(eq(b"entering bootloader now\n" as &[u8]))
        //     .times(1)
        //     .returning(|_| ());

        mock_enter_bootloader.expect_call()
            .times(1)
            .returning(|| ());

        let mut parser = Parser::new(
            //mock_send_message,
            mock_enter_bootloader,
        );

        let answer = parser.parse_message(b"enter bootloader");
        assert_eq!(answer, b"entering bootloader now");
    }

    #[test]
    fn test_nothing_to_parse() {
        let mut parser = Parser::new(
            MockEnterBootloader::new(),
        );

        let answer = parser.parse_message(b"no command");
        assert_eq!(answer, b"nothing to parse");
    }
}
