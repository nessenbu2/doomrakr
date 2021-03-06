use crate::headers;
use crate::connection;
use crate::fs_walker::{Directory, Song};
use crate::logger::logger::log;

use std::{thread, time};
use std::ops::{Deref, DerefMut};
use connection::Connection;
use std::sync::{Arc, Mutex};
use std::io::stdin;

// String.trim() doesn't seem like it actually trims? or it only gets tabs and
// spaces.
fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

pub struct Doomrakr  {
    connections: Vec<Arc<Mutex<Connection>>>,
    dir: Directory
}

impl Doomrakr {

    fn print_con_info(&self) {
        let mut pos = 0;
        for con_ref in self.connections.iter() {
            let con = con_ref.lock().unwrap();
            println!("pos: {}, client id: {}", pos, con.client_id);
            pos = pos + 1;
        }
    }

    pub fn new() -> Doomrakr {
        Doomrakr{connections: Vec::new(),
                 dir: Directory::new()}
    }

    pub fn run(doom: Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        thread::spawn(move || {
            loop {
                let mut doom = doom_ref.lock().unwrap();
                if doom.connections.is_empty() {
                    drop(doom);
                    log("No current connections. Sleeping");
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }

                // TODO: blocking access to doomraker here on user input might be bad
                //       but ideally there's not command line so it might be fine.
                //       I really should learn how to drop the lock -> read input -> re-lock though
                let con_num = match doom.get_client_selection() {
                    Ok(num) => num,
                    Err(error) => continue
                };

                let song = match doom.get_song_selection() {
                    Ok(song) => song,
                    Err(error) => continue
                };

                match doom.connections.get(con_num) {
                    None => println!("Not a valid connection number"),
                    Some(con_ref) => {
                        println!("Sending song: {}/{}/{}", song.artist, song.album, song.name);
                        con_ref.lock().unwrap().deref_mut().send_song(song);
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

    fn get_client_selection(&mut self) -> Result<usize, usize> {
        self.print_con_info();
        println!("\nEnter a connection to send a song to: ");
        let mut con_num_str  = String::new();
        stdin().read_line(&mut con_num_str).unwrap();

        let con_num = con_num_str.trim().parse::<usize>().unwrap();
        if con_num >= self.connections.len() || con_num < 0 {
            println!("Invalid connection number. select a number between 0 and {}",
                     self.connections.len());
            return Err(con_num);
        }
        return Ok(con_num);
    }

    fn get_song_selection(&self) -> Result<Song, String> {
        self.dir.print_artists();
        println!("\nEnter an artist: ");
        let mut artist_name = String::new();
        stdin().read_line(&mut artist_name).unwrap();
        remove_whitespace(&mut artist_name);

        let artist = self.dir.get_artist(&artist_name);
        if artist.is_none() {
            println!("Artist {} does not exist. Try again!", artist_name);
            return Err("asdf".to_string());
        }
        let artist = artist.unwrap();

        artist.print_albums();
        println!("\nEnter an album: ");
        let mut album_name = String::new();
        stdin().read_line(&mut album_name).unwrap();
        remove_whitespace(&mut album_name);

        let album = artist.get_album(&album_name);
        if album.is_none() {
            println!("Album {} does not exist. Try again!", album_name);
            return Err("asdf".to_string());
        }
        let album = album.unwrap();
        album.print_songs();

        let mut song_name = String::new();
        println!("\nEnter a song: ");
        stdin().read_line(&mut song_name).unwrap();
        remove_whitespace(&mut song_name);
        let song = album.get_song(&song_name);
        if song.is_none() {
            println!("Song {} does not exist. Try again!", song_name);
            return Err("asdf".to_string());
        }

        let song = song.unwrap();
        // TODO: there's gotta be a "rust" way of doing this. References might work but there's
        // some threading magic in rust that I don't fully understand yet
        Ok(Song::new(song.artist.clone(), song.album.clone(), song.name.clone()))
    }
}
