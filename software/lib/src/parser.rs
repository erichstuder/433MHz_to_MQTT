//! Parses received messages, forwards them accordingly and returns the answer.

use crate::persistency::{Key, PersistencyTrait};

#[cfg_attr(test, mockall::automock)]
pub trait Command {
    fn cmd_str(&self) -> &'static [u8];
    fn run(&self, answer: &mut [u8]) -> Result<usize, &'static str>;
}

pub struct Parser<'a, P: PersistencyTrait, C: Command> {
    persistency: &'a mut P,
    additional_command: C,
}

impl <'a, P: PersistencyTrait, C: Command> Parser<'a, P, C> {
    pub fn new(persistency: &'a mut P, additional_command: C) -> Self {
        Self {
            persistency,
            additional_command,
        }
    }

    pub async fn parse_message(&mut self, msg: &[u8], answer: &mut [u8]) -> Result<usize, &'static str> {
        const STORE_COMMAND: &[u8] = b"store ";
        const READ_COMMAND: &[u8] = b"read ";
        if msg == self.additional_command.cmd_str() {
            self.additional_command.run(answer)
        }
        else if msg == b"ping" {
            Ok(Self::copy_to_beginning(answer, b"pong"))
        }
        else if msg == b"version" {
            if let (
                Some(version),
                Some(compile_time),
                Some(commit_hash)
            ) = (
                option_env!("CARGO_PKG_VERSION"),
                option_env!("COMPILE_TIME"),
                option_env!("COMMIT_HASH")
            ) {
                let version_text = "version: ";
                let mut idx1 = 0;
                let mut idx2 = version_text.len();
                answer[idx1..idx2].copy_from_slice(version_text.as_bytes());

                idx1 = idx2;
                idx2 += version.len();
                answer[idx1..idx2].copy_from_slice(version.as_bytes());

                let compile_time_text = "\ncompile time: ";
                idx1 = idx2;
                idx2 += compile_time_text.len();
                answer[idx1..idx2].copy_from_slice(compile_time_text.as_bytes());

                idx1 = idx2;
                idx2 += compile_time.len();
                answer[idx1..idx2].copy_from_slice(compile_time.as_bytes());

                let commit_hash_text = "\ncommit hash: ";
                idx1 = idx2;
                idx2 += commit_hash_text.len();
                answer[idx1..idx2].copy_from_slice(commit_hash_text.as_bytes());

                idx1 = idx2;
                idx2 += commit_hash.len();
                answer[idx1..idx2].copy_from_slice(commit_hash.as_bytes());

                Ok(idx2)
            } else {
                Err("version information not set")
            }
        }
        else if msg.starts_with(STORE_COMMAND) {
            let parameters = &msg[STORE_COMMAND.len()..];
            match self.parse_store_command(parameters).await {
                Ok(_) => Ok(0),
                Err(e) => Err(e),
            }
        }
        else if msg.starts_with(READ_COMMAND) {
            let parameters = &msg[READ_COMMAND.len()..];
            self.parse_read_command(parameters, answer).await
        }
        else if msg.starts_with(b"help") {
            Ok(Self::copy_to_beginning(answer, concat!(
                "commands:\n",
                "enter bootloader           : enters the bootloader to flash via usb\n",
                "ping                       : results in 'pong'\n",
                "version                    : provides version information\n",
                "store <value_name> <value> : stores a value persistently\n",
                "read <value_name>          : reads a persistent value\n",
                "help                       : prints this help"
            ).as_bytes()))
        } else {
            Err("not a valid command, type 'help' for help")
        }
    }

    async fn parse_store_command(&mut self, parameters: &[u8]) -> Result<(), &'static str> {
        const WIFI_SSID: &[u8] = b"wifi_ssid ";
        const WIFI_PASSWORD: &[u8] = b"wifi_password ";
        const MQTT_HOST_IP: &[u8] = b"mqtt_host_ip ";
        const MQTT_BROKER_USERNAME: &[u8] = b"mqtt_broker_username ";
        const MQTT_BROKER_PASSWORD: &[u8] = b"mqtt_broker_password ";

        if parameters.starts_with(WIFI_SSID) {
            let value = &parameters[WIFI_SSID.len()..];
            self.persistency.store(Key::WifiSsid, value).await;
            Ok(())
        }
        else if parameters.starts_with(WIFI_PASSWORD) {
            let value = &parameters[WIFI_PASSWORD.len()..];
            self.persistency.store(Key::WifiPassword, value).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_HOST_IP) {
            let value = &parameters[MQTT_HOST_IP.len()..];
            self.persistency.store(Key::MqttHostIp, value).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_BROKER_USERNAME) {
            let value = &parameters[MQTT_BROKER_USERNAME.len()..];
            self.persistency.store(Key::MqttBrokerUsername, value).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_BROKER_PASSWORD) {
            let value = &parameters[MQTT_BROKER_PASSWORD.len()..];
            self.persistency.store(Key::MqttBrokerPassword, value).await;
            Ok(())
        }
        else {
            Err("unknown store parameter, type 'read help' for help ('store help' not yet available)")
        }
    }

    async fn parse_read_command(&mut self, parameters: &[u8], answer: &mut [u8]) -> Result<usize, &'static str>{
        if parameters.starts_with(b"wifi_ssid") {
            Ok(self.persistency.read(Key::WifiSsid, answer).await)
        }
        else if parameters.starts_with(b"wifi_password") {
            Ok(self.persistency.read(Key::WifiPassword, answer).await)
        }
        else if parameters.starts_with(b"mqtt_host_ip") {
            Ok(self.persistency.read(Key::MqttHostIp, answer).await)
        }
        else if parameters.starts_with(b"mqtt_broker_username") {
            Ok(self.persistency.read(Key::MqttBrokerUsername, answer).await)
        }
        else if parameters.starts_with(b"mqtt_broker_password") {
            Ok(self.persistency.read(Key::MqttBrokerPassword, answer).await)
        }
        else if parameters.starts_with(b"help") {
            Ok(Self::copy_to_beginning(answer, concat!(
                "read value names:\n",
                "wifi_ssid\n",
                "wifi_password\n",
                "mqtt_host_ip\n",
                "mqtt_broker_username\n",
                "mqtt_broker_password"
            ).as_bytes()))
        }
        else {
            Err("unknown value name, type 'read help' for help")
        }
    }

    fn copy_to_beginning(dest: &mut [u8], src: &[u8]) -> usize {
        let len = src.len().min(dest.len());
        dest[..len].copy_from_slice(&src[..len]);
        len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use crate::persistency::MockPersistencyTrait;

    type TestParser = Parser<'static, MockPersistencyTrait, MockCommand>;

    fn get_default_parser() -> TestParser {
        let mock_persistency = Box::leak(Box::new(MockPersistencyTrait::new()));
        let mut mock_command = MockCommand::new();
        mock_command.expect_cmd_str().returning(|| "dummy".as_bytes());
        Parser::new(&mut *mock_persistency, mock_command)
    }

    #[tokio::test]
    async fn additional_command() {
        let mut mock_persistency = MockPersistencyTrait::new();
        let mut mock_command = MockCommand::new();
        let my_command = "my_command".as_bytes();
        let ok_message: &[u8] = "my_command_good".as_bytes();
        mock_command.expect_cmd_str().return_once(|| my_command);
        mock_command.expect_run().return_once(|answer| {
            TestParser::copy_to_beginning(answer, ok_message);
            Ok(ok_message.len())
        });
        let mut parser = Parser::new(&mut mock_persistency, mock_command);

        let mut answer: [u8; 32] = ['2' as u8; 32];
        let length = parser.parse_message(my_command, &mut answer).await.unwrap();
        assert_eq!(&answer[..length], ok_message);
    }

    #[tokio::test]
    async fn ping_pong() {
        let mut parser = get_default_parser();

        let mut answer: [u8; 32] = ['2' as u8; 32];
        let length = parser.parse_message(b"ping", &mut answer).await.unwrap();
        assert_eq!(&answer[..length], b"pong");
    }

    #[tokio::test]
    async fn store_command() {
        let commands = vec![
            (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       Key::WifiSsid),
            (b"wifi_password".as_ref(),        b"12345".as_ref(),         Key::WifiPassword),
            (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), Key::MqttHostIp),
            (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     Key::MqttBrokerUsername),
            (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     Key::MqttBrokerPassword),
        ];

        for (command, value, value_id) in commands {
            let mut mock_persistency = MockPersistencyTrait::new();
            mock_persistency.expect_store()
                .times(1)
                .withf(move |id, v| *id == value_id && v == value)
                .returning(|_, _| ());

            let mut mock_command = MockCommand::new();
            mock_command.expect_cmd_str().returning(|| "dummy".as_bytes());
            let mut parser = Parser::new(&mut mock_persistency, mock_command);

            let mut message = Vec::new();
            message.extend_from_slice(b"store ");
            message.extend_from_slice(command);
            message.extend_from_slice(b" ");
            message.extend_from_slice(value);

            let mut answer = ['\0' as u8; 0];
            let length = parser.parse_message(message.as_slice(), &mut answer).await.unwrap();
            assert_eq!(&answer[..length], b"");
        }
    }

    #[tokio::test]
    async fn invalid_store_value_name() {
        let mut parser = get_default_parser();

        let mut answer = ['\0' as u8; 100];
        match parser.parse_message(b"store dummy", &mut answer).await {
            Ok(_) => assert!(false),
            Err(msg) => assert!(msg == "unknown store parameter, type 'read help' for help ('store help' not yet available)"),
        }
    }

    #[tokio::test]
    async fn read_command() {
        const COMMANDS: &[( &[u8], &[u8], Key )] = &[
            (b"wifi_ssid",            b"myValue",       Key::WifiSsid),
            (b"wifi_password",        b"12345",         Key::WifiPassword),
            (b"mqtt_host_ip",         b"this.is.no.ip", Key::MqttHostIp),
            (b"mqtt_broker_username", b"UOWKDNDLE",     Key::MqttBrokerUsername),
            (b"mqtt_broker_password", b"__::)()()",     Key::MqttBrokerPassword),
        ];

        let mut mock_persistency = MockPersistencyTrait::new();
        for (_, value, value_id) in COMMANDS {

            mock_persistency.expect_read()
                .times(1)
                .withf(move |id, _| *id == *value_id)
                .returning_st(move |_, answer| {
                    answer[..value.len()].copy_from_slice(value);
                    value.len()
                });
        }

        let mut mock_command = MockCommand::new();
        mock_command.expect_cmd_str().returning(|| "dummy".as_bytes());
        let mut parser = Parser::new(&mut mock_persistency, mock_command);

        for (command, value, _) in COMMANDS {
            let mut message = Vec::new();
            message.extend_from_slice(b"read ");
            message.extend_from_slice(command);

            let mut answer = ['\0' as u8; 100];
            let length = parser.parse_message(message.as_slice(), &mut answer).await.unwrap();
            assert_eq!(&answer[..length], *value);
        }
    }

    #[tokio::test]
    async fn invalid_read_value_name() {
        let mut parser = get_default_parser();

        let mut answer = ['\0' as u8; 100];
        match parser.parse_message(b"read adfasdf", &mut answer).await {
            Ok(_) => assert!(false),
            Err(msg) => assert!(msg == "unknown value name, type 'read help' for help"),
        }
    }

    #[tokio::test]
    async fn nothing_to_parse() {
        let mut mock_persistency = MockPersistencyTrait::new();

        mock_persistency.expect_read().never();
        mock_persistency.expect_store().never();

        let mut mock_command = MockCommand::new();
        mock_command.expect_cmd_str().returning(|| "dummy".as_bytes());
        let mut parser = Parser::new(&mut mock_persistency, mock_command);

        let mut answer = ['\0' as u8; 300];
        match parser.parse_message(b"no command", &mut answer).await {
            Ok(_) => assert!(false),
            Err(msg) => assert!(msg == "not a valid command, type 'help' for help"),
        }
    }
}
