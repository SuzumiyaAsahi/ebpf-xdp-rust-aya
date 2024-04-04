use anyhow::Context;
use aya::maps::ring_buf::RingBuf;
use aya::maps::ring_buf::RingBufItem;
use aya::maps::HashMap;
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use clap::Parser;
use libc::sleep;
use log::{debug, info, warn};
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::convert::TryFrom;
use std::net::Ipv4Addr;
use tokio::io::unix::AsyncFd;
use tokio::signal;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::time;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/test-app"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/test-app"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp = bpf.program_mut("xdp_firewall").unwrap().try_into()?;

    program.load()?;

    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut blocklist: HashMap<_, u32, u32> = HashMap::try_from(bpf.map_mut("BLOCKLIST").unwrap())?;

    let block_addr: u32 = Ipv4Addr::new(172, 19, 96, 1).try_into()?;

    blocklist.insert(block_addr, 0, 0)?;

    let mut events = RingBuf::try_from(bpf.map_mut("RB").unwrap())?;
    let mut events_fd = AsyncFd::new(events).unwrap();

    loop {
        let mut guard = events_fd.readable_mut().await.unwrap();
        let events = guard.get_inner_mut();
        while let Some(ring_event) = events.next() {
            let the_len = ring_event.len();
            for i in 0..the_len {
                println!("{}", ring_event[i]);
            }
        }
    }

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
