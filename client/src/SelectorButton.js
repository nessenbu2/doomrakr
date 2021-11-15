import React, { Component } from 'react'

class Selector extends Component {
  constructor(props) {
    super(props);
    this.callback = props.callback;
    this.state = {
      name: props.name
    }
  }

  render() {
    return(
      <button onClick={() => this.callback(this.state.name)}>{this.state.name}</button>
    )
  }
}

export default Selector
