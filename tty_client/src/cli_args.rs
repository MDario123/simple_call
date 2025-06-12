use std::net::IpAddr;

use clap::Parser;

/// A simple client to call using the opus protocol.
#[derive(Parser, Debug)] // requires `derive` feature
#[clap(version, about, long_about = None)]
pub struct Args {
    /// The host address of the server to connect to. Ipv6 not supported for now.
    pub host: IpAddr,

    /// The TCP port of the server to connect to.
    pub host_tcp_port: u16,

    /// The room to join. Your partner must join the same room to connect with you.
    pub room: String,

    /// Whether to relay the UDP packets through the server.
    ///
    /// The alternative is to connect directly to the partner, which might not always work.
    #[clap(long, default_value_t = false)]
    pub relay: bool,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_args() {
        use clap::CommandFactory;

        Args::command().debug_assert();
    }
}
