import React, { Component } from 'react';
import Selector from './SelectorButton.js'

class ClientView extends Component {
  constructor(props) {
    super(props);
    this.callback = props.callback;
    this.state = {
      clients: props.clients
    }
  }

  render() {
    let clientButtons = [];
    for (const client of this.state.clients) {
      let id = JSON.parse(client).id;
      clientButtons.push(<Selector key={id} name={id} callback={this.callback}/>);
    }
    return (
      <div>
        {clientButtons}
      </div>
    )
  }
}

export default ClientView;
