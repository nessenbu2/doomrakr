mod doomreadr;
mod player;

use std::net::TcpStream;
use std::time::Duration;

use doomrakr::con::Connection;

use doomreadr::Doomreadr;

fn main() {
    let mut client_id = String::new();
    // May make sense to let the conneciton handle this
    std::io::stdin().read_line(&mut client_id).unwrap();
    let stream = TcpStream::connect("localhost:6142").unwrap();
    println!("Successfully connected to server in port 6142");

    stream.set_read_timeout(Some(Duration::from_millis(500)))
            .expect("Couldn't set read timeout");

    let mut doom = Doomreadr::new(client_id.trim().to_string(), Connection::new(stream));
    doom.run();
}
