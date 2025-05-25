use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
};

use sha2::{Digest, Sha512};

use crate::{
    call_coordinator::SIGNAL_PARTNER_FOUND, main, room_coordinator::SIGNAL_WAITING_IN_ROOM,
};

fn addr_from_bytes(buffer: &[u8]) -> SocketAddr {
    let ip = IpAddr::from([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let port = u16::from_be_bytes([buffer[4], buffer[5]]);
    SocketAddr::new(ip, port)
}

fn conn(room: &[u8], send_msg: &[u8], recv_msg: &[u8]) {
    let mut tcp_stream = TcpStream::connect("127.0.0.1:8383")
        .expect("Failed to connect to TCP listener. Is the server running?");

    let room_hash = Sha512::digest(room);

    tcp_stream
        .write_all(&room_hash)
        .expect("Failed to write to TCP stream.");

    tcp_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(1)))
        .expect("Failed to set read timeout.");
    tcp_stream
        .set_write_timeout(Some(std::time::Duration::from_secs(1)))
        .expect("Failed to set write timeout.");

    // Wait for a bit to ensure the server has processed the request
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Read the response from the stream

    let mut buffer = [0; 1024];

    tcp_stream
        .read_exact(&mut buffer[..4])
        .expect("Failed to read from TCP stream.");

    assert!(matches!(
        buffer[0..4],
        [SIGNAL_WAITING_IN_ROOM, SIGNAL_PARTNER_FOUND, _, _]
    ));

    let server_udp_port = u16::from_be_bytes([buffer[2], buffer[3]]);

    let udp_sock =
        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket. All UDP ports are in use?");

    udp_sock
        .set_read_timeout(Some(std::time::Duration::from_secs(1)))
        .expect("Failed to set read timeout.");

    let server_udp_addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), server_udp_port);

    udp_sock
        .send_to(&[], server_udp_addr)
        .expect("Failed to send UDP packet.");

    // Wait for a bit to ensure the server has processed the request
    std::thread::sleep(std::time::Duration::from_millis(10));

    let peer_udp_addr = {
        let size = tcp_stream
            .read(&mut buffer)
            .expect("Failed to read from TCP stream.");

        assert_eq!(
            size, 6,
            "Expected to receive 6 bytes, ipv4 addr + port, but received {} bytes",
            size
        );

        addr_from_bytes(&buffer[0..6])
    };

    udp_sock
        .send_to(send_msg, peer_udp_addr)
        .expect("Failed to send UDP packet.");

    // Wait for a bit to ensure the server has processed the request
    std::thread::sleep(std::time::Duration::from_millis(10));

    let (size, _) = udp_sock
        .recv_from(&mut buffer)
        .expect("Failed to receive UDP packet.");

    assert_eq!(
        size, 1,
        "Expected to receive 1 byte, but received {} bytes",
        size
    );

    assert_eq!(
        buffer[0], recv_msg[0],
        "Expected to receive {}, but received {}",
        recv_msg[0], buffer[0]
    );
}

#[test]
fn end_to_end() {
    println!("Running end-to-end test...");
    // Call main in a different thread
    std::thread::spawn(|| {
        main();
    });

    // Run several times to detect race conditions, has happened before
    for _ in 0..20 {
        let client1 = std::thread::spawn(move || {
            conn(b"room", &[42], &[24]);
        });

        let client2 = std::thread::spawn(move || {
            conn(b"room", &[24], &[42]);
        });

        while !client1.is_finished() || !client2.is_finished() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        client1.join().expect("Client 1 failed");
        client2.join().expect("Client 2 failed");
    }
}
