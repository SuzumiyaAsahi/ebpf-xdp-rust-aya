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

// 我们这个XDP程序绑定的是eth0网卡
#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

// 其实理论上直接用String就行，但是String没有实现这些个trait，
// Rust的检查规则也不允许给外部类型加入新的trait，所以就需要包装一下
#[derive(Debug, sqlx::FromRow)]
struct BlockedIp {
    ipv4: String,
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
    // 前面的都是把ebpf转入到内核的操作，后面就需要自己DIY了。

    // 内核区的XDP主函数的名字叫做"xdp_firewall"
    let program: &mut Xdp = bpf.program_mut("xdp_firewall").unwrap().try_into()?;

    program.load()?;

    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // 定义用户区的HashMap，用于和内核区通信
    let mut blocklist: HashMap<_, u32, u32> = HashMap::try_from(bpf.map_mut("BLOCKLIST").unwrap())?;

    dotenvy::dotenv().ok();

    // 读取环境变量，得到数据库的url
    let db_url = env::var("DATABASE_URL").expect("Please set DATABASE_URL");

    // 测试链接数据库
    let pool = SqlitePool::connect(db_url.as_str()).await?;

    // 先删一下 package_info 这个数据表，以免上次程序没有正常退出导致这个表还存在
    let _ = sqlx::query("DROP TABLE package_info").execute(&pool).await;

    // 创建 package_info, 建立存储数据包信息的表
    // 其实用TEMPORY 就行，但是不知怎么的，不好使
    // query 相较于 query！的优点是query是运行时检测
    let _ = sqlx::query(
        "CREATE TABLE package_info (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source_ip TEXT NOT NULL,
        source_port INTEGER NOT NULL,
        destination_port INTEGER NOT NULL,
        proto_type TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    // 读取所有的被封锁IP地址, blocked_ip这个表是先天存在的
    let rows: Vec<BlockedIp> = sqlx::query_as("SELECT ipv4 FROM blocked_ip")
        .fetch_all(&pool)
        .await?;

    // 这个要在RingBuf前面执行，要不然就会违反借用原则
    // 将从blocked_ip中读取到的IP地址全部存入到HashMap中
    for i in rows {
        let block_addr: u32 = Ipv4Addr::from_str(&i.ipv4).unwrap().into();
        blocklist.insert(block_addr, 0, 0)?;
    }

    // 获得内核RingBuf的映射
    let events = RingBuf::try_from(bpf.map_mut("RB").unwrap())?;

    // 只能说是面向Discord编程了，还是自己太菜了。
    // 建立异步的RingBuf，自动帮你实现了epoll，这可真是太好了。
    let mut events_fd = AsyncFd::new(events).unwrap();

    // 提示 按Ctrl-C终止程序
    info!("Waiting for Ctrl-C...");

    loop {
        // select语句，两个异步语句选择其中一个执行，那个状态先完成，那个 => 后的代码就执行
        tokio::select! {

            // 如果检测到了用户按了Ctrl-C
            // 由于下面那个执行速度太快
            // 实际上你有可能需要按好几次才能被检测到。
            _ = signal::ctrl_c() => {
                // 退出循环
                info!("Exiting...");
                break;
            },

            // 读取内核缓冲区中的数据，然后再存储到数据库 package_info 中
            _ = async {
                // 检测这个RingBuf是否异步可读
                let mut guard = events_fd.readable_mut().await.unwrap();

                // 获得可变引用
                let events = guard.get_inner_mut();

                    // 循环读取RingBuf
                    while let Some(ring_event) = events.next() {
                    let info = PackageInfo::from_bytes(ring_event.first_chunk::<{ PINFOLEN }>().unwrap());

                    let proto_string = match info.proto_type() {
                        ProtocalType::TCP => String::from("TCP"),
                        ProtocalType::UDP => String::from("UDP"),
                        ProtocalType::Unknown => String::from("Unknown")
                    };

                    let ip_addr = Ipv4Addr::from(info.source_ip).to_string();

                    // 将数据存储到数据库 package_info 中
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

    // 当程序退出前，删除 package_info ,节约空间
    let _ = sqlx::query("DROP TABLE package_info").execute(&pool).await;

    Ok(())
}
