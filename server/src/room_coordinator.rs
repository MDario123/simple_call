use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::call_coordinator::CallCoordinator;

pub const SIGNAL_WAITING_IN_ROOM: u8 = 1;

type RoomsMap = HashMap<[u8; 64], TcpStream>;
type SharedRoomsMap = Arc<Mutex<RoomsMap>>;

#[derive(Default, Clone)]
pub struct RoomCoordinator {
    rooms: SharedRoomsMap,
}

impl RoomCoordinator {
    pub fn handle_incoming_conn(&mut self, mut stream: TcpStream) {
        let room_hash = wait_for_room_hash(&mut stream);
        let partner = self.rooms.lock().unwrap().remove(&room_hash);

        // Always send the waiting signal, even if there is a partner.
        stream
            .write_all(&[SIGNAL_WAITING_IN_ROOM])
            .expect("Failed to write to stream");

        match partner {
            None => {
                self.rooms
                    .lock()
                    .expect("Lock should not be poisoned")
                    .insert(room_hash, stream);
            }
            Some(partner_stream) => {
                CallCoordinator::new(stream, partner_stream).coordinate();
            }
        }
    }
}

/// If someone writes more than 64 bytes, we might also consume them.
fn wait_for_room_hash(stream: &mut TcpStream) -> [u8; 64] {
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

    room_hash
}
