#![no_std]
pub const PINFOLEN: usize = 9;

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
    pub fn to_be_bytes(&self) -> [u8; PINFOLEN] {
        let mut info_buf: [u8; PINFOLEN] = [0; PINFOLEN];
        let need2move = self.source_ip.to_be_bytes();

        info_buf[..4].copy_from_slice(&need2move[..]);

        let need2move = self.source_port.to_be_bytes();

        info_buf[4..4 + 2].clone_from_slice(&need2move[..]);

        let need2move = self.destination_port.to_be_bytes();

        info_buf[6..8].clone_from_slice(&need2move[..]);

        let need2move = self.proto_type.to_be_bytes();

        info_buf[8..].clone_from_slice(&need2move[..]);

        info_buf
    }

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
}
