import React, { Component } from 'react';
import ClientView from './ClientView.js';
import SongSelectView from './SongSelectView.js';

class Doom extends Component {
  constructor() {
    super();
    this.state = {
      isLoaded: false,
    }
    this.clientSelected = this.clientSelected.bind(this);
    this.songSelected = this.songSelected.bind(this);
  }

  componentDidMount() {
    fetch("/status")
      .then(res => res.json())
      .then(
        (result) => {
          this.setState({
            isLoaded: true,
            library: result.library,
            clients: result.clients
          });
        },
        (error) => {
          console.log("error loading data");
          console.log(error);
        }
      )
  }

  clientSelected(clientId) {
    this.setState({
      selectedClient: clientId,
    });
  }

  songSelected(song) {
    fetch(`/play/${this.state.selectedClient}/${song.artist}/${song.album}/${song.name}`);
    this.setState({
      selectedClient: undefined
    });
  }

  render() {
    if (this.state.isLoaded) {
      if (this.state.selectedClient === undefined) {
        return (
          <ClientView clients={this.state.clients} callback={this.clientSelected}/>
        )
      } else {
        return (
          <SongSelectView library={this.state.library} callback={this.songSelected}/>
        )
      } 
    } else {
      return (
        <p>
          Not yet loaded
        </p>
      )
    }
  }
}

export default Doom;
