import React, { Component } from 'react';

class ClientStatusView extends Component {
  render() {
    let formatedState = function(clientName, queue) {
      return (
        <div key={clientName}>
          <h3>{clientName}</h3>
          <p>{queue.map(song => song.name + " ")}</p>
        </div>
      )
    }

    let views = [];
    for (const client of this.props.clientInfo) {
      let clientJson = JSON.parse(client);
      views.push(formatedState(clientJson.id, clientJson.current_queue));
    }
    return (
      <div>
        {views}
      </div>
    )
  }
}

export default ClientStatusView;
