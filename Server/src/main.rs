use std::io;

use server::server::server::start_server;

mod server;

fn main() -> io::Result<()> {
    start_server()
}
