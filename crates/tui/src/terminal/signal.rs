use crossbeam::channel::{Receiver, TryRecvError, unbounded};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;

/// Receives `SIGINT` (Ctrl+C) delivered from a background listener thread.
pub struct SigintReceiver {
    inner: Receiver<i32>,
}

impl SigintReceiver {
    /// Waits for the next signal on the async runtime.
    pub async fn recv(&mut self) -> Option<i32> {
        loop {
            match self.inner.try_recv() {
                Ok(signal) => return Some(signal),
                Err(TryRecvError::Disconnected) => return None,
                Err(TryRecvError::Empty) => {
                    tokio::task::yield_now().await;
                }
            }
        }
    }
}

/// Delivers `SIGINT` (Ctrl+C) to the async runtime via a crossbeam channel.
pub fn sigint_channel() -> SigintReceiver {
    let (tx, rx) = unbounded();

    std::thread::spawn(move || {
        let mut signals = match Signals::new([SIGINT]) {
            Ok(signals) => signals,
            Err(_) => return,
        };

        for signal in signals.forever() {
            if tx.send(signal).is_err() {
                break;
            }
        }
    });

    SigintReceiver { inner: rx }
}