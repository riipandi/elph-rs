#![cfg(unix)]

use elph_tui::sigint_channel;
use signal_hook::consts::SIGINT;
use std::time::Duration;

#[tokio::test]
async fn sigint_channel_delivers_sigint_to_receiver() {
    let mut rx = sigint_channel();

    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(100));
        nix::sys::signal::kill(nix::unistd::getpid(), nix::sys::signal::Signal::SIGINT).unwrap();
    });

    let signal = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timed out waiting for SIGINT")
        .expect("sigint channel closed");

    assert_eq!(signal, SIGINT);
}