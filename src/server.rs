mod logger;
mod fs_walker;
mod headers;
mod doomrakr;
mod connection;
mod con;

use doomrakr::Doomrakr;
use connection::Connection;

use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn main() {
    let addr = "127.0.0.1:6142";
    let listener = TcpListener::bind(addr).unwrap();

    println!("listening for connections");
    let mut doom = Doomrakr::new();
    doom.init();

    let doom_ref = Arc::new(Mutex::new(doom));
    Doomrakr::run(&doom_ref);

    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(mut socket) => {
                    println!("New connection: {}", socket.peer_addr().unwrap());
                    doom_ref.lock().unwrap().handle_new_con(Connection::init_connection(&mut socket));
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
    }
}

