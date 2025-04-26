//! Parses received messages, forwards them accordingly and returns the answer.

use crate::modules::persistency::{ValueId, PersistencyTrait};

pub struct Parser<'a, P: PersistencyTrait> {
    persistency: &'a P,
}

impl <'a, P> Parser<'a, P>
where P: PersistencyTrait,
{
    pub fn new(persistency: &'a P) -> Self {
        Self { persistency }
    }

    async fn parse_store_command(&mut self, parameters: &[u8]) -> Result<(), &'static str> {
        const WIFI_SSID: &[u8] = b"wifi_ssid ";
        const WIFI_PASSWORD: &[u8] = b"wifi_password ";
        const MQTT_HOST_IP: &[u8] = b"mqtt_host_ip ";
        const MQTT_BROKER_USERNAME: &[u8] = b"mqtt_broker_username ";
        const MQTT_BROKER_PASSWORD: &[u8] = b"mqtt_broker_password ";

        if parameters.starts_with(WIFI_SSID) {
            let value = &parameters[WIFI_SSID.len()..];
            self.persistency.store(value, ValueId::WifiSsid).await;
            Ok(())
        }
        else if parameters.starts_with(WIFI_PASSWORD) {
            let value = &parameters[WIFI_PASSWORD.len()..];
            self.persistency.store(value, ValueId::WifiPassword).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_HOST_IP) {
            let value = &parameters[MQTT_HOST_IP.len()..];
            self.persistency.store(value, ValueId::MqttHostIp).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_BROKER_USERNAME) {
            let value = &parameters[MQTT_BROKER_USERNAME.len()..];
            self.persistency.store(value, ValueId::MqttBrokerUsername).await;
            Ok(())
        }
        else if parameters.starts_with(MQTT_BROKER_PASSWORD) {
            let value = &parameters[MQTT_BROKER_PASSWORD.len()..];
            self.persistency.store(value, ValueId::MqttBrokerPassword).await;
            Ok(())
        }
        else {
            Err("unknown store parameter")
        }
    }

    async fn parse_read_command(&mut self, parameters: &[u8], answer: &mut [u8]) -> Result<usize, &'static str> {
        if parameters.starts_with(b"wifi_ssid") {
            self.persistency.read(ValueId::WifiSsid, answer).await
        }
        else if parameters.starts_with(b"wifi_password") {
            self.persistency.read(ValueId::WifiPassword, answer).await
        }
        else if parameters.starts_with(b"mqtt_host_ip") {
            self.persistency.read(ValueId::MqttHostIp, answer).await
        }
        else if parameters.starts_with(b"mqtt_broker_username") {
            self.persistency.read(ValueId::MqttBrokerUsername, answer).await
        }
        else if parameters.starts_with(b"mqtt_broker_password") {
            self.persistency.read(ValueId::MqttBrokerPassword, answer).await
        }
        else {
            Err("unknown read parameter")
        }
    }

    fn copy_to_beginning(dest: &mut [u8], src: &[u8]) -> usize {
        let len = src.len().min(dest.len());
        dest[..len].copy_from_slice(&src[..len]);
        len
    }

    pub async fn parse_message(&mut self, msg: &[u8], answer: &mut [u8]) -> Result<usize, &'static str> {
        const STORE_COMMAND: &[u8] = b"store ";
        const READ_COMMAND: &[u8] = b"read ";
        if msg == b"enter bootloader" {
            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
            // Note: probably this message won't be seen, because of immediate restart.
            Ok(Self::copy_to_beginning(answer, b"entering bootloader now"))
        } else if msg == b"ping" {
            Ok(Self::copy_to_beginning(answer, b"pong"))
        } else if msg.starts_with(STORE_COMMAND) {
            let parameters = &msg[STORE_COMMAND.len()..];
            match self.parse_store_command(parameters).await {
                Ok(_) => Ok(0),
                Err(e) => Err(e),
            }
        } else if msg.starts_with(READ_COMMAND) {
            let parameters = &msg[READ_COMMAND.len()..];
            self.parse_read_command(parameters, answer).await
        } else {
            Ok(Self::copy_to_beginning(answer, b"nothing to parse"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use crate::modules::persistency::MockPersistencyTrait;

    #[tokio::test]
    async fn test_ping_pong() {
        let mock_persistency = MockPersistencyTrait::new();
        let mut parser = Parser::new(&mock_persistency);

        let mut answer: [u8; 32] = ['2' as u8; 32];
        let length = parser.parse_message(b"ping", &mut answer).await.unwrap();
        assert_eq!(&answer[..length], b"pong");
    }

    #[tokio::test]
    async fn test_store_command() {
        let commands = vec![
            (b"wifi_ssid".as_ref(),            b"myValue".as_ref(),       ValueId::WifiSsid),
            (b"wifi_password".as_ref(),        b"12345".as_ref(),         ValueId::WifiPassword),
            (b"mqtt_host_ip".as_ref(),         b"this.is.no.ip".as_ref(), ValueId::MqttHostIp),
            (b"mqtt_broker_username".as_ref(), b"UOWKDNDLE".as_ref(),     ValueId::MqttBrokerUsername),
            (b"mqtt_broker_password".as_ref(), b"__::)()()".as_ref(),     ValueId::MqttBrokerPassword),
        ];

        for (command, value, value_id) in commands {
            let mut mock_persistency = MockPersistencyTrait::new();
            mock_persistency.expect_store()
                .times(1)
                .withf(move |v, id| v == value && *id == value_id)
                .returning(|_, _| ());

            let mut parser = Parser::new(&mock_persistency);

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
    async fn test_read_command() {
        const COMMANDS: &[( &[u8], &[u8], ValueId )] = &[
            (b"wifi_ssid",            b"myValue",       ValueId::WifiSsid),
            (b"wifi_password",        b"12345",         ValueId::WifiPassword),
            (b"mqtt_host_ip",         b"this.is.no.ip", ValueId::MqttHostIp),
            (b"mqtt_broker_username", b"UOWKDNDLE",     ValueId::MqttBrokerUsername),
            (b"mqtt_broker_password", b"__::)()()",     ValueId::MqttBrokerPassword),
        ];

        let mut mock_persistency = MockPersistencyTrait::new();
        for (_, value, value_id) in COMMANDS {

            mock_persistency.expect_read()
                .times(1)
                .withf(move |id, _| *id == *value_id)
                .returning_st(move |_, answer| {
                    answer[..value.len()].copy_from_slice(value);
                    Ok(value.len())
                });
        }

        let mut parser = Parser::new(&mock_persistency);

        for (command, value, _) in COMMANDS {
            let mut message = Vec::new();
            message.extend_from_slice(b"read ");
            message.extend_from_slice(command);

            let mut answer = ['\0' as u8; 100];
            let length = parser.parse_message(message.as_slice(), &mut answer).await.unwrap();
            assert_eq!(&answer[..length], *value);
        }
    }

    //TODO: add a test for a failing read

    #[tokio::test]
    async fn test_nothing_to_parse() {
        let mut mock_persistency = MockPersistencyTrait::new();

        mock_persistency.expect_read().never();
        mock_persistency.expect_store().never();

        let mut parser = Parser::new(&mock_persistency);

        let mut answer = ['\0' as u8; 20];
        let length = parser.parse_message(b"no command", &mut answer).await.unwrap();
        assert_eq!(&answer[..length], b"nothing to parse");
    }
}
