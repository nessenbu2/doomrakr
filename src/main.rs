mod logger;
mod fs_walker;

use crate::logger::logger::log;
use crate::fs_walker::fs_walker::fetch_doom;

use std::net::TcpListener;

use std::fs::File;
use std::io::{self, Read};
use std::io::prelude::*;
use std::path::Path;

//use std::fs::File;

fn main() {
    log("Hello, world!\n".to_string());

    let path = Path::new("/home/nick/file_to_send.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open file {}: {}", display, why),
        Ok(file) => file
    };
    let mut contents = Vec::new();
    file.read_to_end(&mut contents);

    log("Opened file\n".to_string());

    fetch_doom("/home/nick/music".to_string());

    let addr = "127.0.0.1:6142";
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut socket) => {
                println!("New connection: {}", socket.peer_addr().unwrap());
                socket.write(&mut contents.as_mut_slice());
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
}
