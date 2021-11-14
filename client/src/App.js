import React, { Component } from 'react';
import './App.css';

class Data extends Component {
  constructor() {
    super();
    this.state = {
      isLoaded: false,
      data: null
    }
  }

  componentDidMount() {
    fetch("/hello")
      .then(res => res.json())
      .then(
        (result) => {
          console.log(result);
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
  render() {
    if (this.state.isLoaded) {
      return (
        <p>
          {this.state.library}
          {this.state.clients}
        </p>
      )
    } else {
      return (
        <p>
          Not yet loaded
        </p>
      )
    }
  }
}

function App() {
  return (
    <div className="App">
      <h1> hello :3</h1>
      <Data />
    </div>
  );
}

export default App;
