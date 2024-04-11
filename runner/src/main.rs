use std::process::Command;
use std::thread;

use anyhow::{Ok, Result};

fn main() -> Result<(), anyhow::Error> {
    let http_server = thread::spawn(|| {
        Command::new("sh")
            .args(["-c", "cd ../http_server && cargo run"])
            .spawn()
            .expect("failed to start http server")
            .wait()
            .expect("failed to wait on http server");
    });

    let ebpf_server = thread::spawn(|| {
        Command::new("sh")
            .args(["-c", "cd ../ && RUST_LOG=info cargo xtask run"])
            .spawn()
            .expect("failed to start ebpf server")
            .wait()
            .expect("failed to wait on ebpf server");
    });

    ebpf_server.join().unwrap();
    http_server.join().unwrap();

    Ok(())
}
