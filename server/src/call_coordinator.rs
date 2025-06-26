use std::{
    io::Write,
    net::{SocketAddr, TcpStream, UdpSocket},
    thread,
    time::Duration,
};

use crate::utils::new_udp_socket;

// Constants

pub const SIGNAL_PARTNER_FOUND: u8 = 2;

// Types

pub struct CallSettings {
    pub relay: bool,
}

pub struct Handshake {
    tcp1: TcpStream,
    tcp2: TcpStream,

    udp1: UdpSocket,
    udp2: UdpSocket,

    client1_udp_addr: Option<SocketAddr>,
    client2_udp_addr: Option<SocketAddr>,

    retries: u8,
}

pub enum CallCoordinatorState {
    HandshakeBegin(TcpStream, TcpStream),
    Handshake(Handshake),
    Relay(UdpSocket, UdpSocket, SocketAddr, SocketAddr),
    Finished,
}

pub struct CallCoordinator {
    pub settings: CallSettings,
    pub state: CallCoordinatorState,
}

// Functions

impl CallSettings {
    pub fn merge(self, other: CallSettings) -> Self {
        Self {
            relay: self.relay || other.relay,
        }
    }
}

impl CallCoordinator {
    pub fn new(stream1: TcpStream, stream2: TcpStream, settings: CallSettings) -> Self {
        let state = CallCoordinatorState::HandshakeBegin(stream1, stream2);
        Self { settings, state }
    }

    pub fn coordinate(mut self) {
        'outer: loop {
            match self.state {
                CallCoordinatorState::HandshakeBegin(mut stream1, mut stream2) => {
                    let udp1 = new_udp_socket();
                    let udp2 = new_udp_socket();

                    let send_udp_addr = |udp: &UdpSocket, stream: &mut TcpStream| {
                        let port = udp.local_addr().unwrap().port();
                        let port = port.to_be_bytes();

                        stream
                            .write_all(&[SIGNAL_PARTNER_FOUND, port[0], port[1]])
                            .expect("Failed to write to stream");
                    };

                    send_udp_addr(&udp1, &mut stream1);
                    send_udp_addr(&udp2, &mut stream2);

                    let handshake = Handshake {
                        tcp1: stream1,
                        tcp2: stream2,
                        udp1,
                        udp2,
                        client1_udp_addr: None,
                        client2_udp_addr: None,
                        retries: 10,
                    };
                    self.state = CallCoordinatorState::Handshake(handshake);
                }
                CallCoordinatorState::Handshake(mut handshake) => {
                    handshake.retries -= 1;
                    if handshake.retries == 0 {
                        eprintln!("Failed to receive UDP addresses from clients.");
                        break 'outer;
                    }

                    let get_udp_addr = |udp: &UdpSocket| {
                        let mut buffer = [0; 0];
                        udp.set_read_timeout(Some(Duration::from_millis(200)))
                            .unwrap();

                        if let Ok((_, addr)) = udp.recv_from(&mut buffer) {
                            Some(addr)
                        } else {
                            None
                        }
                    };

                    if handshake.client1_udp_addr.is_none() {
                        handshake.client1_udp_addr = get_udp_addr(&handshake.udp1);
                    }

                    if handshake.client2_udp_addr.is_none() {
                        handshake.client2_udp_addr = get_udp_addr(&handshake.udp2);
                    }

                    if let (Some(addr1), Some(addr2)) =
                        (handshake.client1_udp_addr, handshake.client2_udp_addr)
                    {
                        if self.settings.relay {
                            self.state = CallCoordinatorState::Relay(
                                handshake.udp1,
                                handshake.udp2,
                                addr1,
                                addr2,
                            );
                        } else {
                            println!("UDP addresses received.");
                            println!("Client 1 UDP address: {}", addr1);
                            println!("Client 2 UDP address: {}", addr2);

                            let udp_addr_to_bytes = |addr: &SocketAddr| {
                                match addr {
                                    SocketAddr::V6(_) => panic!("IPv6 is not supported"),
                                    SocketAddr::V4(addr) => {
                                        // Convert the IP address to bytes
                                        let ip = addr.ip().octets();
                                        let port = addr.port().to_be_bytes();
                                        [ip[0], ip[1], ip[2], ip[3], port[0], port[1]]
                                    }
                                }
                            };

                            let send_udp_addr = |tcp: &mut TcpStream, addr: &SocketAddr| {
                                let bytes = udp_addr_to_bytes(addr);
                                tcp.write_all(&bytes)
                                    .expect("Failed to write UDP address to TCP stream");
                            };

                            // To each client, send the UDP address of their partner
                            send_udp_addr(&mut handshake.tcp1, &addr2);
                            send_udp_addr(&mut handshake.tcp2, &addr1);

                            self.state = CallCoordinatorState::Finished;
                        }
                    } else {
                        self.state = CallCoordinatorState::Handshake(handshake);
                    }
                }
                CallCoordinatorState::Relay(ref udp1, ref udp2, ref client1_addr, client2_addr) => {
                    // In relay mode, we simply relay messages between the two UDP sockets

                    udp1.set_nonblocking(true)
                        .expect("Failed to set non-blocking mode for udp1");
                    udp2.set_nonblocking(true)
                        .expect("Failed to set non-blocking mode for udp2");

                    let mut buffer = [0; 1024];

                    if let Ok((size, _)) = udp1.recv_from(&mut buffer) {
                        println!("Received message({} bytes) from address: {}", size, client1_addr);
                        udp2.send_to(&buffer[..size], client2_addr)
                            .expect("Failed to relay message from client 1 to client 2.");
                    }

                    if let Ok((size, _)) = udp2.recv_from(&mut buffer) {
                        println!("Received message({} bytes) from address: {}", size, client2_addr);
                        udp1.send_to(&buffer[..size], client1_addr)
                            .expect("Failed to relay message from client 2 to client 1.");
                    }
                }
                CallCoordinatorState::Finished => {
                    println!("Call coordination finished.");
                    break 'outer;
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}
