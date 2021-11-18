use std::{thread, time};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::io::stdin;
use std::collections::HashMap;

use json::{object, array, JsonValue};
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
    workers: HashMap<String, Arc<Mutex<DoomrakrWorker>>>,
    dir: Directory
}

impl Doomrakr {
    fn print_client_info(&self) {
        let mut pos = 0;
        println!("Connected to {} clients", self.workers.len());
        for (_, worker_ref) in self.workers.iter() {
            let worker = worker_ref.lock().unwrap();
            println!("Client Number: {}, Client ID: {}", pos, worker.client_id);
            pos = pos + 1;
        }
    }

    pub fn new() -> Doomrakr {
        Doomrakr{workers: HashMap::new(),
                 dir: Directory::new()}
    }

    pub fn run(doom: &mut Arc<Mutex<Doomrakr>>) {
        let doom_ref = doom.clone();
        thread::spawn(move || {
            loop {
                let mut doom = doom_ref.lock().unwrap();
                doom.clean_closed_connections();
                drop(doom);
                thread::sleep(time::Duration::from_millis(1000));
            }
        });
    }

    // TODO: should have a Result return type
    pub fn add_song(&mut self, client_id: String, artist: String, album: String, song: String) {
        let song = Song::new(artist, album, song);
        println!("client_id: {}, artist: {}, album: {}, song: {}", client_id, song.artist, song.album, song.name);
        self.workers.get(&client_id).unwrap().lock().unwrap().deref_mut().send_song(song).unwrap();
    }

    pub fn pause_song(&mut self, client_id: String) {
        println!("pausing {}", client_id);
        self.workers.get(&client_id).unwrap().lock().unwrap().deref_mut().pause().unwrap();
    }

    // should probably be called "track_new_con" and shouldn't take a worker.
    // just a Connection object
    pub fn handle_new_con(&mut self, client_id: String, worker: Arc<Mutex<DoomrakrWorker>>) {
        self.workers.insert(client_id, worker.clone());
    }

    pub fn dump_dir(&self) {
        let json = serde_json::to_string(&self.dir).unwrap();
        println!("{}", json);
    }

    pub fn dump_status(&self, client_id: String) {
        let state = match self.workers.get(&client_id) {
            Some(worker_ref) => {
                let mut worker = worker_ref.lock().unwrap();
                worker.deref_mut().get_status()
            }
            None => {
                println!("No current worker");
                return;
            }
        };
        let mut data = object!{};
        data["client_num"] = JsonValue::from(client_id);
        data["is_paused"] = JsonValue::from(state.is_paused);
        let song_names = state.current_queue.clone().iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>();
        data["current_queue"] = JsonValue::from(song_names);

        println!("{}", json::stringify(data));
    }

    // TODO: some awkwardness here converting to json -> string -> json again
    pub fn get_all_status(&self) -> String {
        let dir_json = serde_json::to_string(&self.dir).unwrap();
        let mut client_json = array!{};
        for (client_id, client_ref) in self.workers.iter() {
            let mut client = client_ref.lock().unwrap();
            let json = JsonValue::from(serde_json::to_string(&client.get_status()).unwrap());
            client_json.push(json).unwrap();
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
        self.workers.retain(|_, worker| {
            !worker.lock().unwrap().is_closed() 
        })
    }
}
