mod call;
mod cli_args;

use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
    thread::sleep,
    time::Duration,
};

use clap::Parser;
use sha2::{Digest, Sha512};

pub const SIGNAL_WAITING_IN_ROOM: u8 = 1;
pub const SIGNAL_PARTNER_FOUND: u8 = 2;
pub const SIGNAL_READY: u8 = 3;

fn main() {
    // Parse command line arguments
    let args = cli_args::Args::parse();

    let host = args.host;
    let host_tcp_port = args.host_tcp_port;
    let room = args.room;
    let relay = args.relay;

    // Create a TCP connection to the server
    let mut tcp_stream = TcpStream::connect((host, host_tcp_port))
        .expect("Failed to connect to TCP listener. Is the server running?");

    // Send server what room we want to join
    let room_hash = Sha512::digest(room);

    tcp_stream
        .write_all(&room_hash)
        .expect("Failed to write to TCP stream.");

    // Relay true
    tcp_stream
        .write_all(&[if relay { 1 } else { 0 }])
        .expect("Failed to write to TCP stream.");

    // Receive server udp port
    let mut buffer = [0; 1024];
    let mut patience = 100;

    let server_udp_port = 'udp_loop: loop {
        sleep(Duration::from_millis(10));

        let size = tcp_stream
            .read(&mut buffer[0..1])
            .expect("Failed to read from TCP stream.");

        if size == 1 {
            match buffer[0] {
                SIGNAL_WAITING_IN_ROOM => {
                    patience = 6000;
                }
                SIGNAL_PARTNER_FOUND => {
                    tcp_stream
                        .read_exact(&mut buffer[0..2])
                        .expect("Failed to read from TCP stream.");
                    break 'udp_loop u16::from_be_bytes([buffer[0], buffer[1]]);
                }
                SIGNAL_READY => {
                    tcp_stream
                        .write_all(&[SIGNAL_READY])
                        .expect("Failed to write to TCP stream.");
                }
                _ => panic!("Unexpected signal from server: {}", buffer[0]),
            }
        } else {
            patience -= 1;
        }

        if patience == 0 {
            panic!("Server is not responding.");
        }
    };

    let udp_sock =
        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket. All UDP ports are in use?");

    let server_udp_addr = SocketAddr::new(host, server_udp_port);

    let peer_udp_addr = if relay {
        server_udp_addr
    } else {
        // Get peer's UDP address
        udp_sock
            .send_to(&[], server_udp_addr)
            .expect("Failed to send UDP packet.");

        // Wait for a bit to ensure the server has processed the request
        std::thread::sleep(std::time::Duration::from_millis(10));

        {
            let size = tcp_stream
                .read(&mut buffer)
                .expect("Failed to read from TCP stream.");

            assert_eq!(
                size, 6,
                "Expected to receive 6 bytes, ipv4 addr + port, but received {} bytes",
                size
            );

            addr_from_bytes(&buffer[0..6])
        }
    };

    call::handle_call(udp_sock, peer_udp_addr);
}

fn addr_from_bytes(buffer: &[u8]) -> SocketAddr {
    let ip = IpAddr::from([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let port = u16::from_be_bytes([buffer[4], buffer[5]]);
    SocketAddr::new(ip, port)
}
