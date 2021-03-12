use std::fs;
use std::collections::HashMap;
use std::vec::Vec;

use crate::con::{Connection, ConnectionSend, ConnectionGet};

pub struct Song {
    pub artist: String,
    pub album: String,
    pub name: String,
}

impl Song {
    pub fn new(artist: String, album: String, name: String) -> Song {
        Song {
            artist: artist,
            album: album,
            name: name
        }
    }

    pub fn get_path(song: &Song) -> String {
        format!("{}/{}/{}",song.artist.clone(), song.album.clone(), song.name.clone())
    }
}

impl ConnectionGet for Song {
    fn get(con: &mut Connection) -> Result<Self, String> where Self: Sized {
        let mut length = [0 as u8; 8];
        // Read lenghts of song names
        con.get(&mut length)?;
        let artist_length = usize::from_be_bytes(length);
        con.get(&mut length)?;
        let album_length = usize::from_be_bytes(length);
        con.get(&mut length)?;
        let song_length = usize::from_be_bytes(length);

        let mut artist_bytes = vec![0u8; artist_length];
        let mut album_bytes = vec![0u8; album_length];
        let mut song_bytes = vec![0u8; song_length];
        con.get(&mut artist_bytes)?;
        con.get(&mut album_bytes)?;
        con.get(&mut song_bytes)?;

        let artist = match String::from_utf8(artist_bytes) {
            Ok(artist) => artist,
            Err(message) => return Err(message.to_string())
        };

        let album = match String::from_utf8(album_bytes) {
            Ok(album) => album,
            Err(message) => return Err(message.to_string())
        };

        let song = match String::from_utf8(song_bytes) {
            Ok(song) => song,
            Err(message) => return Err(message.to_string())
        };

        Ok(Song::new(artist, album, song))
    }
}

impl ConnectionSend for Song {
    fn send(&self, con: &mut Connection) -> Result<usize, String> {
        let bytes = con.send(&usize::to_be_bytes(self.artist.len()))?;
        con.send(&usize::to_be_bytes(self.album.len()))?;
        con.send(&usize::to_be_bytes(self.name.len()))?;
        con.send(&mut self.artist.as_bytes())?;
        con.send(&mut self.album.as_bytes())?;
        Ok(bytes + con.send(&mut self.name.as_bytes())?)
    }
}

pub struct Album {
    songs: Vec<Song>,
    name: String
}

impl Album {
    pub fn new(name: String) -> Album {
        Album {
            name: name,
            songs: Vec::new()
        }
    }

    pub fn print_songs(&self) {
        for song in self.songs.iter() {
            println!("{:?}", song.name);
        }
    }

    pub fn get_song(&self, song: &String) -> Option<&Song> {
        self.songs.iter().find(|&s| s.name == *song)
    }
}

pub struct Artist {
    albums: Vec<Album>,
    name: String
}

impl Artist {
    pub fn new(name: String) -> Artist {
        Artist {
            name: name,
            albums: Vec::new()
        }
    }

    pub fn get_album(&self, album: &String) -> Option<&Album> {
        self.albums.iter().find(|&a| a.name == *album)
    }

    pub fn print_albums(&self) {
        for album in self.albums.iter() {
            println!("{:?}", album.name);
        }
    }
}

pub struct Directory {
    artists: HashMap<String, Artist>
}

impl Directory {
    pub fn new() -> Directory {
        Directory { artists: HashMap::new()}
    }

    pub fn print_artists(&self) {
        for (_, artist) in self.artists.iter() {
            println!("{:?}", artist.name);
        }
    }

    pub fn get_artist(&self, artist: &String) -> Option<&Artist>{
        self.artists.get(artist)
    }

    // Since I'm bad at rust, files are going to be stored at
    // the given base directory following the pattern 'ARTIST/ALBUM/Song.format'.
    pub fn fetch_doom(&mut self, base_dir:String) -> () {

        // this breaks if files aren't set up correctly
        let artists = fs::read_dir(base_dir).unwrap();

        // TODO: figure out how to make this shit not all PathBuf types.
        //       It shouldn't be too hard to just make these strings.
        //
        // ALSO, clean up the variable names in here. Christ nick
        for artist in artists {
            let artist = artist.unwrap();
            let mut _artist = Artist::new(artist.file_name().into_string().unwrap());
            let albums = fs::read_dir(artist.path()).unwrap();
            for album in albums {
                let album_path = album.unwrap();
                let mut _album = Album::new(album_path.file_name().into_string().unwrap());
                let songs = fs::read_dir(album_path.path()).unwrap();
                for song in songs {
                    let song_name = song.unwrap().file_name().into_string().unwrap();
                    _album.songs.push(Song::new(
                            _artist.name.clone(),
                            _album.name.clone(),
                            song_name.clone()));
                }
                _artist.albums.push(_album);
            }
            self.artists.insert(_artist.name.clone(), _artist);
        }

        // dump files for debugging
        for (_, artist) in self.artists.iter() {
            println!("{:?}", artist.name);
            for album in artist.albums.iter() {
                println!("   {:?}", album.name);
                for song in album.songs.iter() {
                    println!("        {:?}", song.name);
                }
            }
        }
    }
}
