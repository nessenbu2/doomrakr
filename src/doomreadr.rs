use crate::headers;
use crate::player;
use crate::fs_walker::Song;
use crate::con::{Connection, ConnectionGet, ConnectionSend};

use std::time::SystemTime;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};
use headers::Header;
use player::Player;
use std::fs::OpenOptions;

enum ClientState {
    Idle,
    Playing,
    Paused,
    Closed
}

pub struct Doomreadr {
    client_id: String,
    con: Connection,
    state: ClientState,
    last_hb_time: SystemTime,
    player: Player
    // may want a last_ack_time if i wanna be really robust
}

fn check_for_commands(doom: &mut Doomreadr) {
    if (doom.con.has_data()) {
        let header = match Header::get(&mut doom.con) {
            Ok(header) => header,
            Err(message) => {
                println!("{}", message);
                return;
            }
        };

        println!("got header. action {}", header.action);
        match header.action {
            headers::SERVER_ACK => recv_ack(doom, &header),
            headers::SERVER_INIT_STREAM => init_stream(doom, &header),
            headers::SERVER_STREAM_CHUNK => recv_chunk(doom, &header),
            headers::SERVER_STREAM_FINISHED => finish_stream(doom, &header),
            headers::SERVER_START => start_play(doom, &header),
            headers::SERVER_PAUSE => pause_play(doom, &header),
            // TODO: skip maybe? idk
            _ => println!("Didn't understand action: {}", header.action)
        }
    } else {
        // NOOP
    }
}

fn maybe_heartbeat(doom: &mut Doomreadr) {
    if SystemTime::now().duration_since(doom.last_hb_time).unwrap().as_secs() >= 1 {
        let mut msg_header = Header::new(headers::CLIENT_HB, doom.client_id.clone());
        match msg_header.send(&mut doom.con) {
            Ok(_) => (), // NOOP
            Err(message) => {
                println!("{}", message);
                return;
            }
        }
        doom.last_hb_time = SystemTime::now();
    }
}

fn recv_ack(doom: &mut Doomreadr, header: &Header) {
    // NOOP
}

fn init_stream(doom: &mut Doomreadr, header: &Header) {
    println!("got a request to init a stream. id: {}", header.id);

    let song = match Song::get(&mut doom.con) {
        Ok(song) => song,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    println!("Got request to stream a song");
    println!("Artist: {}, Album: {}, Song: {}", song.artist, song.album, song.name);

    if Player::is_song_cached(&song) || Player::is_song_streaming(&song) {
        let mut cached_song_header = Header::new(headers::CLIENT_SONG_CACHED, doom.client_id.clone());
        cached_song_header.send(&mut doom.con);
        doom.player.play(&song);
    } else {
        Player::init_stream(&song);
        let mut ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
        ack_header.send(&mut doom.con);
    }
}

fn recv_chunk(doom: &mut Doomreadr, header: &Header) {
    let mut data = [0 as u8; 4096];
    let song = Song::get(&mut doom.con).unwrap();
    let read = doom.con.get(&mut data).unwrap();

    Player::add_chunk(&song, &mut data);

    let mut ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
    ack_header.send(&mut doom.con);
}

fn finish_stream(doom: &mut Doomreadr, header: &Header) {

    let song = Song::get(&mut doom.con).unwrap();
    Player::complete_stream(&song);

    doom.player.play(&song);

    let mut ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
    ack_header.send(&mut doom.con);
}

fn start_play(doom: &mut Doomreadr, header: &Header) {
}

fn pause_play(doom: &mut Doomreadr, header: &Header) {
}

impl Doomreadr {
    pub fn run(&mut self) {
        let mut hello_header = Header::new(headers::CLIENT_HELLO, self.client_id.clone());
        hello_header.send(&mut self.con).unwrap();
        loop {
            check_for_commands(self);
            maybe_heartbeat(self);
        }
    }

    pub fn new(client_id: String, con: Connection) -> Doomreadr {
        Doomreadr{
            client_id: client_id,
            con: con,
            state: ClientState::Idle,
            last_hb_time: SystemTime::now(),
            player: Player::new()
        }
    }
}

