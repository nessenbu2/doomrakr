// Maybe this is the rust way of doing things?
mod doom;
mod doomrakr_worker;
mod fs_walker;

use doomrakr::headers;
use doomrakr::headers::Header;
use doomrakr::con::{Connection, ConnectionGet};

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
                    let mut con = Connection::new(socket);
                    let header = match Header::get(&mut con) {
                        Ok(con) => con,
                        Err(msg) => {
                            println!("{}", msg);
                            continue;
                        }
                    };

                    if header.action == headers::CLIENT_HELLO {
                        let worker = DoomrakrWorker::init_connection(con, header);
                        doom_ref.lock().unwrap().handle_new_con(worker);
                    } else if header.action == headers::CLIENT_GET_LIBRARY {
                        println!("get lib");
                    } else if header.action == headers::CLIENT_GET_QUEUE_INFO {
                        println!("get queue info");
                    } else {
                        println!("Unable to understand incoming request. dropped");
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
    }
}

