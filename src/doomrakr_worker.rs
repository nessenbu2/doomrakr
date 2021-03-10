use crate::headers;
use crate::fs_walker::Song;

use std::vec::Vec;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::ops::DerefMut;
use std::{thread, time};
use std::option::Option;
use headers::{Header, get_header_from_stream};
use std::mem;
use std::fs::File;

enum State {
    Idle,
    Streaming,
    Closed
}

pub struct DoomrakrWorker {
    pub client_id: String,
    socket :TcpStream,
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
    let mut peek = [0 as u8; 1];
    let len = doom.socket.peek(&mut peek).expect("peek failed");

    if len != 0 {
        let header = get_header_from_stream(&mut doom.socket);
        let mut data_buf = Vec::new();
        data_buf.resize(header.length, 0);
        doom.socket.read(&mut data_buf).expect("Was given a length of data to read but failed");
        let echo = String::from_utf8(data_buf).unwrap();
    }

    let mut ack_header = Header{action:headers::SERVER_ACK, length:0};
    ack_header.send(&mut doom.socket).unwrap();
}

// TODO: implement retries
fn send_chunk(doom: &mut DoomrakrWorker) {
    let mut data = [0 as u8; 4096];


    let chunk_len = doom.file.as_ref().unwrap().read(&mut data).unwrap();
    let mut chunk_header = Header{action:headers::SERVER_STREAM_CHUNK, length:chunk_len};

    // write song "metadata"
    let song = doom.song.as_ref().unwrap();

    if (chunk_len == 0) {
        let mut fin_header = Header{action:headers::SERVER_STREAM_FINISHED, length:chunk_len};
        fin_header.send(&mut doom.socket).unwrap();
        doom.socket.write(&usize::to_be_bytes(song.artist.len()));
        doom.socket.write(&usize::to_be_bytes(song.album.len()));
        doom.socket.write(&usize::to_be_bytes(song.name.len()));
        doom.socket.write(&mut song.artist.as_bytes());
        doom.socket.write(&mut song.album.as_bytes());
        doom.socket.write(&mut song.name.as_bytes());
        doom.state = State::Idle;
        doom.song = None;
        doom.file = None; // TODO: maybe clean up? Rust might just magically close the file tho
    } else {
        let mut chunk_header = Header{action:headers::SERVER_STREAM_CHUNK, length:chunk_len};
        chunk_header.send(&mut doom.socket).unwrap();
        doom.socket.write(&usize::to_be_bytes(song.artist.len()));
        doom.socket.write(&usize::to_be_bytes(song.album.len()));
        doom.socket.write(&usize::to_be_bytes(song.name.len()));
        doom.socket.write(&mut song.artist.as_bytes());
        doom.socket.write(&mut song.album.as_bytes());
        doom.socket.write(&mut song.name.as_bytes());
        doom.socket.write(&data);
    }

    let header = get_header_from_stream(&mut doom.socket);
    if (header.action != headers::CLIENT_ACK) {
        print!("got a response, but it wasn't ack");
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

    pub fn init_connection(mut socket: &mut TcpStream) -> Arc<Mutex<DoomrakrWorker>> {
        let header = get_header_from_stream(socket);
        if header.action != headers::CLIENT_HELLO {
            println!("Got header but it's not a hello? Let's see what happens. action: {} length: {}",
                     header.action, header.length);
        }

        let mut client_id_bytes = Vec::new();
        client_id_bytes.resize(header.length, 0);

        // TODO: this expect will crash the server
        socket.read(&mut client_id_bytes).expect("Failed to read bytes");
        let mut ack_header = Header{action: 1, length:0};
        ack_header.send(&mut socket).unwrap(); // :J
        let doom_mutex = Arc::new(Mutex::new(DoomrakrWorker{
                                client_id: String::from_utf8(client_id_bytes).unwrap(),
                                socket: socket.try_clone().unwrap(),
                                state: State::Idle,
                                is_changed: false,
                                song: Option::None,
                                file: Option::None}));
        start_heartbeating(doom_mutex.clone());
        doom_mutex
    }

    // TODO: refactor this stuff to use some sort of ByteBuffer class if it exists in rust
    pub fn send_song(&mut self, song: Song) {
        let body_len = mem::size_of::<usize>() * 3 + song.artist.len() + song.album.len() + song.name.len();
        let mut init_header = Header{action:headers::SERVER_INIT_STREAM, length:0};
        init_header.send(&mut self.socket).unwrap();
        self.socket.write(&usize::to_be_bytes(song.artist.len()));
        self.socket.write(&usize::to_be_bytes(song.album.len()));
        self.socket.write(&usize::to_be_bytes(song.name.len()));
        self.socket.write(&mut song.artist.as_bytes());
        self.socket.write(&mut song.album.as_bytes());
        self.socket.write(&mut song.name.as_bytes());

        let ack = get_header_from_stream(&mut self.socket);
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
