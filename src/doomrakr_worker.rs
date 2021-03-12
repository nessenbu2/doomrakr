use crate::headers;
use crate::fs_walker::Song;
use crate::con::{Connection, ConnectionGet, ConnectionSend};

use std::vec::Vec;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::ops::DerefMut;
use std::{thread, time};
use std::option::Option;
use headers::Header;
use std::mem;
use std::fs::File;

enum State {
    Idle,
    Streaming,
    Closed
}

pub struct DoomrakrWorker {
    pub client_id: String,
    id: String,
    con: Connection,
    state: State,
    is_changed: bool,
    // TODO: should try to hide this somewhere
    song: Option<Song>, // not valid if state is not Streaming
    file: Option<File> // also not valid if state is not Streaming
}

fn doom_main(doom_ref: Arc<Mutex<DoomrakrWorker>>) {
    loop {
        let mut doom = doom_ref.lock().unwrap();

        if doom.is_changed {
            // TODO
        }

        match doom.state {
            State::Idle => heartbeat(doom.deref_mut()),
            State::Streaming => send_chunk(doom.deref_mut()),
            State::Closed => {
                clean_up(doom.deref_mut());
                break;
            },
        }

        match doom.state {
            State::Idle => {
                drop(doom);
                thread::sleep(time::Duration::from_millis(10));
            },
            _ => ()
        }
    }
}

fn heartbeat(doom: &mut DoomrakrWorker) {

    if doom.con.has_data() {
        let header = Header::get(&mut doom.con);
        // Currently just dropping pings from the clients. Should probably manage them somehow
    }

    let mut ack_header = Header::new(headers::SERVER_ACK, doom.client_id.clone());
    match ack_header.send(&mut doom.con) {
        Ok(_) => (),
        Err(error) => println!("{}", error)
    }
}

fn send_chunk(doom: &mut DoomrakrWorker) {
    let mut data = [0 as u8; 4096];
    let chunk_len = doom.file.as_ref().unwrap().read(&mut data).unwrap();

    // write song "metadata"
    let song = doom.song.as_ref().unwrap();

    if (chunk_len == 0) {
        let mut fin_header = Header::new(headers::SERVER_STREAM_FINISHED, doom.id.clone());
        match fin_header.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                println!("{}", message);
                return;
            }
        };
        match song.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                println!("{}", message);
                return;
            }
        };
        doom.state = State::Idle;
        doom.song = None;
        doom.file = None; // TODO: maybe clean up? Rust might just magically close the file tho
    } else {
        let mut chunk_header = Header::new(headers::SERVER_STREAM_CHUNK, doom.id.clone());
        match chunk_header.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                println!("{}", message);
                return;
            }
        };
        match song.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                println!("{}", message);
                return;
            }
        };
        match doom.con.send(&data) {
            Ok(_) => (),
            Err(message) => {
                println!("{}", message);
                return;
            }
        }
    }

    let header = Header::get(&mut doom.con).unwrap(); // TODO: return for this should be a result
    if (header.action != headers::CLIENT_ACK) {
        print!("got a response, but it wasn't ack. Don't freak out, but it's probably busted");
    }
}

fn clean_up(doom: &mut DoomrakrWorker) {
    // TODO: :3
}

fn start_heartbeating(doom_mutex: Arc<Mutex<DoomrakrWorker>>) {
    thread::spawn(move || {
        doom_main(Arc::clone(&doom_mutex))
    });
}

impl DoomrakrWorker {
    pub fn init_connection(mut con: Connection) -> Arc<Mutex<DoomrakrWorker>> {
        let header = Header::get(&mut con).unwrap();
        if header.action != headers::CLIENT_HELLO {
            println!("Got header but it's not a hello? Let's see what happens. action: {} id: {}",
                     header.action, header.id);
        }

        let mut ack_header = Header::new(headers::SERVER_ACK, header.id.clone());
        ack_header.send(&mut con).unwrap();

        let doom_mutex = Arc::new(Mutex::new(DoomrakrWorker{
                                client_id: header.id,
                                id: String::from("MASTER"),
                                con: con,
                                state: State::Idle,
                                is_changed: false,
                                song: Option::None,
                                file: Option::None}));

        start_heartbeating(doom_mutex.clone());
        doom_mutex
    }

    pub fn send_song(&mut self, song: Song) {
        let header = Header::new(headers::SERVER_INIT_STREAM, self.id.clone());
        header.send(&mut self.con).unwrap();
        song.send(&mut self.con).unwrap();

        let ack = Header::get(&mut self.con).unwrap();
        if ack.action == headers::CLIENT_SONG_CACHED {
            // Don't need to do anything :)
            return;
        }

        let base_path = "/home/nick/music";
        let song_path = Song::get_path(&song);
        let path = format!("{}/{}", base_path, song_path);
        self.file = Some(std::fs::File::open(path).unwrap());
        self.song = Some(song);

        self.state = State::Streaming
    }

}
