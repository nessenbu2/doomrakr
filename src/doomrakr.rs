use crate::headers;
use crate::connection;
use crate::fs_walker::Directory;

use headers::{Header, get_header_from_stream};
use std::{thread, time};
use connection::Connection;
use std::sync::{Arc, Mutex};

pub struct  Doomrakr  {
    connections: Vec<Arc<Mutex<Connection>>>,
    dir: Directory
}

impl Doomrakr {

    pub fn new() -> Doomrakr {
        Doomrakr{connections: Vec::new(),
                 dir: Directory::new()}
    }

    pub fn run(mut doom: Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        thread::spawn(move || {
            loop {
                // Get user input
                let doom = doom_ref.lock().unwrap();
                for con in doom.connections.iter() {
                    // do something
                }
                drop(doom);
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
    }

    // should probably be called "track_new_con"
    pub fn handle_new_con(&mut self, mut con: Arc<Mutex<Connection>>) {
        self.connections.push(con.clone())
    }

    pub fn init(&mut self) {
        self.dir.fetch_doom("/home/nick/music".to_string())
    }
}
