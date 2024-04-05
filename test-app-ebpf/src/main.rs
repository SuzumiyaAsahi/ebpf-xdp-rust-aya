#![no_std]
#![no_main]

use aya_ebpf::macros::map;
use aya_ebpf::maps::ring_buf::RingBuf;
use aya_ebpf::maps::HashMap;
use aya_ebpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::info;
use test_app_common::{PackageInfo, PINFOLEN};

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

// const PINFOLEN: usize = 9;
//
// struct PackageInfo {
//     source_ip: u32,
//     source_port: u16,
//     destination_port: u16,
//     proto_type: u8,
// }
//
// impl PackageInfo {
//     pub fn new(
//         source_ip: u32,
//         source_port: u16,
//         destination_port: u16,
//         proto_type: u8,
//     ) -> PackageInfo {
//         PackageInfo {
//             source_ip,
//             source_port,
//             destination_port,
//             proto_type,
//         }
//     }
//     pub fn to_be_bytes(&self) -> [u8; PINFOLEN] {
//         let mut info_buf: [u8; PINFOLEN] = [0; PINFOLEN];
//         let need2move = self.source_ip.to_be_bytes();
//
//         info_buf[..4].copy_from_slice(&need2move[..]);
//
//         let need2move = self.source_port.to_be_bytes();
//
//         info_buf[4..4 + 2].clone_from_slice(&need2move[..]);
//
//         let need2move = self.destination_port.to_be_bytes();
//
//         info_buf[6..8].clone_from_slice(&need2move[..]);
//
//         let need2move = self.proto_type.to_be_bytes();
//
//         info_buf[8..].clone_from_slice(&need2move[..]);
//
//         info_buf
//     }
// }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[map]
static BLOCKLIST: HashMap<u32, u32> = HashMap::<u32, u32>::with_max_entries(1024, 0);

#[map]
static RB: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

// fn block_ip(address: u32) -> bool {
//     unsafe { BLOCKLIST.get(&address).is_some() }
// }

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

fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?; //

    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = ptr_at(&ctx, EthHdr::LEN)?;
    let source_ip = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    // unsafe {
    //     let source_addr_be = source_addr.to_be_bytes();
    //     if let Some(mut buf) = RB.reserve::<[u8; 4]>(0) {
    //         (*buf.as_mut_ptr())[..source_addr_be.len()].copy_from_slice(&source_addr_be[..]);
    //         buf.submit(0);
    //     } else {
    //         info!(&ctx, "RingBuf does not have enough space to store data");
    //     }

    //这段代码被上面的代码所替代了，
    //上面的更安全，
    //在插入时检查了环形缓冲区是否有足够的空间来存储数据。
    // This code was replaced by the code above,
    // which is safer and checks
    // whether the ring buffer has enough space to store the data during insertion.

    // bpf_ringbuf_output(
    //     &RB as *const RingBuf as *mut core::ffi::c_void,
    //     &source_addr_be as *const _ as *mut c_void,
    //     core::mem::size_of::<u32>() as u64,
    //     0,
    // );
    info!(&ctx, "A num is writen");

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
        IpProto::Icmp => {
            info!(&ctx, "SRC IP:{:i}", source_ip);
            return Ok(xdp_action::XDP_PASS);
        }
        _ => return Err(()),
    };

    unsafe {
        let info = PackageInfo::new(source_ip, source_port, destination_port, proto_type as u8)
            .to_be_bytes();
        if let Some(mut buf) = RB.reserve::<[u8; PINFOLEN]>(0) {
            (*buf.as_mut_ptr())[..PINFOLEN].copy_from_slice(&info[..]);
            buf.submit(0)
        } else {
            info!(&ctx, "RingBuf does not have enough space to store data");
        }
    }

    // let source_addr_be = source_addr.to_be_bytes();
    // if let Some(mut buf) = RB.reserve::<[u8; 4]>(0) {
    //     (*buf.as_mut_ptr())[..source_addr_be.len()].copy_from_slice(&source_addr_be[..]);
    //     buf.submit(0);
    // } else {
    //     info!(&ctx, "RingBuf does not have enough space to store data");
    // }

    info!(
        &ctx,
        "SRC IP: {:i}, SRC PORT: {}, DES PORT {}", source_ip, source_port, destination_port
    );

    Ok(xdp_action::XDP_PASS)
}
