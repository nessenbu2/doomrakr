mod doomreadr;
mod player;

use std::{thread, time};
use std::net::TcpStream;
use std::time::Duration;
use std::env;

use doomrakr::con::Connection;

use doomreadr::Doomreadr;

fn handle(stream: TcpStream, client_id: &mut String) {
    println!("Successfully connected to server in port 6142");

    stream.set_read_timeout(Some(Duration::from_millis(500)))
        .expect("Couldn't set read timeout");

    let mut doom = Doomreadr::new(client_id.trim().to_string(), Connection::new(stream));
    doom.run();

    println!("Connection ended. Sleeping...");
    thread::sleep(time::Duration::from_millis(5000));
}

fn main() {

    if env::args().count() < 3 {
        println!("usage: ./client <client_id> <server_host_address>");
        return;
    } else if env::args().count() > 3 {
        println!("usage: ./client <client_id> <server_host_address>");
        println!("You've supplied more than 2 arguments, let's see how this goes");
    }

    let mut args = env::args();
    args.next(); // Skip the binary file name
    let mut client_id = match args.next() {
        Some(val) => val,
        None => return
    };

    let hostname = match args.next() {
        Some(val) => val,
        None => return
    };

    println!("Connecting to {}:6124...", hostname);

    loop {
        match TcpStream::connect(format!("{}:6142", hostname)) {
            Ok(stream) => handle(stream, &mut client_id),
            Err(err) => {
                // Should consider having a retryable class or errors but the only one I care about
                // here seems to be ConnectionRefused. Will revisit if that's not correct.
                if err.kind() == std::io::ErrorKind::ConnectionRefused {
                    println!("Connection Refused. The server is probably not active");
                    println!("Will retry after sleeping");
                    thread::sleep(time::Duration::from_millis(5000));
                } else {
                    println!("Got an error that's probably not retryable: {:?}", err);
                    return;
                }
            }
        };
    }
}
