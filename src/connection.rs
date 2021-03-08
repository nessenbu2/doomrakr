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

pub struct Connection {
    pub client_id: String,
    socket :TcpStream,
    state: State,
    is_changed: bool,
    // TODO: should try to hide this somewhere
    song: Option<Song>, // not valid if state is not Streaming
    file: Option<File> // also not valid if state is not Streaming
}

fn con_main(con_ref: Arc<Mutex<Connection>>) {
    loop {
        let mut con = con_ref.lock().unwrap();

        if con.is_changed {
            // TODO
        }

        match con.state {
            State::Idle => heartbeat(con.deref_mut()),
            State::Streaming => send_chunk(con.deref_mut()),
            State::Closed => {
                clean_up(con.deref_mut());
                break;
            },
        }

        match con.state {
            State::Idle => {
                drop(con);
                thread::sleep(time::Duration::from_millis(10));
            },
            _ => ()
        }
    }
}

fn heartbeat(con: &mut Connection) {
    let mut peek = [0 as u8; 1];
    let len = con.socket.peek(&mut peek).expect("peek failed");

    if len != 0 {
        let header = get_header_from_stream(&mut con.socket);
        let mut data_buf = Vec::new();
        data_buf.resize(header.length, 0);
        con.socket.read(&mut data_buf).expect("Was given a length of data to read but failed");
        let echo = String::from_utf8(data_buf).unwrap();
    }

    let mut ack_header = Header{action:headers::SERVER_ACK, length:0};
    ack_header.send(&mut con.socket).unwrap();
}

// TODO: implement retries
fn send_chunk(con: &mut Connection) {
    let mut data = [0 as u8; 4096];


    let chunk_len = con.file.as_ref().unwrap().read(&mut data).unwrap();
    let mut chunk_header = Header{action:headers::SERVER_STREAM_CHUNK, length:chunk_len};

    // write song "metadata"
    let song = con.song.as_ref().unwrap();

    if (chunk_len == 0) {
        let mut fin_header = Header{action:headers::SERVER_STREAM_FINISHED, length:chunk_len};
        fin_header.send(&mut con.socket).unwrap();
        con.socket.write(&usize::to_be_bytes(song.artist.len()));
        con.socket.write(&usize::to_be_bytes(song.album.len()));
        con.socket.write(&usize::to_be_bytes(song.name.len()));
        con.socket.write(&mut song.artist.as_bytes());
        con.socket.write(&mut song.album.as_bytes());
        con.socket.write(&mut song.name.as_bytes());
        con.state = State::Idle;
        con.song = None;
        con.file = None; // TODO: maybe clean up? Rust might just magically close the file tho
    } else {
        let mut chunk_header = Header{action:headers::SERVER_STREAM_CHUNK, length:chunk_len};
        chunk_header.send(&mut con.socket).unwrap();
        con.socket.write(&usize::to_be_bytes(song.artist.len()));
        con.socket.write(&usize::to_be_bytes(song.album.len()));
        con.socket.write(&usize::to_be_bytes(song.name.len()));
        con.socket.write(&mut song.artist.as_bytes());
        con.socket.write(&mut song.album.as_bytes());
        con.socket.write(&mut song.name.as_bytes());
        con.socket.write(&data);
    }

    let header = get_header_from_stream(&mut con.socket);
    if (header.action != headers::CLIENT_ACK) {
        print!("got a response, but it wasn't ack");
    }
}

fn clean_up(con: &mut Connection) {
    // TODO: :3
}

fn start_heartbeating(con_mutex: Arc<Mutex<Connection>>) {
    thread::spawn(move || {
        con_main(Arc::clone(&con_mutex))
    });
}

impl Connection {

    pub fn init_connection(mut socket: &mut TcpStream) -> Arc<Mutex<Connection>> {
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
        let con_mutex = Arc::new(Mutex::new(Connection{
                                client_id: String::from_utf8(client_id_bytes).unwrap(),
                                socket: socket.try_clone().unwrap(),
                                state: State::Idle,
                                is_changed: false,
                                song: Option::None,
                                file: Option::None}));
        start_heartbeating(con_mutex.clone());
        con_mutex
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
