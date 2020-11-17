mod logger;
mod fs_walker;
mod headers;
mod doomrakr;
mod connection;

use headers::{Header, get_header_from_stream};
use doomrakr::Doomrakr;
use connection::Connection;
use crate::logger::logger::log;
use crate::fs_walker::Directory;

use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;

use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;

fn main() {
    let path = Path::new("/home/nick/file_to_send.txt");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open file {}: {}", display, why),
        Ok(file) => file
    };

    log("Opened file\n".to_string());

    let mut dir = Directory::new();
    dir.fetch_doom("/home/nick/music".to_string());

    let addr = "127.0.0.1:6142";
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();

    println!("listening for connections");
    let mut connections: Vec<Connection> = Vec::new();
    let mut doom = Doomrakr::new();

    let doom_ref = Arc::new(Mutex::new(&mut doom));
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
