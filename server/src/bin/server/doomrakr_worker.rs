use std::sync::{Arc, Mutex};
use std::io::Read;
use std::ops::DerefMut;
use std::{thread, time};
use std::option::Option;
use std::fs::File;
use std::mem;
use std::convert::TryInto;
use serde::{Serialize, Deserialize};

use doomrakr::headers;
use doomrakr::headers::Header;
use doomrakr::song::Song;
use doomrakr::con::{Connection, ConnectionGet, ConnectionSend};

#[derive(PartialEq)]
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
    pub is_paused: bool,
    pub current_queue: Vec<Song>,
    // TODO: should try to hide this somewhere
    song: Option<Song>, // not valid if state is not Streaming
    file: Option<File> // also not valid if state is not Streaming
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientStatus {
    pub id: String,
    pub is_paused: bool,
    pub current_queue: Vec<Song>
}

fn doom_main(doom_ref: Arc<Mutex<DoomrakrWorker>>) {
    loop {
        let mut doom = doom_ref.lock().unwrap();

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

fn heartbeat(mut doom: &mut DoomrakrWorker) {

    // If there's no data, just wait for the next heatbeat.
    //
    // TODO: track last hb time to kill stale/dead clients that failed
    //       to disconnect for whatever reason.
    if !doom.con.has_data() {
        return;
    }

    // 
    let _header = Header::get(&mut doom.con);

    // First read 2 u64s that represent paused and number of songs
    let mut buf = [0 as u8; mem::size_of::<u64>()];
    // Read if is paused
    doom.con.get(&mut buf).unwrap();
    let is_paused = u64::from_be_bytes(buf);
    let is_paused = is_paused != 0;

    // Read number of songs
    doom.con.get(&mut buf).unwrap();
    let queue_len = u64::from_be_bytes(buf);

    let mut queue = Vec::new();
    for _ in 0..queue_len {
        let song = Song::get(&mut doom.con).unwrap();
        queue.push(song);
    }

    doom.is_paused = is_paused;
    doom.current_queue = queue;

    let ack_header = Header::new(headers::SERVER_ACK, doom.client_id.clone());
    match ack_header.send(&mut doom.con) {
        Ok(_) => (),
        Err(error) => print_and_close(&mut doom, error)
    }
}

fn send_chunk(mut doom: &mut DoomrakrWorker) {
    let mut data = [0 as u8; 4096];
    let chunk_len = doom.file.as_ref().unwrap().read(&mut data).unwrap();

    // write song "metadata"
    let song = doom.song.as_ref().unwrap();

    if chunk_len == 0 {
        println!("done reading song");
        let fin_header = Header::new(headers::SERVER_STREAM_FINISHED, doom.id.clone());
        match fin_header.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                print_and_close(&mut doom, message);
                return;
            }
        };
        match song.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                print_and_close(&mut doom, message);
                return;
            }
        };
        doom.state = State::Idle;
        doom.song = None;
        doom.file = None; // TODO: maybe clean up? Rust might just magically close the file tho
    } else {
        let chunk_header = Header::new(headers::SERVER_STREAM_CHUNK, doom.id.clone());
        match chunk_header.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                print_and_close(&mut doom, message);
                return;
            }
        };
        match song.send(&mut doom.con) {
            Ok(_) => (),
            Err(message) => {
                print_and_close(&mut doom, message);
                return;
            }
        };
         match doom.con.send(&u64::to_be_bytes(chunk_len.try_into().unwrap())) {
            Ok(_) => (),
            Err(message) => {
                print_and_close(&mut doom, message);
                return;
            }
        };
        match doom.con.send(&data[..chunk_len]) {
            Ok(_) => (),
            Err(message) => {
                print_and_close(&mut doom, message);
                return;
            }
        }
    }

    let header = Header::get(&mut doom.con).unwrap(); // TODO: return for this should be a result
    if header.action != headers::CLIENT_ACK {
        println!("got a response, but it wasn't ack. Don't freak out, but it's probably busted");
        println!("action: {}, id: {}", header.action, header.id);
    }
}

fn clean_up(_doom: &mut DoomrakrWorker) {
    // Currently do nothing but maybe we'll want to clean up state later
}

fn start_heartbeating(doom_mutex: Arc<Mutex<DoomrakrWorker>>) {
    thread::spawn(move || {
        doom_main(Arc::clone(&doom_mutex))
    });
}

fn print_and_close(doom: &mut DoomrakrWorker, message: String) {
    println!("{}", message);
    doom.state = State::Closed;
}

impl DoomrakrWorker {
    pub fn init_connection(mut con: Connection, header: Header) -> (String, Arc<Mutex<DoomrakrWorker>>)  {
        let ack_header = Header::new(headers::SERVER_ACK, header.id.clone());
        ack_header.send(&mut con).unwrap();

        let doom_mutex = Arc::new(Mutex::new(DoomrakrWorker{
            client_id: header.id.clone(),
            id: String::from("MASTER"),
            con: con,
            state: State::Idle,
            is_paused: true,
            current_queue: Vec::new(),
            song: Option::None,
            file: Option::None}));

        start_heartbeating(doom_mutex.clone());
        (header.id, doom_mutex)
    }

    pub fn resume(&mut self) -> Result<usize, String> {
        let header = Header::new(headers::SERVER_RESUME, self.id.clone());
        header.send(&mut self.con)?;

        let ack = Header::get(&mut self.con)?;
        if ack.action != headers::CLIENT_RESUMED {
            Err("Failed to pause stream".to_string())
        } else {
            Ok(0)
        }
    }

    pub fn pause(&mut self) -> Result<usize, String> {
        let header = Header::new(headers::SERVER_PAUSE, self.id.clone());
        header.send(&mut self.con)?;

        let ack = Header::get(&mut self.con)?;
        if ack.action != headers::CLIENT_PAUSED {
            Err("Failed to pause stream".to_string())
        } else {
            Ok(0)
        }
    }

    pub fn get_status(&mut self) -> ClientStatus {
        ClientStatus {
            id: self.client_id.clone(),
            is_paused: self.is_paused,
            current_queue: self.current_queue.clone()
        }
    }

    pub fn send_song(&mut self, song: Song) -> Result<usize, String> {
        let header = Header::new(headers::SERVER_INIT_STREAM, self.id.clone());
        let sent = header.send(&mut self.con)?;
        let sent = sent + song.send(&mut self.con)?;

        let ack = Header::get(&mut self.con)?;
        if ack.action == headers::CLIENT_SONG_CACHED {
            // Don't need to do anything :)
            return Ok(0);
        }

        let base_path = "/home/nick/music";
        let song_path = Song::get_path(&song);
        let path = format!("{}/{}", base_path, song_path);
        self.file = Some(std::fs::File::open(path).unwrap());
        self.song = Some(song.clone());

        self.state = State::Streaming;
        if (self.current_queue.len() == 0) {
            self.is_paused = false;
        }
        self.current_queue.push(song);

        Ok(sent)
    }

    pub fn is_closed(&self) -> bool {
        self.state == State::Closed
    }
}
