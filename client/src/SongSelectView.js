import React, { Component } from 'react';
import _ from 'lodash';
import Selector from './SelectorButton.js'

class SongSelectView extends Component {
  constructor(props) {
    super(props);
    this.state = {
      library: JSON.parse(props.library),
      currentSelection: "artist"
    }
    this.callback = props.callback;
    this.selectArtist = this.selectArtist.bind(this);
    this.selectAlbum = this.selectAlbum.bind(this);
    this.selectSong = this.selectSong.bind(this);
  }

  selectArtist(artist) {
    let artistObj = this.state.library.artists[artist];
    this.setState({
      selectedArtist: artistObj,
      currentSelection: "album"
    });
  }

  selectAlbum(album) {
    let albumObj = _.find(this.state.selectedArtist.albums, {name:album});
    this.setState({
      selectedAlbum: albumObj,
      currentSelection: "song"
    });
  }

  selectSong(song) {
    let songObj = _.find(this.state.selectedAlbum.songs, {name:song});
    this.callback(songObj);
  }

  render() {
    let buttons = [];
    if (this.state.currentSelection === "artist") {
      for (let name in this.state.library.artists) {
        buttons.push(<Selector key={name} name={name} callback={this.selectArtist} />);
      }
    } else if (this.state.currentSelection === "album") {
      for (const album of this.state.selectedArtist.albums) {
        buttons.push(<Selector key={album.name} name={album.name} callback={this.selectAlbum} />);
      }
    } else if (this.state.currentSelection === "song") {
      for (const song of this.state.selectedAlbum.songs) {
        buttons.push(<Selector key={song.name} name={song.name} callback={this.selectSong} />);
      }
    }
    return (
      <div>
        <p>select ur {this.state.currentSelection}.</p>
        {buttons}
      </div>
    )
  }
}

export default SongSelectView;
