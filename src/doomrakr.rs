use crate::headers;
use crate::connection;

use headers::{Header, get_header_from_stream};
use std::{thread, time};
use connection::Connection;
use std::sync::{Arc, Mutex};

pub struct Doomrakr {
    connections: Vec<Arc<Mutex<&Connection>>>
}

impl Doomrakr {

    pub fn new() -> Doomrakr {
        Doomrakr{connections: Vec::new()}
    }

    pub fn run(mut doom: Arc<Mutex<&mut Doomrakr>>) {
        thread::spawn(|| {
            loop {
                let doom = doom.lock().unwrap();
                for con in doom.connections.iter() {
                    // do something
                }
                drop(doom);
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
    }

    // should probably be called "track_new_con"
    pub fn handle_new_con(&mut self, mut con: Arc<Mutex<&C onnection>>) {
        self.connections.push(con)
    }
}
