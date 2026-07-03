//! Handling of the terminal communication.

#[derive(Debug)]
pub enum Error {
    BufferOverflow,
    Disconnected,
}

#[cfg_attr(test, mockall::automock)]
pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn send(&self, data: &[u8]) -> Result<(), Error>;
    #[allow(async_fn_in_trait)]
    async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, Error>;
    #[allow(async_fn_in_trait)]
    async fn parse_message(&mut self, msg: &[u8], answer: &mut [u8]) -> Result<usize, &'static str>;
}

pub struct Terminal<A, const MAX_PACKET_SIZE: usize> {
    actions: A,
}

impl<A: Actions, const MAX_PACKET_SIZE: usize> Terminal<A, MAX_PACKET_SIZE> {
    pub fn new(actions: A) -> Self {
        Self{ actions }
    }

    pub async fn run(&mut self) -> ! {
        let mut bytes = [0u8; MAX_PACKET_SIZE];
        let mut receive_buffer = [0u8; 128];
        let mut receive_buffer_index = 0usize;
        let mut ignore_message = false;

        loop {
            let byte_cnt = self.await_package(&mut bytes).await;

            for n in 0..byte_cnt {
                if bytes[n] == b'\n' {
                    if ignore_message {
                        ignore_message = false;
                    }
                    else {
                        let mut answer = [0u8; 300];
                        match self.actions.parse_message(&receive_buffer[..receive_buffer_index], &mut answer).await {
                            Ok(length) => {
                                self.actions.send(&answer[..length]).await.unwrap();
                            },
                            Err(e) => {
                                self.actions.send(b"ERROR: ").await.unwrap();
                                self.actions.send(&e.as_bytes()).await.unwrap();
                            },
                        };
                        self.actions.send("\n".as_bytes()).await.unwrap();
                    }
                    receive_buffer_index = 0;
                }
                else {
                    if receive_buffer_index < receive_buffer.len() {
                        receive_buffer[receive_buffer_index] = bytes[n];
                        receive_buffer_index += 1;
                    } else {
                        ignore_message = true;
                        self.actions.send("receive buffer overflow, this message is ignored: ".as_bytes()).await.unwrap();
                        self.actions.send(&receive_buffer).await.unwrap();
                        self.actions.send("...\n".as_bytes()).await.unwrap();
                        receive_buffer_index = 0;
                    }
                }
            }
        }
    }

    async fn await_package(&mut self, bytes: &mut [u8]) -> usize {
        loop {
            match self.actions.read_packet(bytes).await {
                Ok(byte_cnt) => return byte_cnt,
                Err(e) => {
                    match e {
                        Error::BufferOverflow => {
                            self.actions.send(b"receive buffer overflow, this message is ignored: ").await.unwrap();
                            self.actions.send(&bytes).await.unwrap();
                        },
                        Error::Disconnected => {
                            // May happen. No problem.
                        },
                    }
                    continue;
                },
            };
        }
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use tokio;
    use tokio::sync::oneshot::{self, Sender, Receiver};

    const MAX_PACKET_SIZE: usize = 10;

    fn setup() -> (MockActions, (Sender<()>, Receiver<()>)) {
        (
            MockActions::new(),
            oneshot::channel(),
        )
    }

    async fn run_terminal(mock_actions: MockActions, rx: Receiver<()>) {
        let mut terminal = Terminal::<MockActions, MAX_PACKET_SIZE>::new(mock_actions);
        tokio::spawn(async move { terminal.run().await; });
        let result = rx.await;
        assert!(result.is_ok(), "The background task dropped before hitting the assertion!");
    }

    #[tokio::test]
    async fn packet_size_correct() {
        let (mut mock_actions, (tx, rx)) = setup();

        mock_actions.expect_read_packet()
            .return_once(|bytes| {
                assert_eq!(bytes.len(), MAX_PACKET_SIZE);
                let _ = tx.send(());
                Ok(0)
            });

        run_terminal(mock_actions, rx).await;
    }

    #[tokio::test]
    async fn buffer_overflow() {
        let (mut mock_actions, (tx, rx)) = setup();

        mock_actions.expect_read_packet()
            .times(1)
            .return_once(|_| {
                Err(Error::BufferOverflow)
            });

        mock_actions.expect_send()
            .return_once(|data| {
                assert!(data.starts_with(b"receive buffer overflow, this message is ignored: "));
                let _ = tx.send(());
                Ok(())
            });

        run_terminal(mock_actions, rx).await;
    }

    #[tokio::test]
    async fn disconnected() {
        let (mut mock_actions, (tx, rx)) = setup();

        // The first call of read_packet we return an error.
        // The second time tx.send() is called to end the test.
        // send() must never be called.

        mock_actions.expect_read_packet()
            .times(1)
            .return_once(|_| {
                Err(Error::Disconnected)
            });
        mock_actions.expect_read_packet()
            .times(1)
            .return_once(|_| {
                let _ = tx.send(());
                Ok(0)
            });

        mock_actions.expect_send().never();

        run_terminal(mock_actions, rx).await;
    }

    #[tokio::test]
    async fn valid_message() {
        const MESSAGE: &[u8] = b"123456789\n";
        const ANSWER: &[u8] = b"hello";
        const ANSWER_LEN: usize = ANSWER.len();
        let (mut mock_actions, (tx, rx)) = setup();

        mock_actions.expect_read_packet()
            .times(1)
            .return_once(|buffer| {
                buffer.copy_from_slice(MESSAGE);
                Ok(MAX_PACKET_SIZE)
            });

        mock_actions.expect_parse_message()
            .times(1)
            .return_once(|msg, answer| {
                assert_eq!(msg, &MESSAGE[..MESSAGE.len()-1]);
                answer[..ANSWER_LEN].copy_from_slice(ANSWER);
                Ok(ANSWER_LEN)
            });

        mock_actions.expect_send()
            .times(1)
            .return_once(|data| {
                assert_eq!(data.len(), ANSWER_LEN);
                let _ = tx.send(());
                Ok(())
            });

        run_terminal(mock_actions, rx).await;
    }

    #[tokio::test]
    async fn parsing_error() {
        const ERROR_MSG: &str = "no_good";
        let (mut mock_actions, (tx, rx)) = setup();

        mock_actions.expect_read_packet()
            .times(1)
            .return_once(|buffer| {
                buffer[..1].copy_from_slice(b"\n");
                Ok(MAX_PACKET_SIZE)
            });

        mock_actions.expect_parse_message()
            .times(1)
            .return_once(|_, _| {
                Err(ERROR_MSG)
            });

        mock_actions.expect_send()
            .times(1)
            .return_once(|data| {
                assert_eq!(data, b"ERROR: ");
                Ok(())
            });
        mock_actions.expect_send()
            .times(1)
            .return_once(|data| {
                assert_eq!(data, ERROR_MSG.as_bytes());
                let _ = tx.send(());
                Ok(())
            });

        run_terminal(mock_actions, rx).await;
    }



    #[tokio::test]
    async fn too_long_message() {
        let (mut mock_actions, (tx, rx)) = setup();

        mock_actions.expect_read_packet()
            .returning(|buffer| {
                buffer.copy_from_slice(b"123456789_"); // data without \n to provoke receive buffer overflow
                Ok(MAX_PACKET_SIZE)
            });

        mock_actions.expect_send()
            .return_once(|data| {
                assert!(data.starts_with(b"receive buffer overflow, this message is ignored: "));
                let _ = tx.send(());
                Ok(())
            });

        run_terminal(mock_actions, rx).await;
    }
}
