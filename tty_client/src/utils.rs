use std::net::{IpAddr, SocketAddr};

pub fn addr_from_bytes(buffer: &[u8]) -> SocketAddr {
    let ip = IpAddr::from([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let port = u16::from_be_bytes([buffer[4], buffer[5]]);
    SocketAddr::new(ip, port)
}
