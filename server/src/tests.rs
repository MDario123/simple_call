use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
};

use sha2::{Digest, Sha512};

use crate::{SIGNAL_PARTNER_FOUND, SIGNAL_WAITING_IN_ROOM, main};

#[test]
fn end_to_end() {
    println!("Running end-to-end test...");
    // Call main in a different thread
    let _ = std::thread::spawn(|| {
        main();
    });

    // Wait for a bit to ensure the main thread is running
    std::thread::sleep(std::time::Duration::from_millis(1));

    // Connect to the TCP listener
    let mut tcp_stream1 = TcpStream::connect("127.0.0.1:8383")
        .expect("Failed to connect to TCP listener. Is the server running?");
    let mut tcp_stream2 = TcpStream::connect("127.0.0.1:8383")
        .expect("Failed to connect to TCP listener. Is the server running?");

    // Connect to room "room"
    let room_hash = Sha512::digest(b"room");

    tcp_stream1
        .write_all(&room_hash)
        .expect("Failed to write to TCP stream.");

    // Wait for a bit to ensure the server has processed the request
    std::thread::sleep(std::time::Duration::from_millis(1));

    tcp_stream2
        .write_all(&room_hash)
        .expect("Failed to write to TCP stream.");

    // Wait for a bit to ensure the server has processed the request
    std::thread::sleep(std::time::Duration::from_millis(1));

    let mut buffer = [0; 1024];

    // Read the response from the first stream
    let size = tcp_stream1
        .read(&mut buffer)
        .expect("Failed to read from TCP stream.");

    assert_eq!(
        size, 4,
        "Expected to receive 4 byte, but received {} bytes",
        size
    );

    assert_eq!(
        (buffer[0], buffer[1]),
        (SIGNAL_WAITING_IN_ROOM, SIGNAL_PARTNER_FOUND),
        "Expected {:?}, but received {:?}",
        (SIGNAL_WAITING_IN_ROOM, SIGNAL_PARTNER_FOUND),
        (buffer[0], buffer[1]),
    );

    let udp_port1 = u16::from_be_bytes([buffer[2], buffer[3]]);

    // Read the response from the second stream

    let size = tcp_stream2
        .read(&mut buffer)
        .expect("Failed to read from TCP stream.");

    assert_eq!(
        size, 3,
        "Expected to receive 4 byte, but received {} bytes",
        size
    );

    assert_eq!(
        buffer[0], SIGNAL_PARTNER_FOUND,
        "Expected {:?}, but received {:?}",
        SIGNAL_PARTNER_FOUND, buffer[0],
    );

    let udp_port2 = u16::from_be_bytes([buffer[1], buffer[2]]);

    println!("UDP ports: {} and {}", udp_port1, udp_port2);

    let udp_sock1 =
        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket. All UDP ports are in use?");
    let udp_sock2 =
        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket. All UDP ports are in use?");

    let server_udp_addr1 = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), udp_port1);
    let server_udp_addr2 = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), udp_port2);

    udp_sock1
        .send_to(&[], server_udp_addr1)
        .expect("Failed to send UDP packet.");
    udp_sock2
        .send_to(&[], server_udp_addr2)
        .expect("Failed to send UDP packet.");

    let addr_from_bytes = |buffer: &[u8]| {
        let ip = IpAddr::from([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let port = u16::from_be_bytes([buffer[4], buffer[5]]);
        SocketAddr::new(ip, port)
    };

    let udp_sock2_addr = {
        let size = tcp_stream1
            .read(&mut buffer)
            .expect("Failed to read from TCP stream.");

        assert_eq!(
            size, 6,
            "Expected to receive 6 bytes, ipv4 addr + port, but received {} bytes",
            size
        );

        addr_from_bytes(&buffer[0..6])
    };

    assert_eq!(
        udp_sock2_addr.port(),
        udp_sock2.local_addr().unwrap().port(),
        "Expected UDP port {}, but received {}",
        udp_sock2.local_addr().unwrap().port(),
        udp_sock2_addr.port()
    );

    let udp_sock1_addr = {
        let size = tcp_stream2
            .read(&mut buffer)
            .expect("Failed to read from TCP stream.");

        assert_eq!(
            size, 6,
            "Expected to receive 6 bytes, ipv4 addr + port, but received {} bytes",
            size
        );

        addr_from_bytes(&buffer[0..6])
    };

    assert_eq!(
        udp_sock1_addr.port(),
        udp_sock1.local_addr().unwrap().port(),
        "Expected UDP port {}, but received {}",
        udp_sock1.local_addr().unwrap().port(),
        udp_sock1_addr.port()
    );

    // Write to each other's UDP socket to ensure they are connected

    udp_sock1
        .send_to(&[42], udp_sock2_addr)
        .expect("Failed to send UDP packet.");

    let (size, _) = udp_sock2
        .recv_from(&mut buffer)
        .expect("Failed to receive UDP packet.");

    assert_eq!(
        size, 1,
        "Expected to receive 1 byte, but received {} bytes",
        size
    );

    assert_eq!(
        buffer[0], 42,
        "Expected to receive 42, but received {}",
        buffer[0]
    );

    udp_sock2
        .send_to(&[24], udp_sock1_addr)
        .expect("Failed to send UDP packet.");

    let (size, _) = udp_sock1
        .recv_from(&mut buffer)
        .expect("Failed to receive UDP packet.");

    assert_eq!(
        size, 1,
        "Expected to receive 1 byte, but received {} bytes",
        size
    );

    assert_eq!(
        buffer[0], 24,
        "Expected to receive 24, but received {}",
        buffer[0]
    );
}
