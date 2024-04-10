#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::ring_buf::RingBuf,
    maps::HashMap,
    programs::XdpContext,
};
use aya_log_ebpf::info;
use test_app_common::{PackageInfo, PINFOLEN};

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

// 这个根本不会用到，这是为了满足Rust编译器的检查要求，
// 所谓Rust特色编程主义了。
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

// 保存被封锁的ip
#[map]
static BLOCKLIST: HashMap<u32, u32> = HashMap::<u32, u32>::with_max_entries(1024, 0);

// 环形缓冲区，用于内核和用户程序的通信
#[map]
static RB: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

// 所有通过内核的数据包都会通过这个函数
#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        // 如果是Ok，就通过这个包
        Ok(ret) => ret,
        // 如果是Err，就丢弃这个包
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

// 检查这个包的IP是不是在被封锁的IP范畴中
fn block_ip(address: u32) -> bool {
    unsafe { BLOCKLIST.get(&address).is_some() }
}

// 检查对包的操作是否越界，由此来通过ebpf的检查器的检验
// 属于ebpf特色主义编程了。
#[inline(always)] //
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

// 对数据包的具体操作函数
fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {
    // 从二层数据开始检查
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?; //

    // 如果是Ipv4就继续代码
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    // 开始检查三层数据包的检查
    let ipv4hdr: *const Ipv4Hdr = ptr_at(&ctx, EthHdr::LEN)?;

    // 将IP地址由大端序转化为小端序
    let source_ip = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    // 如果该IP地址是要被封锁的地址，就直接返回，返回丢弃命令
    if block_ip(source_ip) {
        return Ok(xdp_action::XDP_DROP);
    }

    // info!(&ctx, "A num is writen");

    // 获得源端口，目的端口，协议类型等信息
    let (source_port, destination_port, proto_type) = match unsafe { (*ipv4hdr).proto } {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr = ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            (
                u16::from_be(unsafe { (*tcphdr).source }),
                u16::from_be(unsafe { (*tcphdr).dest }),
                IpProto::Tcp,
            )
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr = ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            (
                u16::from_be(unsafe { (*udphdr).source }),
                u16::from_be(unsafe { (*udphdr).dest }),
                IpProto::Udp,
            )
        }
        // 阻止ICMP包通过，预防DDOS攻击
        // IpProto::Icmp => {
        //     info!(&ctx, "SRC IP:{:i}", source_ip);
        //     return Ok(xdp_action::XDP_PASS);
        // }
        _ => return Err(()),
    };

    // 将收集到的信息存储在RingBuf,也就是环形缓冲区中，实现对用户态程序的通信
    unsafe {
        let info = PackageInfo::new(source_ip, source_port, destination_port, proto_type as u8)
            .to_be_bytes();
        // 先判断环形缓冲区还有没有空间来存储数据
        if let Some(mut buf) = RB.reserve::<[u8; PINFOLEN]>(0) {
            // 如果有，就将数据复制到缓冲区中
            (*buf.as_mut_ptr())[..PINFOLEN].copy_from_slice(&info[..]);
            // 然后提交，转入到map中，这样用户态数据就可以读取了
            buf.submit(0)
        } else {
            // 如果无法存储，就打印一个日志
            info!(&ctx, "RingBuf does not have enough space to store data");
        }
    }

    // 打印一些信息
    info!(
        &ctx,
        "SRC IP: {:i}, SRC PORT: {}, DES PORT {}", source_ip, source_port, destination_port
    );

    Ok(xdp_action::XDP_PASS)
}
