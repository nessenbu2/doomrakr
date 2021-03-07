use crate::headers;
use crate::player;

use std::time::SystemTime;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};
use headers::{Header, get_header_from_stream};
use player::Player;
use std::fs::OpenOptions;

enum ClientState {
    Idle,
    Playing,
    Paused,
    Closed
}

pub struct ClientConnection {
    client_id: String,
    stream: TcpStream,
    state: ClientState,
    last_hb_time: SystemTime,
    player: Player
    // may want a last_ack_time if i wanna be really robust
}

fn check_for_commands(con: &mut ClientConnection) {
    let mut peek = [0 as u8; 1];
    match con.stream.peek(&mut peek) {
        Ok(_) => {
            let header = get_header_from_stream(&mut con.stream);
            println!("got header. action {}, len: {}", header.action, header.length);
            match header.action {
                headers::SERVER_ACK => recv_ack(con, &header),
                headers::SERVER_INIT_STREAM => init_stream(con, &header),
                headers::SERVER_STREAM_CHUNK => recv_chunk(con, &header),
                headers::SERVER_STREAM_FINISHED => finish_stream(con, &header),
                headers::SERVER_START => start_play(con, &header),
                headers::SERVER_PAUSE => pause_play(con, &header),
                // TODO: skip maybe? idk
                _ => println!("Didn't understand action: {}", header.action)
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // NOOP
        }
        Err(e) => panic!("hit some IO error {}", e)
    }
}

fn maybe_heartbeat(con: &mut ClientConnection) {
    if SystemTime::now().duration_since(con.last_hb_time).unwrap().as_secs() >= 1 {
        let mut msg_header = Header{action:headers::CLIENT_HB, length: con.client_id.len()};
        msg_header.send(&mut con.stream);
        con.stream.write(con.client_id.as_bytes());
        con.last_hb_time = SystemTime::now();
    }
}

fn recv_ack(con: &mut ClientConnection, header: &Header) {
    if header.length > 0 {
        println!("header >0. idk what to do");
    }
}

fn init_stream(con: &mut ClientConnection, header: &Header) {
    println!("got a request to init a stream. len: {}", header.length);
    let mut length = [0 as u8; 8];

    // Read lenghts of song names
    con.stream.read(&mut length).unwrap(); // TODO :)
    let artist_length = usize::from_be_bytes(length);
    con.stream.read(&mut length).unwrap(); // TODO :)
    let album_length = usize::from_be_bytes(length);
    con.stream.read(&mut length).unwrap(); // TODO :)
    let song_length = usize::from_be_bytes(length);

    let mut artist_bytes = vec![0u8; artist_length];
    let mut album_bytes = vec![0u8; album_length];
    let mut song_bytes = vec![0u8; song_length];
    con.stream.read(&mut artist_bytes);
    con.stream.read(&mut album_bytes);
    con.stream.read(&mut song_bytes);
    
    let artist = String::from_utf8(artist_bytes).unwrap(); // :)
    let album = String::from_utf8(album_bytes).unwrap(); // :)
    let song = String::from_utf8(song_bytes).unwrap(); // :)

    println!("Got request to stream a song");
    println!("Artist: {}, Album: {}, Song: {}", artist, album, song);

    let mut ack_header = Header{action:headers::CLIENT_ACK, length:0};
    ack_header.send(&mut con.stream);
}

fn recv_chunk(con: &mut ClientConnection, header: &Header) {
    let mut data = [0 as u8; 4096];
    let read = con.stream.read(&mut data).unwrap();

    let mut file = OpenOptions::new().append(true).create(true).open("song.ogg").unwrap();
    if (read == 0) {
        file.flush();
    } else {
        file.write(&mut data);
    }

    let mut ack_header = Header{action:headers::CLIENT_ACK, length:0};
    ack_header.send(&mut con.stream);
}

fn finish_stream(con: &mut ClientConnection, header: &Header) {
    // TODO: handle music in a different class
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("song.ogg").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    let mut ack_header = Header{action:headers::CLIENT_ACK, length:0};
    ack_header.send(&mut con.stream);
    sink.play();
}

fn start_play(con: &mut ClientConnection, header: &Header) {
}

fn pause_play(con: &mut ClientConnection, header: &Header) {
}

impl ClientConnection {
    pub fn run(&mut self) {
        let mut hello_header = Header{action:headers::CLIENT_HELLO, length: self.client_id.len()};
        hello_header.send(&mut self.stream).unwrap();
        self.stream.write(self.client_id.as_bytes());
        loop {
            check_for_commands(self);
            maybe_heartbeat(self);
        }
    }

    pub fn new(client_id: String, stream: TcpStream) -> ClientConnection {
        ClientConnection{
            client_id: client_id,
            stream: stream,
            state: ClientState::Idle,
            last_hb_time: SystemTime::now(),
            player: Player::new()
        }
    }
}

