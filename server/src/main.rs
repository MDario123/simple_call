pub mod call_coordinator;
pub mod room_coordinator;
pub mod utils;

#[cfg(test)]
mod tests;

use std::{net::TcpListener, thread};

use room_coordinator::RoomCoordinator;

fn main() {
    let rooms: RoomCoordinator = RoomCoordinator::default();

    // TCP listener
    let tcp_listener = TcpListener::bind("0.0.0.0:8383")
        .expect("Failed to bind TCP listener. Most likely port 8383 is already in use.");

    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => {
                // Handle the incoming connection
                println!("New connection established.");
                let mut rooms_clone = rooms.clone();
                thread::spawn(move || {
                    rooms_clone.handle_incoming_conn(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
