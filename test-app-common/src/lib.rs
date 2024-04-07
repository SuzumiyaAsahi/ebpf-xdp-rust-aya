#![no_std]
// 我们要传入缓冲区的数据长度
pub const PINFOLEN: usize = 9;

// 我们要传入缓冲区的数据信息
#[repr(C)]
pub struct PackageInfo {
    pub source_ip: u32,
    pub source_port: u16,
    pub destination_port: u16,
    pub proto_type: u8,
}

impl PackageInfo {
    pub fn new(
        source_ip: u32,
        source_port: u16,
        destination_port: u16,
        proto_type: u8,
    ) -> PackageInfo {
        PackageInfo {
            source_ip,
            source_port,
            destination_port,
            proto_type,
        }
    }

    // 将结构体转换为字节流，这样才能把数据出入到缓冲区中
    pub fn to_be_bytes(&self) -> [u8; PINFOLEN] {
        let mut info_buf: [u8; PINFOLEN] = [0; PINFOLEN];
        let need2move = self.source_ip.to_be_bytes();

        info_buf[..4].copy_from_slice(&need2move[..]);

        let need2move = self.source_port.to_be_bytes();

        // 太抽象了，不过确实没想到什么好方法
        info_buf[4..4 + 2].clone_from_slice(&need2move[..]);

        let need2move = self.destination_port.to_be_bytes();

        info_buf[6..8].clone_from_slice(&need2move[..]);

        let need2move = self.proto_type.to_be_bytes();

        info_buf[8..].clone_from_slice(&need2move[..]);

        info_buf
    }

    // 将字节流数据再重新恢复成结构体，这个函数是给用户区使用的。
    pub fn from_bytes(bytes: &[u8; PINFOLEN]) -> Self {
        let source_ip = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let source_port = u16::from_be_bytes([bytes[4], bytes[5]]);
        let destination_port = u16::from_be_bytes([bytes[6], bytes[7]]);
        let proto_type = bytes[8];

        PackageInfo {
            source_ip,
            source_port,
            destination_port,
            proto_type,
        }
    }

    // 最后再将从缓冲区读到的协议编号转换为具体的协议枚举
    pub fn proto_type(&self) -> ProtocalType {
        match self.proto_type {
            6 => ProtocalType::TCP,
            17 => ProtocalType::UDP,
            _ => ProtocalType::Unknown,
        }
    }
}

// 协议枚举，其实用字符串也行，不过这样扩展性更好点，虽说也没啥用。
pub enum ProtocalType {
    TCP,
    UDP,
    Unknown,
}
