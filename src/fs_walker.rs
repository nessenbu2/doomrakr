pub mod fs_walker {

    use std::{fs, io};
    use std::collections::HashMap;

    // Since I'm bad at rust, files are going to be stored at
    // the given base directory following the pattern 'ARTIST/ALBUM/Song.format'.
    // This function will return things in this structure. Types TBD
    pub fn fetch_doom(base_dir:String) {
        // this breaks if files aren't set up correctly
        let mut artists_map = HashMap::new();
        let artists = fs::read_dir(base_dir).unwrap();

        for artist in artists {
            let mut albums_map = HashMap::new();
            let artist = artist.unwrap();;
            let albums = fs::read_dir(artist.path()).unwrap();
            for album in albums {
                let album = album.unwrap();
                let mut songs_vec = Vec::new();
                let songs = fs::read_dir(album.path()).unwrap();
                for song in songs {
                    songs_vec.push(song.unwrap().path());
                }
                albums_map.insert(album.path(), songs_vec);
            }
            artists_map.insert(artist.path(), albums_map);
        }

        for (artist, albums) in artists_map {
            println!("{:?}", artist.to_str());
            for (album, songs) in albums {
                println!("   {:?}", album.to_str());
                for song in songs {
                    println!("        {:?}", song.to_str());
                }
            }
        }
    }

}
