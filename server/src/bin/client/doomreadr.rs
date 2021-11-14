use std::time::SystemTime;
use std::convert::TryInto;
use std::mem;

use doomrakr::headers;
use doomrakr::headers::Header;
use doomrakr::song::Song;
use doomrakr::con::{Connection, ConnectionGet, ConnectionSend};

use crate::player::Player;

pub struct Doomreadr {
    client_id: String,
    con: Connection,
    last_hb_time: SystemTime,
    player: Player
    // may want a last_ack_time if i wanna be really robust
}

fn check_for_commands(doom: &mut Doomreadr) -> Result<(), String> {
    if doom.con.has_data() {
        let header = Header::get(&mut doom.con)?;

        println!("got header. action: {}, id: {}", header.action, header.id);
        match header.action {
            headers::SERVER_ACK => recv_ack(doom, &header),
            headers::SERVER_INIT_STREAM => init_stream(doom, &header)?,
            headers::SERVER_STREAM_CHUNK => recv_chunk(doom, &header)?,
            headers::SERVER_STREAM_FINISHED => finish_stream(doom, &header)?,
            headers::SERVER_RESUME => resume_play(doom, &header)?,
            headers::SERVER_PAUSE => pause_play(doom, &header)?,
            headers::SERVER_GET_STATUS => get_status(doom, &header)?,
            _ => return Err(format!("Didn't understand action. Reconnecting. Action: {}", header.action))
        }
        doom.last_hb_time = SystemTime::now();
    } else {
        // NOOP
    }
    Ok(())
}

fn maybe_heartbeat(doom: &mut Doomreadr) -> Result<(), String> {
    if SystemTime::now().duration_since(doom.last_hb_time).unwrap().as_secs() >= 1 {
        let msg_header = Header::new(headers::CLIENT_HB, doom.client_id.clone());
        let is_paused = doom.player.is_paused();
        let queue = doom.player.get_queue();

        msg_header.send(&mut doom.con)?;

        // Send paused status and then the number of songs in the queue
        doom.con.send(&u64::to_be_bytes(is_paused as u64))?;
        doom.con.send(&u64::to_be_bytes(queue.len().try_into().unwrap()))?;

        for song in &queue {
            song.send(&mut doom.con)?;
        }
        doom.last_hb_time = SystemTime::now();
    }
    Ok(())
}

fn recv_ack(_doom: &mut Doomreadr, _header: &Header) {
    // NOOP
}

fn init_stream(doom: &mut Doomreadr, _header: &Header) -> Result<(), String> {
    let song = Song::get(&mut doom.con)?;

    println!("Got request to stream a song");
    println!("Artist: {}, Album: {}, Song: {}", song.artist, song.album, song.name);

    if Player::is_song_cached(&song) || Player::is_song_streaming(&song) {
        let cached_song_header = Header::new(headers::CLIENT_SONG_CACHED, doom.client_id.clone());
        cached_song_header.send(&mut doom.con)?;
        doom.player.add_to_queue(song);
    } else {
        Player::init_stream(&song);
        let ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
        ack_header.send(&mut doom.con)?;
    }
    Ok(())
}

fn recv_chunk(doom: &mut Doomreadr, _header: &Header) -> Result<(), String> {

    let song = Song::get(&mut doom.con)?;

    let mut length = [0 as u8; mem::size_of::<u64>()];
    doom.con.get(&mut length)?;

    let length = u64::from_be_bytes(length);
    let mut data = vec![0 as u8; length.try_into().unwrap()];

    doom.con.get_exact(&mut data)?;

    Player::add_chunk(&song, &mut data);

    let ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
    ack_header.send(&mut doom.con)?;
    Ok(())
}

fn finish_stream(doom: &mut Doomreadr, _header: &Header) -> Result<(), String> {

    let song = Song::get(&mut doom.con)?;
    Player::complete_stream(&song);

    doom.player.add_to_queue(song);

    let ack_header = Header::new(headers::CLIENT_ACK, doom.client_id.clone());
    ack_header.send(&mut doom.con)?;
    Ok(())
}

fn resume_play(doom: &mut Doomreadr, _header: &Header) -> Result<(), String> {
    doom.player.resume();
    let response_header = Header::new(headers::CLIENT_RESUMED, doom.client_id.clone());

    response_header.send(&mut doom.con)?;
    Ok(())
}

fn pause_play(doom: &mut Doomreadr, _header: &Header) -> Result<(), String> {
    doom.player.pause();
    let response_header = Header::new(headers::CLIENT_PAUSED, doom.client_id.clone());

    response_header.send(&mut doom.con)?;
    Ok(())
}

// TODO: just use cached values
fn get_status(doom: &mut Doomreadr, _header: &Header) -> Result<(), String> {
    let is_paused = doom.player.is_paused();
    let queue = doom.player.get_queue();

    let response_header = Header::new(headers::CLIENT_STATUS, doom.client_id.clone());
    response_header.send(&mut doom.con)?;

    // Send paused status and then the number of songs in the queue
    doom.con.send(&u64::to_be_bytes(is_paused as u64))?;
    doom.con.send(&u64::to_be_bytes(queue.len().try_into().unwrap()))?;

    for song in &queue {
        song.send(&mut doom.con)?;
    }
    Ok(())
}

impl Doomreadr {
    pub fn run(&mut self) {
        let hello_header = Header::new(headers::CLIENT_HELLO, self.client_id.clone());
        hello_header.send(&mut self.con).unwrap();
        loop {
            match check_for_commands(self) {
                Ok(_) => (),
                Err(message) => {
                    // Just return on all errors. We'll reconnect anyway
                    println!("{}", message);
                    return;
                }
            }
            match maybe_heartbeat(self) {
                Ok(_) => (),
                Err(message) => {
                    // Just return on all errors. We'll reconnect anyway
                    println!("{}", message);
                    return;
                }
            }
            self.player.maybe_enquque_song();
        }
    }

    pub fn new(client_id: String, con: Connection) -> Doomreadr {
        Doomreadr{
            client_id: client_id,
            con: con,
            last_hb_time: SystemTime::now(),
            player: Player::new()
        }
    }
}

