import React, { Component } from 'react';

class ClientStatusView extends Component {
  render() {
    let formatedState = function(client) {
      console.log(client.is_paused);
      return (
        <div key={client.id}>
          <h3>{client.id}</h3>
          <p>is paused: {client.is_paused.toString()}</p>
          <p>{client.current_queue.map(song => song.name + " ")}</p>
        </div>
      )
    }

    let views = [];
    for (const client of this.props.clientInfo) {
      let clientJson = JSON.parse(client);
      views.push(formatedState(clientJson));
    }
    return (
      <div>
        {views}
      </div>
    )
  }
}

export default ClientStatusView;
