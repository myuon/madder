import './App.css';
import { Reciever, Request } from '../lib.js';
import React, { Component } from 'react';
import Button from '@material-ui/core/Button';
import Slider from '@material-ui/lab/Slider';
import Screen from './screen';
import Timeline from './timeline';
import ComponentDetail from './component_detail';

const remote = window.require('electron').remote;

class App extends Component {
  constructor(props) {
    super(props);

    this.timeline = React.createRef();
    this.componentDetail = React.createRef();
    this.screen = React.createRef();

    this.state = {
      value: 0,
    };

    window.onload = (event) => {
      this.timeline.current.updateComponents();
      this.screen.current.renderScreen(0);
    };
  }

  createVideoComponent = () => {
    const dialog = remote.dialog;

    let filenames = dialog.showOpenDialog(null, {
      properties: ['openFile'],
      title: 'Select a video file',
      defaultPath: '.',
      filters: [
        {name: 'video file', extensions: ['mp4']}
      ]
    });

    if (filenames.length > 0) {
      this.props.comm.send(Request.Create(
        '/component',
        {
          component_type: 'Video',
          start_time: 0,
          length: 100,
          data_path: filenames[0]
        }
      ), Reciever.discard());
    }
  };

  createImageComponent = () => {
    const dialog = remote.dialog;

    let filenames = dialog.showOpenDialog(null, {
      properties: ['openFile'],
      title: 'Select an image file',
      defaultPath: '.',
      filters: [
        {name: 'image file', extensions: ['png']}
      ]
    });

    if (filenames.length > 0) {
      this.props.comm.send(Request.Create(
        '/component',
        {
          component_type: 'Image',
          start_time: 0,
          length: 200,
          data_path: filenames[0]
        }
      ), Reciever.discard());
    }
  };

  updatePosition = (event, value) => {
    this.setState({ value: value });
    this.screen.current.renderScreen(value);
  };

  render() {
    return (
      <div className="App">
        <Screen comm={this.props.comm} ref={this.screen} />
        <Button variant="contained" color="primary" onClick={this.createVideoComponent}>Create VideoComponent</Button>
        <Button variant="contained" color="primary" onClick={this.createImageComponent}>Create ImageComponent</Button>
        <Slider min={0} max={1000} value={this.state.value} step={10} aria-labelledby="label" onChange={this.updatePosition} />
        <Timeline comm={this.props.comm} detailed={this.componentDetail} ref={this.timeline} />
        <ComponentDetail comm={this.props.comm} ref={this.componentDetail} />
      </div>
    );
  }
}

export default App;
