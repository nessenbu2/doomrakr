use std::{thread, time};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::io::stdin;

use doomrakr::song::Song;

use crate::fs_walker::Directory;
use crate::doomrakr_worker::DoomrakrWorker;

// String.trim() doesn't seem like it actually trims? or it only gets tabs and
// spaces but not new lines.
fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

fn get_client_selection(max_num: usize) -> Result<usize, String> {
    println!("\nEnter a connection to send a song to: ");
    let mut con_num_str  = String::new();
    stdin().read_line(&mut con_num_str).unwrap();

    let con_num = match con_num_str.trim().parse::<usize>() {
        Ok(num) => num,
        Err(error) => return Err(error.to_string())
    };

    if con_num > max_num {
        return Err(format!("Invalid connection number. select a number between 0 and {}",
                           max_num));
    }
    return Ok(con_num);
}

pub struct Doomrakr  {
    workers: Vec<Arc<Mutex<DoomrakrWorker>>>,
    dir: Directory
}

impl Doomrakr {
    fn print_client_info(&self) {
        let mut pos = 0;
        println!("Connected to {} clients", self.workers.len());
        for worker_ref in self.workers.iter() {
            let worker = worker_ref.lock().unwrap();
            println!("Client Number: {}, Client ID: {}", pos, worker.client_id);
            pos = pos + 1;
        }
    }

    pub fn new() -> Doomrakr {
        Doomrakr{workers: Vec::new(),
                 dir: Directory::new()}
    }

    pub fn run(doom: &mut Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        thread::spawn(move || {
            loop {
                let mut doom = doom_ref.lock().unwrap();
                doom.clean_closed_connections();
                if doom.workers.is_empty() {
                    drop(doom);
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }

                doom.print_client_info();
                let max_num = doom.workers.len();
                drop(doom);

                // Should probably get by ID since it's possible for a connection to be
                // removed while we block on inupt.
                let worker_num = match get_client_selection(max_num - 1) {
                    Ok(num) => num,
                    Err(error) => {
                        println!("Error: {}", error);
                        continue
                    }
                };

                // Not ideal to block on input here :(
                let doom = doom_ref.lock().unwrap();
                let song = match doom.get_song_selection() {
                    Ok(song) => song,
                    Err(error) => {
                        println!("Error: {}", error);
                        drop(doom);
                        continue
                    }
                };

                match doom.workers.get(worker_num) {
                    None => println!("Not a valid client number"),
                    Some(worker_ref) => {
                        println!("Sending song: {}/{}/{}", song.artist, song.album, song.name);
                        worker_ref.lock().unwrap().deref_mut().send_song(song);
                    }
                }

                drop(doom);
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
    }

    // should probably be called "track_new_con" and shouldn't take a worker.
    // just a Connection object
    pub fn handle_new_con(&mut self, worker: Arc<Mutex<DoomrakrWorker>>) {
        self.workers.push(worker.clone())
    }

    pub fn init(&mut self) {
        self.dir.fetch_doom("/home/nick/music".to_string())
    }

    fn clean_closed_connections(&mut self) {
        self.workers.retain(|worker| {
            !worker.lock().unwrap().is_closed() 
        })
    }

    fn get_song_selection(&self) -> Result<Song, String> {
        self.dir.print_artists();
        println!("\nEnter an artist: ");
        let mut artist_name = String::new();
        stdin().read_line(&mut artist_name).unwrap();
        remove_whitespace(&mut artist_name);

        let artist = self.dir.get_artist(&artist_name);
        if artist.is_none() {
            return Err(format!("Artist {} does not exist. Try again!", artist_name));
        }
        let artist = artist.unwrap();

        artist.print_albums();
        println!("\nEnter an album: ");
        let mut album_name = String::new();
        stdin().read_line(&mut album_name).unwrap();
        remove_whitespace(&mut album_name);

        let album = artist.get_album(&album_name);
        if album.is_none() {
            return Err(format!("Album {} does not exist. Try again!", album_name));
        }
        let album = album.unwrap();
        album.print_songs();

        let mut song_name = String::new();
        println!("\nEnter a song: ");
        stdin().read_line(&mut song_name).unwrap();
        remove_whitespace(&mut song_name);
        let song = album.get_song(&song_name);
        if song.is_none() {
            return Err(format!("Song {} does not exist. Try again!", song_name));
        }

        let song = song.unwrap();
        // TODO: there's gotta be a "rust" way of doing this. References might work but there's
        // some threading magic in rust that I don't fully understand yet
        Ok(Song::new(song.artist.clone(), song.album.clone(), song.name.clone()))
    }
}
