/*
* Everything in this library shall be runnable on any target, at least on:
* - thumbv6m-none-eabi
* - x86_64-unknown-linux-gnu
*/

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use mockall::*;

#[cfg(test)]
use mockall::predicate::*;

#[cfg_attr(test, automock)]
pub trait ParserTrait {
    fn send_message(&self, msg: &[u8]);
    fn enter_bootlaoder(&self);
}

pub struct Parser<'a> {
     parser_trait: &'a dyn ParserTrait,
}

impl Parser<'_> {
    pub fn new(parser_trait: &'static impl ParserTrait) {
        Self {parser_trait};
    }

    pub fn parse_message(&self, msg: &[u8]) {
        if msg.eq(b"enter bootloader") {
            self.parser_trait.send_message(b"entering bootloader now\n");
            self.parser_trait.enter_bootlaoder();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_bootloader_test() {
        let mut mock = MockParserTrait::new();

        let text = b"Hello";

        mock.expect_send_message()
            .times(1)
            .withf(|msg: &[u8]| msg.eq(text))
            .return_const(());

        mock.send_message(text);
    }
}
