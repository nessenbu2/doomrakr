use crate::headers;
use crate::connection;
use crate::fs_walker::Directory;

use std::{thread, time};
use std::ops::Deref;
use connection::Connection;
use std::sync::{Arc, Mutex};
use std::io::stdin;

pub struct  Doomrakr  {
    connections: Vec<Arc<Mutex<Connection>>>,
    dir: Directory
}

fn print_con_info(doom: &Doomrakr) {
    let pos = 0;
    for con_ref in doom.connections.iter() {
        let con = con_ref.lock().unwrap();
        println!("pos: {}, client id: {}", pos, con.client_id);
    }
}

fn get_selection() -> (usize, String) {
    let mut con_num = String::new();
    let mut song_name = String::new();
    stdin().read_line(&mut con_num).unwrap();
    stdin().read_line(&mut song_name).unwrap();
    (con_num.trim().parse::<usize>().unwrap(), song_name)
}

impl Doomrakr {

    pub fn new() -> Doomrakr {
        Doomrakr{connections: Vec::new(),
                 dir: Directory::new()}
    }

    pub fn run(doom: Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        thread::spawn(move || {
            loop {
                let doom = doom_ref.lock().unwrap();
                if doom.connections.is_empty() {
                    drop(doom);
                    println!("No current connections. Sleeping");
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
                print_con_info(doom.deref());
                drop(doom); // Drop the lock while we wait on user input
                let (con_num, song_name) = get_selection();

                let doom = doom_ref.lock().unwrap();
                match doom.connections.get(con_num) {
                    None => println!("Not a valid connection number"),
                    Some(con_ref) => {
                        con_ref.lock().unwrap().deref().send_song(song_name);
                    }
                }

                drop(doom);
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
    }

    // should probably be called "track_new_con"
    pub fn handle_new_con(&mut self, con: Arc<Mutex<Connection>>) {
        self.connections.push(con.clone())
    }

    pub fn init(&mut self) {
        self.dir.fetch_doom("/home/nick/music".to_string())
    }
}
