// Maybe this is the rust way of doing things?
mod doom;
mod doomrakr_worker;
mod fs_walker;

use doomrakr::con::Connection;
use doom::Doomrakr;
use doomrakr_worker::DoomrakrWorker;

use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn main() {
    let addr = "127.0.0.1:6142";
    let listener = TcpListener::bind(addr).unwrap();

    println!("listening for connections");
    let mut doom = Doomrakr::new();
    doom.init();

    let mut doom_ref = Arc::new(Mutex::new(doom));
    Doomrakr::run(&mut doom_ref);

    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(socket) => {
                    println!("New connection: {}", socket.peer_addr().unwrap());
                    doom_ref.lock().unwrap().handle_new_con(
                        DoomrakrWorker::init_connection(Connection::new(socket)));
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
    }
}

