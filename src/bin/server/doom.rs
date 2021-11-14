use std::{thread, time};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::io::stdin;

use json::{object, JsonValue};
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

fn get_action() -> Result<fn(&Doomrakr) -> Result<usize, String>, String> {
    println!("Type an action or 'help' to get a list of commands");

    let mut action = String::new();
    stdin().read_line(&mut action).unwrap();
    remove_whitespace(&mut action);
    let action = action.to_lowercase();

    if action == "add" {
        Ok(add_to_queue)
    } else if action == "resume" {
        Ok(resume)
    } else if action == "pause" {
        Ok(pause)
    } else if action == "status" {
        Ok(status)
    } else if action == "help" {
        Ok(print_command_info)
    } else  {
        Err(format!("Command not recognized: {}", action))
    }
}

fn print_command_info(_doom: &Doomrakr) -> Result<usize, String> {
    println!("commands: add, resume, pause, status, help");
    Ok(0)
}

fn add_to_queue(doom: &Doomrakr) -> Result<usize, String> {
    doom.print_client_info();
    let max_num = doom.workers.len();

    let worker_num = get_client_selection(max_num - 1)?;
    let song = doom.get_song_selection()?;

    match doom.workers.get(worker_num) {
        None => Err("Not a valid client number".to_string()),
        Some(worker_ref) => {
            println!("Sending song: {}/{}/{}", song.artist, song.album, song.name);
            worker_ref.lock().unwrap().deref_mut().send_song(song)
        }
    }
}

fn resume(doom: &Doomrakr) -> Result<usize, String> {
    // TODO: dedupe some of this code
    doom.print_client_info();
    let max_num = doom.workers.len();
    let worker_num = get_client_selection(max_num - 1)?;

    match doom.workers.get(worker_num) {
        None => Err("Not a valid client number".to_string()),
        Some(worker_ref) => {
            let mut worker = worker_ref.lock().unwrap();
            println!("Pausing: {}", worker.client_id);
            worker.deref_mut().resume()
        }
    }
}

fn pause(doom: &Doomrakr) -> Result<usize, String> {
    doom.print_client_info();
    let max_num = doom.workers.len();
    let worker_num = get_client_selection(max_num - 1)?;

    match doom.workers.get(worker_num) {
        None => Err("Not a valid client number".to_string()),
        Some(worker_ref) => {
            let mut worker = worker_ref.lock().unwrap();
            println!("Pausing: {}", worker.client_id);
            worker.deref_mut().pause()
        }
    }
}

fn status(doom: &Doomrakr) -> Result<usize, String> {
    doom.print_client_info();
    let max_num = doom.workers.len();
    let worker_num = get_client_selection(max_num - 1)?;

    let status = match doom.workers.get(worker_num) {
        None => return Err("Not a valid client number".to_string()),
        Some(worker_ref) => {
            let mut worker = worker_ref.lock().unwrap();
            println!("Status for client: {}", worker.client_id);
            worker.deref_mut().get_status()?
        }
    };

    println!("Is Paused: {}. Current queue len: {}", status.is_paused, status.current_queue.len());
    println!("");
    println!("Current queue:");
    println!("");
    for song in status.current_queue.iter() {
        println!("{}", song.name);
    }
    println!("");

    Ok(status.current_queue.len())
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
                drop(doom);

                let action = match get_action() {
                    Ok(action) => action,
                    Err(message) => {
                        println!("{}", message);
                        thread::sleep(time::Duration::from_millis(5000));
                        continue;
                    }
                };

                let doom = doom_ref.lock().unwrap();
                match action(&doom) {
                    Ok(_) => (),
                    Err(message) => println!("{}", message)
                };

                drop(doom);
                thread::sleep(time::Duration::from_millis(5000));
            }
        });
    }

    // should probably be called "track_new_con" and shouldn't take a worker.
    // just a Connection object
    pub fn handle_new_con(&mut self, worker: Arc<Mutex<DoomrakrWorker>>) {
        self.workers.push(worker.clone())
    }

    pub fn dump_dir(&self) {
        let json = serde_json::to_string(&self.dir).unwrap();
        println!("{}", json);
    }

    // TODO: client_id instead of client_num
    pub fn dump_status(&self, client_num: usize) {
        let state = match self.workers.get(client_num) {
            Some(worker_ref) => {
                let mut worker = worker_ref.lock().unwrap();
                worker.deref_mut().get_status().unwrap()
            }
            None => {
                println!("No current worker");
                return;
            }
        };
        let mut data = object!{};
        data["client_num"] = JsonValue::from(client_num);
        data["is_paused"] = JsonValue::from(state.is_paused);
        let song_names = state.current_queue.clone().iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>();
        data["current_queue"] = JsonValue::from(song_names);

        println!("{}", json::stringify(data));
    }

    pub fn get_all_status(&self) -> String {
        let dir_json = serde_json::to_string(&self.dir).unwrap();
        let mut client_json = object!{};
        for client_ref in self.workers.iter() {
            let client = client_ref.lock().unwrap();
            client_json[&client.client_id] = JsonValue::from(client.to_json());
        }
        let mut final_json = object!{};
        final_json["library"] = JsonValue::from(dir_json);
        final_json["clients"] = JsonValue::from(client_json);
        json::stringify(final_json)
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
