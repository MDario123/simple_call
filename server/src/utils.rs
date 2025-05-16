use std::net::UdpSocket;

pub fn new_udp_socket() -> UdpSocket {
    UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket. All UDP ports are in use?")
}
