use std::fs;
use std::collections::HashMap;
use std::vec::Vec;

use doomrakr::song::Song;

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
