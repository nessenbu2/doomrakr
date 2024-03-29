// Maybe this is the rust way of doing things?
mod doom;
mod doomrakr_worker;
mod fs_walker;
mod http;

use doomrakr::headers;
use doomrakr::headers::Header;
use doomrakr::con::{Connection, ConnectionGet};

use doom::Doomrakr;
use doomrakr_worker::DoomrakrWorker;

use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> () {
    let addr = "0.0.0.0:6142";
    let listener = TcpListener::bind(addr).unwrap();

    println!("listening for connections");
    let mut doom = Doomrakr::new();
    doom.init();

    let mut doom_ref = Arc::new(Mutex::new(doom));
    Doomrakr::run(&mut doom_ref);
    http::doom_http::run(&mut doom_ref);

    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(socket) => {
                    println!("New connection: {}", socket.peer_addr().unwrap());
                    let mut con = Connection::new(socket);
                    let header = match Header::get(&mut con) {
                        Ok(header) => header,
                        Err(msg) => {
                            println!("{}", msg);
                            continue;
                        }
                    };

                    if header.action == headers::CLIENT_HELLO {
                        let (client_id, worker) = DoomrakrWorker::init_connection(con, header);
                        doom_ref.lock().unwrap().handle_new_con(client_id, worker);
                    } else if header.action == headers::CLIENT_GET_LIBRARY {
                        doom_ref.lock().unwrap().dump_dir();
                    } else if header.action == headers::CLIENT_GET_QUEUE_INFO {
                        //doom_ref.lock().unwrap().dump_status(0);
                        //TODO: fix debugging
                    } else if header.action == headers::DEBUG_HELLO {
                        println!("Got a debug msg: {}", header.id);
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

