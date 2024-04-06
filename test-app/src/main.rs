use anyhow::{Context, Ok};
use aya::programs::{Xdp, XdpFlags};
use aya::{include_bytes_aligned, Bpf};
use aya::{maps::ring_buf::RingBuf, maps::HashMap};
use aya_log::BpfLogger;
use clap::Parser;
use log::{debug, info, warn};
use sqlx::sqlite::SqlitePool;
use std::{env, net::Ipv4Addr, str::FromStr};
use test_app_common::{PackageInfo, ProtocalType, PINFOLEN};
use tokio::{io::unix::AsyncFd, signal};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

#[derive(Debug)]
struct BLockedIp {
    ipv4: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
struct IpInfo {
    id: i64,
    source_ip: String,
    source_port: i64,
    destination_port: i64,
    proto_type: String,
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

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("Please set DATABASE_URL");
    let pool = SqlitePool::connect(db_url.as_str()).await?;
    let _ = sqlx::query("DROP TABLE package_info").execute(&pool).await;

    let _ = sqlx::query!(
        "CREATE TABLE package_info (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source_ip TEXT NOT NULL,
        source_port INTEGER NOT NULL,
        destination_port INTEGER NOT NULL,
        proto_type TEXT NOT NULL
        );"
    )
    .execute(&pool)
    .await?;

    let rows: Vec<BLockedIp> = sqlx::query_as!(BLockedIp, r#"SELECT ipv4 FROM blocked_ip"#)
        .fetch_all(&pool)
        .await?;

    // 这个要在RingBuf前面执行，要不然就会违反借用原则
    for i in rows {
        let block_addr: u32 = Ipv4Addr::from_str(&i.ipv4).unwrap().try_into()?;
        blocklist.insert(block_addr, 0, 0)?;
    }

    let events = RingBuf::try_from(bpf.map_mut("RB").unwrap())?;
    let mut events_fd = AsyncFd::new(events).unwrap();
    info!("Waiting for Ctrl-C...");
    loop {
        tokio::select! {

            _ = signal::ctrl_c() => {
                info!("Exiting...");
                break;
            },

            _ = async {
                let mut guard = events_fd.readable_mut().await.unwrap();
                let events = guard.get_inner_mut();
                    while let Some(ring_event) = events.next() {
                    let info = PackageInfo::from_bytes(ring_event.first_chunk::<{ PINFOLEN }>().unwrap());

                    let proto_string = match info.proto_type() {
                        ProtocalType::TCP => String::from("TCP"),
                        ProtocalType::UDP => String::from("UDP"),
                        ProtocalType::Unknown => String::from("Unknown")
                    };

                    let ip_addr = Ipv4Addr::from(info.source_ip).to_string();
                    let _ = sqlx::query("INSERT INTO package_info
                        (source_ip, source_port, destination_port, proto_type) 
                        VALUES ($1, $2, $3, $4)"
                        )
                        .bind(ip_addr)
                        .bind(info.source_port)
                        .bind(info.destination_port)
                        .bind(proto_string)
                        .execute(&pool)
                        .await;
                }

            }=>{}
        };
    }

    let _ = sqlx::query("DROP TABLE package_info").execute(&pool).await;

    Ok(())
}
