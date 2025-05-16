#[cfg(test)]
mod tests;
mod utils;

use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};

use utils::*;

type RoomsMap = HashMap<[u8; 64], TcpStream>;

type SharedRoomsMap = Arc<Mutex<RoomsMap>>;

fn main() {
    let rooms: SharedRoomsMap = Arc::new(Mutex::new(HashMap::new()));

    // TCP listener
    let tcp_listener = TcpListener::bind("0.0.0.0:8383")
        .expect("Failed to bind TCP listener. Most likely port 8383 is already in use.");

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                // Handle the incoming connection
                println!("New connection established.");
                process_incoming_tcp_stream(stream, rooms.clone());
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

pub const SIGNAL_WAITING_IN_ROOM: u8 = 1;
pub const SIGNAL_PARTNER_FOUND: u8 = 2;

fn process_incoming_tcp_stream(mut stream: TcpStream, rooms: SharedRoomsMap) {
    let mut room_hash = [0; 64];
    let mut room_hash_len = 0;

    let mut buffer = [0; 1024];

    while room_hash_len < room_hash.len() {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size > 0 {
                    println!("Received {} bytes", size);
                    for i in buffer.iter().take(size) {
                        room_hash[room_hash_len] = *i;
                        room_hash_len += 1;
                    }
                } else {
                    println!("No data received.");
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
            }
        }
    }

    let partner = rooms
        .lock()
        .expect("Lock should not be poisoned")
        .remove(&room_hash);

    match partner {
        Some(partner_stream) => {
            create_connection_between_peers(stream, partner_stream);
        }
        None => {
            stream
                .write_all(&[SIGNAL_WAITING_IN_ROOM])
                .expect("Failed to write to stream");
            rooms
                .lock()
                .expect("Lock should not be poisoned")
                .insert(room_hash, stream);
        }
    }
}

fn create_connection_between_peers(mut stream1: TcpStream, mut stream2: TcpStream) {
    println!("Partner found, sending signal.");

    let udp1: UdpSocket = new_udp_socket();
    let udp2: UdpSocket = new_udp_socket();

    let send_udp_addr = |udp: &UdpSocket, stream: &mut TcpStream| {
        let port = udp.local_addr().unwrap().port();
        let port = port.to_be_bytes();

        stream
            .write_all(&[SIGNAL_PARTNER_FOUND, port[0], port[1]])
            .expect("Failed to write to stream");
    };

    send_udp_addr(&udp1, &mut stream1);
    send_udp_addr(&udp2, &mut stream2);

    println!("Both peers know their corresponding UDP ports.");

    let (client1_udp_addr, client2_udp_addr) = {
        let mut addr1 = None;
        let mut addr2 = None;

        let mut retries = 10;

        udp1.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        udp2.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

        while retries > 0 && (addr1.is_none() || addr2.is_none()) {
            retries -= 1;

            eprintln!("Waiting for UDP addresses... {} retries left", retries);

            let mut buffer = [0; 0];

            if let Ok((_, addr)) = udp1.recv_from(&mut buffer) {
                addr1 = Some(addr);
            }

            if let Ok((_, addr)) = udp2.recv_from(&mut buffer) {
                addr2 = Some(addr);
            }
        }

        (
            addr1.expect("Failed to receive UDP address from peer 1"),
            addr2.expect("Failed to receive UDP address from peer 2"),
        )
    };

    println!("UDP addresses received.");

    // Send the UDP addresses to each other
    let adrr_to_bytes = |addr: &std::net::SocketAddr| match addr {
        std::net::SocketAddr::V4(addr_v4) => {
            let ip = addr_v4.ip().octets();
            let port = addr_v4.port().to_be_bytes();
            [ip[0], ip[1], ip[2], ip[3], port[0], port[1]]
        }
        std::net::SocketAddr::V6(_) => panic!("IPv6 is not supported"),
    };

    stream1
        .write_all(&adrr_to_bytes(&client2_udp_addr))
        .expect("Failed to write to stream");
    stream2
        .write_all(&adrr_to_bytes(&client1_udp_addr))
        .expect("Failed to write to stream");
}
