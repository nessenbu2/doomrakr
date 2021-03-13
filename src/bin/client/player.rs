use std::io::{BufReader, ErrorKind, Write};
use std::collections::LinkedList;
use std::path::Path;
use std::fs::File;
use std::fs::OpenOptions;

use doomrakr::song::Song;

const BASE_PATH: &str = "/tmp/doomrakr";
const STREAMING_PATH: &str = "/tmp/doomrakr/streaming";

fn get_path_for_song(song: &Song) -> String {
    format!("{}/{}", BASE_PATH, Song::get_path(song))
}

fn get_for_stream(song: &Song) -> String {
    format!("{}/{}", STREAMING_PATH, Song::get_path(song))
}

fn get_dir_for_song(song: &Song) -> String {
    format!("{}/{}/{}", BASE_PATH, song.artist, song.album)
}

fn get_dir_for_stream(song: &Song) -> String {
    format!("{}/{}/{}", STREAMING_PATH, song.artist, song.album)
}

pub struct Player {
    queue: LinkedList<Song>,
    stream: rodio::OutputStream,
    sink: rodio::Sink
}

impl Player {
    pub fn new() -> Player {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();

        // TODO: probably check if this is a file or not?
        //       also add a helper
        match std::fs::create_dir(Path::new(BASE_PATH)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
        // Don't mess this up :)
        match std::fs::remove_dir_all(STREAMING_PATH) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    panic!("Can't purge songs that failed to stream: {}", e)
                }
            }
        };

        match std::fs::create_dir(Path::new(STREAMING_PATH)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }

        Player {
            queue: LinkedList::new(),
            stream: stream,
            sink: rodio::Sink::try_new(&handle).unwrap()
        }
    }

    pub fn play(&mut self, song: &Song) {
        let path = get_path_for_song(song);

        // TODO: handle cases where this doesn't exists
        let song_file = std::fs::File::open(path).unwrap();
        self.sink.append(rodio::Decoder::new(BufReader::new(song_file)).unwrap());
        self.sink.pause();
        println!("pauesd: {}", self.sink.is_paused());
        self.sink.play();
        println!("pauesd: {}", self.sink.is_paused());
        println!("volume: {}", self.sink.volume());
        println!("len: {}", self.sink.len());
    }

    // Rethink these. Can probably do this waaaaay smarter
    pub fn is_song_cached(song: &Song) -> bool {
        let path = get_path_for_song(song);
        Path::new(&path).exists()
    }

    pub fn is_song_streaming(song: &Song) -> bool {
        let path = get_for_stream(song);
        Path::new(&path).exists()
    }

    pub fn init_stream(song: &Song) {
        let path = get_for_stream(song);
        // Just nuke the file for now, it'll be created when we stream chunks
        match std::fs::remove_file(path) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        };

        match std::fs::create_dir_all(get_dir_for_stream(song)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
    }

    pub fn add_chunk(song: &Song, data: &mut [u8; 4096]) {
        let path = get_for_stream(song);
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap()
            .write(data) {
                Ok(_) => (),
                Err(err) => print!("{}", err.to_string())
            };
    }

    pub fn complete_stream(song: &Song) {
        match std::fs::create_dir_all(get_dir_for_song(song)) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    panic!("Can't open cache dir: {}", e);
                }
            }
        }
        let path = get_for_stream(song);
        let mv_path = get_path_for_song(song);
        match std::fs::rename(path, mv_path) {
            Ok(_) => (),
            Err(err) => println!("{}", err.to_string())
        }
    }
}

