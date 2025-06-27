mod call;
mod cli_args;
mod coordination;
mod utils;

#[cfg(debug_assertions)]
use std::net::UdpSocket;

#[cfg(debug_assertions)]
use call::handle_call;
use clap::Parser;
use coordination::handle_coordination;

fn main() {
    // Parse command line arguments
    let args = cli_args::Args::parse();

    #[cfg(debug_assertions)]
    if args.test {
        println!("Running in test mode. This is not a real call.");
        let udp_sock = UdpSocket::bind("127.0.0.1:0")
            .expect("Failed to bind UDP socket. All UDP ports are in use?");
        let addr = udp_sock.local_addr().unwrap();
        handle_call(udp_sock, addr);
    }

    handle_coordination(args.host, args.host_tcp_port, args.room, args.relay);
}
