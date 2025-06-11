use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::call_coordinator::{CallCoordinator, CallSettings};

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

        // Always send the waiting signal, even if there is a partner.
        stream
            .write_all(&[SIGNAL_WAITING_IN_ROOM])
            .expect("Failed to write to stream");

        println!("Sent SIGNAL_WAITING_IN_ROOM");

        let mut partner_stream = {
            let mut rooms = self.rooms.lock().expect("Lock should not be poisoned");
            let partner = rooms.remove(&room_hash);

            match partner {
                None => {
                    println!("No partner found, waiting for one");
                    rooms.insert(room_hash, stream);
                    return;
                }
                Some(partner_stream) => partner_stream,
            }
        };

        let user1_settings = wait_for_preferred_settings(&mut stream);
        let user2_settings = wait_for_preferred_settings(&mut partner_stream);

        println!("Initiating call with partner");
        CallCoordinator::new(stream, partner_stream, user1_settings.merge(user2_settings))
            .coordinate();
    }
}

/// If someone writes more than 64 bytes, we might also consume them.
fn wait_for_room_hash(stream: &mut TcpStream) -> [u8; 64] {
    let mut room_hash = [0; 64];

    stream
        .read_exact(&mut room_hash)
        .expect("Failed to read room hash from stream");

    room_hash
}

fn wait_for_preferred_settings(stream: &mut TcpStream) -> CallSettings {
    let mut buffer = [0; 1];

    if stream.read_exact(&mut buffer).is_err() {
        eprintln!("Failed to read preferred settings from stream.");
        return CallSettings { relay: false };
    }

    if buffer[0] == 1 {
        CallSettings { relay: true }
    } else {
        CallSettings { relay: false }
    }
}
