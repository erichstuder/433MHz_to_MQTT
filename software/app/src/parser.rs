#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EnterBootloader {
    fn call(&mut self);
}

#[cfg_attr(test, automock)]
pub trait Persistency {
    fn store_wifi_ssid(&mut self, wifi_ssid: &[u8]);
    fn read_wifi_ssid(&mut self) -> &[u8];
}

pub struct Parser<E: EnterBootloader, P: Persistency> {
    enter_bootloader: E,
    persistency: P,
}

impl<E: EnterBootloader, P: Persistency> Parser<E, P> {
    pub fn new(enter_bootloader: E, persistency: P) -> Self {
        Self {
            enter_bootloader,
            persistency,
        }
    }

    fn parse_store_command(&mut self, parameters: &[u8]) {
        const WIFI_SSID: &[u8] = b"wifi_ssid ";

        if parameters.starts_with(WIFI_SSID) {
            let value = &parameters[WIFI_SSID.len()..];
            self.persistency.store_wifi_ssid(value);
        }
    }

    fn parse_read_command(&mut self, parameters: &[u8]) -> &[u8]{
        const WIFI_SSID: &[u8] = b"wifi_ssid";

        if parameters.starts_with(WIFI_SSID) {
            return self.persistency.read_wifi_ssid();
        }
        b""
    }

    pub fn parse_message(&mut self, msg: &[u8]) -> &[u8] {
        const STORE_COMMAND: &[u8] = b"store ";
        const READ_COMMAND: &[u8] = b"read ";

        if msg == b"enter bootloader" {
            self.enter_bootloader.call();
            // Note: probably this message won't be seen, because of immediate restart.
            b"entering bootloader now\n"
        } else if msg == b"ping" {
            b"pong\n"
        } else if msg.starts_with(STORE_COMMAND) {
            let parameters = &msg[STORE_COMMAND.len()..];
            self.parse_store_command(parameters);
            b""
        } else if msg.starts_with(READ_COMMAND) {
            let parameters = &msg[READ_COMMAND.len()..];
            self.parse_read_command(parameters)
        } else {
            b"nothing to parse\n"
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

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
            MockPersistency::new(),
        );

        let answer = parser.parse_message(b"enter bootloader");
        assert_eq!(answer, b"entering bootloader now\n");
    }

    #[test]
    fn test_ping_pong() {
        let mut parser = Parser::new(
            MockEnterBootloader::new(),
            MockPersistency::new(),
        );

        let answer = parser.parse_message(b"ping");
        assert_eq!(answer, b"pong\n");
    }

    #[test]
    fn test_store_command() {
        let mut mock_persistency = MockPersistency::new();

        mock_persistency.expect_store_wifi_ssid()
            .with(eq(b"myValue" as &[u8]))
            .times(1)
            .returning(|_| ());

        let mut parser = Parser::new(
            MockEnterBootloader::new(),
            mock_persistency,
        );

        let answer = parser.parse_message(b"store wifi_ssid myValue");
        assert_eq!(answer, b"");
    }

    #[test]
    fn test_nothing_to_parse() {
        let mut parser = Parser::new(
            MockEnterBootloader::new(),
            MockPersistency::new(),
        );

        let answer = parser.parse_message(b"no command");
        assert_eq!(answer, b"nothing to parse\n");
    }
}
