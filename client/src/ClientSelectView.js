import React, { Component } from 'react';
import Selector from './SelectorButton.js'

class ClientStatusView extends Component {
  constructor(props) {
    super(props);
    this.callback = props.callback;
    this.state = {
      clients: props.clients
    }
    this.selectClient = this.selectedClient.bind(this);
  }

  selectedClient(client) {
    this.setState({
      selectedClient: client
    })
  }

  render() {
    let buttons= [];
    if (this.state.selectedClient === undefined) {
      for (const client of this.state.clients) {
        let id = JSON.parse(client).id;
        buttons.push(<Selector key={id} name={id} callback={this.selectClient}/>);
      }
    } else {
      buttons.push(<Selector key="play" name="play"
                    callback={() => {this.callback("add", this.state.selectedClient);}} />);
      buttons.push(<Selector key="pause" name="pause"
                    callback={() => {
                        this.setState({selectedClient: undefined});
                        this.callback("pause", this.state.selectedClient);}} />);
      buttons.push(<Selector key="resume" name="resume"
                    callback={() => {
                        this.setState({selectedClient: undefined});
                        this.callback("resume", this.state.selectedClient);}} />);
    }
    return (
      <div>
        {buttons}
      </div>
    )
  }
}

export default ClientStatusView;
