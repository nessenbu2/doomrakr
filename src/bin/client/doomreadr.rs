use std::time::SystemTime;
use std::net::TcpStream;
use std::io::{Read, Write, BufReader};
use std::fs::OpenOptions;

use doomrakr::headers;
use doomrakr::headers::Header;
use doomrakr::song::Song;
use doomrakr::con::{Connection, ConnectionGet, ConnectionSend};

use crate::player::Player;

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

fn check_for_commands(doom: &mut Doomreadr) -> Result<(), String> {
    if (doom.con.has_data()) {
        let header = Header::get(&mut doom.con)?;

        println!("got header. action {}", header.action);
        match header.action {
            headers::SERVER_ACK => recv_ack(doom, &header),
            headers::SERVER_INIT_STREAM => init_stream(doom, &header)?,
            headers::SERVER_STREAM_CHUNK => recv_chunk(doom, &header)?,
            headers::SERVER_STREAM_FINISHED => finish_stream(doom, &header)?,
            headers::SERVER_START => start_play(doom, &header),
            headers::SERVER_PAUSE => pause_play(doom, &header),
            // TODO: skip maybe? idk
            _ => println!("Didn't understand action: {}", header.action)
        }
    } else {
        // NOOP
    }
    Ok(())
}

fn maybe_heartbeat(doom: &mut Doomreadr) -> Result<(), String> {
    if SystemTime::now().duration_since(doom.last_hb_time).unwrap().as_secs() >= 1 {
        let msg_header = Header::new(headers::CLIENT_HB, doom.client_id.clone());
        msg_header.send(&mut doom.con)?;
        doom.last_hb_time = SystemTime::now();
    }
    Ok(())
}

fn recv_ack(_doom: &mut Doomreadr, _header: &Header) {
    // NOOP
}

fn init_stream(doom: &mut Doomreadr, header: &Header) -> Result<(), String> {
    println!("got a request to init a stream. id: {}", header.id);

    let song = Song::get(&mut doom.con)?;

    println!("Got request to stream a song");
    println!("Artist: {}, Album: {}, Song: {}", song.artist, song.album, song.name);

    if Player::is_song_cached(&song) || Player::is_song_streaming(&song) {
        let cached_song_header = Header::new(headers::CLIENT_SONG_CACHED, doom.client_id.clone());
        cached_song_header.send(&mut doom.con)?;
        doom.player.play(&song);
    } else {
        Player::init_stream(&song);
        let ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
        ack_header.send(&mut doom.con)?;
    }
    Ok(())
}

fn recv_chunk(doom: &mut Doomreadr, header: &Header) -> Result<(), String> {
    // TODO: send and recv a length of the chunk
    let mut data = [0 as u8; 4096];
    let song = Song::get(&mut doom.con)?;
    let read = doom.con.get(&mut data)?;

    Player::add_chunk(&song, &mut data);

    let ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
    ack_header.send(&mut doom.con)?;
    Ok(())
}

fn finish_stream(doom: &mut Doomreadr, header: &Header) -> Result<(), String> {

    let song = Song::get(&mut doom.con)?;
    Player::complete_stream(&song);

    doom.player.play(&song);

    let ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
    ack_header.send(&mut doom.con)?;
    Ok(())
}

fn start_play(doom: &mut Doomreadr, header: &Header) {
}

fn pause_play(doom: &mut Doomreadr, header: &Header) {
}

impl Doomreadr {
    pub fn run(&mut self) {
        let hello_header = Header::new(headers::CLIENT_HELLO, self.client_id.clone());
        hello_header.send(&mut self.con).unwrap();
        loop {
            match check_for_commands(self) {
                Ok(_) => (),
                Err(message) => println!("{}", message)
            }
            match maybe_heartbeat(self) {
                Ok(_) => (),
                Err(message) => println!("{}", message)
            }
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

