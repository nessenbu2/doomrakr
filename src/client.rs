mod headers;
mod player;
mod doomreadr;
mod fs_walker;
mod con;

use doomreadr::Doomreadr;
use con::Connection;
use std::net::{TcpStream};
use std::time::Duration;

fn main() {
    let mut client_id = String::new();
    // May make sense to let the conneciton handle this
    std::io::stdin().read_line(&mut client_id).unwrap();
    let stream = TcpStream::connect("localhost:6142").unwrap();
    println!("Successfully connected to server in port 6142");
    stream.set_read_timeout(Some(Duration::from_millis(500)));

    let mut doom = Doomreadr::new(client_id.trim().to_string(), Connection::new(stream));
    doom.run();
}
