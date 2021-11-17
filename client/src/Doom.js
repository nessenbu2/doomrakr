import React, { Component } from 'react';
import ClientSelectView from './ClientSelectView.js';
import SongSelectView from './SongSelectView.js';
import ClientStatusView from './ClientStatusView.js';

class Doom extends Component {
  constructor() {
    super();
    this.state = {
      isLoaded: false,
    }
    this.clientSelected = this.clientSelected.bind(this);
    this.songSelected = this.songSelected.bind(this);
    this.fetchLatest = this.fetchLatest.bind(this);
  }

  fetchLatest() {
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

  componentDidMount() {
    this.fetchLatest();
  }

  clientSelected(clientId) {
    this.setState({
      selectedClient: clientId,
    });
  }

  songSelected(song) {
    this.setState({
      selectedClient: undefined
    });

    fetch(`/play/${this.state.selectedClient}/${song.artist}/${song.album}/${song.name}`)
      .then(
        (result) => {
          this.fetchLatest();
        }
      );
  }

  render() {
    console.log("state:");
    console.log(this.state);
    let selectView = null;
    if (!this.state.isLoaded) {
      return (
        <div>
          Not yet loaded
        </div>
      );
    } else {
      if (this.state.selectedClient === undefined) {
        selectView = <ClientSelectView clients={this.state.clients} callback={this.clientSelected}/>
      } else {
        selectView = <SongSelectView library={this.state.library} callback={this.songSelected}/>
      } 
    }
    return (
      <div>
        <ClientStatusView />
        {selectView}
      </div>
    )
  }
}

export default Doom;
