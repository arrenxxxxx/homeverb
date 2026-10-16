//! Bounded hidden terminal input; cancellation restores terminal settings before returning.

use homeverb_core::SecretBytes;
use homeverb_domain::{Error, Result};
use std::io::{self, IsTerminal};
use tokio_util::sync::CancellationToken;

fn unavailable() -> Error {
    Error::new(
        "INPUT_UNAVAILABLE",
        "Protected interactive terminal input is unavailable.",
    )
}

pub fn require_interactive() -> Result<()> {
    if !cfg!(target_os = "linux") || !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(unavailable());
    }
    Ok(())
}

pub async fn input(prompt: &str, cancellation: &CancellationToken) -> Result<SecretBytes> {
    require_interactive()?;
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY)
            .open("/dev/tty")
            .map_err(|_| unavailable())?;
        linux::read(file, prompt, cancellation).await
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (prompt, cancellation);
        Err(unavailable())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::{
        fs::File,
        io::{Read, Write},
        os::fd::AsRawFd,
    };
    use tokio::io::unix::AsyncFd;
    use zeroize::Zeroizing;

    struct HiddenTerminal {
        file: File,
        original: libc::termios,
    }

    impl HiddenTerminal {
        fn new(file: File) -> Result<Self> {
            let mut original = std::mem::MaybeUninit::uninit();
            // The descriptor belongs to this file; tcgetattr initializes termios on success.
            if unsafe { libc::tcgetattr(file.as_raw_fd(), original.as_mut_ptr()) } != 0 {
                return Err(unavailable());
            }
            let original = unsafe { original.assume_init() };
            let terminal = Self { file, original };
            let mut hidden = original;
            hidden.c_lflag &= !(libc::ECHO | libc::ECHONL);
            hidden.c_lflag |= libc::ICANON | libc::ISIG;
            // Discard queued input before disabling echo; never treat typeahead as a credential.
            if unsafe { libc::tcsetattr(terminal.as_raw_fd(), libc::TCSAFLUSH, &hidden) } != 0 {
                return Err(unavailable());
            }
            Ok(terminal)
        }
    }

    impl AsRawFd for HiddenTerminal {
        fn as_raw_fd(&self) -> std::os::fd::RawFd {
            self.file.as_raw_fd()
        }
    }

    impl Drop for HiddenTerminal {
        fn drop(&mut self) {
            // Restore on success, refusal or cancellation, discarding any unread private input.
            unsafe {
                libc::tcsetattr(self.as_raw_fd(), libc::TCSAFLUSH, &self.original);
            }
        }
    }

    pub(super) async fn read(
        file: File,
        prompt: &str,
        cancellation: &CancellationToken,
    ) -> Result<SecretBytes> {
        // HiddenTerminal owns its File and never closes or replaces the descriptor while registered.
        let terminal =
            unsafe { AsyncFd::register(HiddenTerminal::new(file)?) }.map_err(|_| unavailable())?;
        let mut output = &terminal.get_ref().file;
        output
            .write_all(prompt.as_bytes())
            .map_err(|_| unavailable())?;
        let mut bytes = Zeroizing::new(Vec::with_capacity(258));
        let mut chunk = Zeroizing::new([0u8; 258]);
        loop {
            let mut ready = tokio::select! {
                biased;
                _ = cancellation.cancelled() => return Err(Error::new("CANCELLED", "Protected terminal input was cancelled.")),
                ready = terminal.readable() => ready.map_err(|_| unavailable())?,
            };
            let read = ready.try_io(|inner| (&inner.get_ref().file).read(&mut *chunk));
            let count = match read {
                Err(_) => continue,
                Ok(Err(error)) if error.kind() == io::ErrorKind::Interrupted => continue,
                Ok(result) => result.map_err(|_| unavailable())?,
            };
            if count == 0 {
                return Err(unavailable());
            }
            // Never reallocate a buffer containing private input, even for an oversized line.
            if bytes.len() + count > 258 {
                return Err(Error::new(
                    "MIDEA_LOGIN_INPUT_UNSUPPORTED",
                    "The reviewed login flow requires bounded credentials.",
                ));
            }
            bytes.extend_from_slice(&chunk[..count]);
            if bytes.last() == Some(&b'\n') {
                bytes.pop();
                if bytes.last() == Some(&b'\r') {
                    bytes.pop();
                }
                if bytes.len() <= 256 {
                    output.write_all(b"\n").map_err(|_| unavailable())?;
                    return Ok(SecretBytes::new(bytes.to_vec()));
                }
            }
            if bytes.len() > 256 {
                return Err(Error::new(
                    "MIDEA_LOGIN_INPUT_UNSUPPORTED",
                    "The reviewed login flow requires bounded credentials.",
                ));
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::fd::FromRawFd;

        fn pair() -> (File, File) {
            let (mut master, mut slave) = (-1, -1);
            // Fresh test-only terminal descriptors, never the user's terminal.
            assert_eq!(
                unsafe {
                    libc::openpty(
                        &mut master,
                        &mut slave,
                        std::ptr::null_mut(),
                        std::ptr::null(),
                        std::ptr::null(),
                    )
                },
                0
            );
            assert_ne!(
                unsafe { libc::fcntl(slave, libc::F_SETFL, libc::O_NONBLOCK) },
                -1
            );
            unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) }
        }

        fn echo(file: &File) -> bool {
            let mut term = std::mem::MaybeUninit::uninit();
            assert_eq!(
                unsafe { libc::tcgetattr(file.as_raw_fd(), term.as_mut_ptr()) },
                0
            );
            unsafe { term.assume_init().c_lflag & libc::ECHO != 0 }
        }

        #[tokio::test]
        async fn hidden_input_is_bounded_and_restores_echo() {
            for length in [1, 256, 257] {
                let (mut master, slave) = pair();
                let inspection = slave.try_clone().unwrap();
                let original = echo(&inspection);
                let cancel = CancellationToken::new();
                let input = read(slave, "Fictional prompt: ", &cancel);
                tokio::pin!(input);
                tokio::select! { biased;
                    _ = &mut input => panic!("Input should wait for a line."),
                    _ = tokio::task::yield_now() => {}
                }
                assert!(!echo(&inspection));
                let mut line = vec![b'x'; length];
                line.push(b'\n');
                master.write_all(&line).unwrap();
                let result = tokio::time::timeout(std::time::Duration::from_secs(1), input)
                    .await
                    .unwrap();
                if length <= 256 {
                    assert_eq!(result.unwrap().expose(), &line[..length]);
                } else {
                    assert_eq!(result.err().unwrap().code, "MIDEA_LOGIN_INPUT_UNSUPPORTED");
                }
                assert_eq!(echo(&inspection), original);
                let mut output = [0; 64];
                let count = master.read(&mut output).unwrap();
                assert!(!output[..count].contains(&b'x'));
            }
        }

        #[tokio::test]
        async fn cancellation_restores_terminal_and_flushes_queued_credentials() {
            let (mut master, slave) = pair();
            let inspection = slave.try_clone().unwrap();
            let original = echo(&inspection);
            let cancel = CancellationToken::new();
            let input = read(slave, "Fictional prompt: ", &cancel);
            tokio::pin!(input);
            tokio::select! { biased;
                _ = &mut input => panic!("Input should wait for a line."),
                _ = tokio::task::yield_now() => {}
            }
            master.write_all(b"FICTIONAL_PRIVATE_INPUT\n").unwrap();
            tokio::time::timeout(std::time::Duration::from_secs(1), async {
                loop {
                    let mut count: libc::c_int = 0;
                    assert_eq!(
                        unsafe { libc::ioctl(inspection.as_raw_fd(), libc::FIONREAD, &mut count) },
                        0
                    );
                    if count > 0 {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            cancel.cancel();
            assert_eq!(input.await.err().unwrap().code, "CANCELLED");
            assert_eq!(echo(&inspection), original);
            master.write_all(b"\n").unwrap();
            let mut buffer = [0; 64];
            // The queued private input was flushed before echo was restored.
            assert_eq!((&inspection).read(&mut buffer).unwrap(), 1);
            assert_eq!(buffer[0], b'\n');
        }
    }
}
