#![no_std]
#![no_main]

use aya_ebpf::cty::c_void;
use aya_ebpf::helpers::bpf_ringbuf_output;
use aya_ebpf::macros::map;
use aya_ebpf::maps::ring_buf::RingBuf;
use aya_ebpf::maps::HashMap;
use aya_ebpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::info;

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

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

fn block_ip(address: u32) -> bool {
    unsafe { BLOCKLIST.get(&address).is_some() }
}

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
    let source_addr = u32::from_be(unsafe { (*ipv4hdr).src_addr });

    unsafe {
        let source_addr_be = source_addr.to_be_bytes();
        bpf_ringbuf_output(
            &RB as *const RingBuf as *mut core::ffi::c_void,
            &source_addr_be as *const _ as *mut c_void,
            core::mem::size_of::<u32>() as u64,
            0,
        );
        info!(&ctx, "A num is writen");
    };

    let source_port = match unsafe { (*ipv4hdr).proto } {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr = ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            u16::from_be(unsafe { (*tcphdr).source })
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr = ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            u16::from_be(unsafe { (*udphdr).source })
        }
        IpProto::Icmp => {
            info!(&ctx, "SRC IP:{:i}", source_addr);
            return Ok(xdp_action::XDP_PASS);
        }
        _ => return Err(()),
    };

    info!(&ctx, "SRC IP: {:i}, SRC PORT: {}", source_addr, source_port);

    Ok(xdp_action::XDP_PASS)
}
