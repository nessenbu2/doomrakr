mod logger;
mod fs_walker;
mod headers;
mod doomrakr;
mod connection;

use doomrakr::Doomrakr;
use connection::Connection;
// use crate::logger::logger::log; TODO: remove this/ move it where it will be used

use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn main() {
    let addr = "127.0.0.1:6142";
    let listener = TcpListener::bind(addr).unwrap();

    println!("listening for connections");
    let mut doom = Doomrakr::new();
    doom.init();

    let doom_ref = Arc::new(Mutex::new(doom));
    Doomrakr::run(doom_ref.clone());

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

/*
   let mut artist = String::new();
   let mut album = String::new();
   let mut song = String::new();
   io::stdin().read_line(&mut artist).unwrap();
   io::stdin().read_line(&mut album).unwrap();
   io::stdin().read_line(&mut song).unwrap();
   let mut path = format!("{}/{}/{}/{}", "/home/nick/music", artist.trim(), album.trim(), song.trim());
   println!("{}", path);
   let mut file = std::fs::File::open(path).unwrap();
   loop {
   let mut val = [0 as u8; 1]; // send something to pause until reader is ready for more
   let mut data = [0 as u8; 4096];
   let read = file.read(&mut data).unwrap();
   socket.write(&mut data);
   socket.read(&mut val);
   if (read == 0) {
   break;
   }
   }
   */
