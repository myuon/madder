import * as React from "react";
import * as ReactDOM from "react-dom";
import Button from '@material-ui/core/Button';
import Slider from '@material-ui/lab/Slider';
import * as electron from 'electron';
import { Communicator, hold, discard } from './lib';
import { Screen } from './components/screen';
import { Timeline } from './components/timeline';
import { ComponentDetail } from './components/component_detail';

class App extends React.Component<{com: Communicator}, {value: number}> {
  private timeline: React.RefObject<Timeline>;
  private component_detail: React.RefObject<ComponentDetail>;
  private screen: React.RefObject<Screen>;

  constructor(props: any) {
    super(props);

    this.timeline = React.createRef();
    this.component_detail = React.createRef();
    this.screen = React.createRef();

    this.state = {
      value: 0,
    };

    window.onload = (event: Event) => {
      this.timeline.current.updateComponents();
      this.screen.current.renderScreen(0);
    }
  }

  createImage = () => {
    const dialog = electron.remote.dialog;

    let filenames = dialog.showOpenDialog(null, {
        properties: ['openFile'],
        title: 'Select a text file',
        defaultPath: '.',
        filters: [
            {name: 'image file', extensions: ['png']}
        ]
    });

    if (filenames.length > 0) {
      com.send(`{
        "method": "Create",
        "path": "/component",
        "entity": {
          "component_type": "Image",
          "start_time": 0,
          "length": 100,
          "data_path": "${filenames[0]}"
        }
      }`, discard());
  
      this.timeline.current.updateComponents();
    }
  }

  handleClick = () => {
    const dialog = electron.remote.dialog;

    let filenames = dialog.showOpenDialog(null, {
        properties: ['openFile'],
        title: 'Select a text file',
        defaultPath: '.',
        filters: [
            {name: 'video file', extensions: ['mp4']}
        ]
    });

    if (filenames.length > 0) {
      com.send(`{
        "method": "Create",
        "path": "/component",
        "entity": {
          "component_type": "Video",
          "start_time": 0,
          "length": 100,
          "data_path": "${filenames[0]}"
        }
      }`, discard());
  
      this.timeline.current.updateComponents();
    }
  }

  onChange = (event: React.ChangeEvent, value: number) => {
    this.setState({ value: value });
    this.screen.current.renderScreen(value);
  }

  render() {
    return (
      <div>
        <Screen com={this.props.com} ref={this.screen} />
        <Button variant="contained" color="primary" onClick={this.handleClick}>Create VideoComponent</Button>
        <Button variant="contained" color="primary" onClick={this.createImage}>Create ImageComponent</Button>
        <Slider value={this.state.value} min={0} max={1000} step={10} aria-labelledby="label" onChange={this.onChange} />
        <Timeline com={this.props.com} detailed={this.component_detail} ref={this.timeline} />
        <ComponentDetail ref={this.component_detail} />
      </div>
    );
  }
}

const com = new Communicator();
ReactDOM.render(<App com={com} />, document.getElementById("app"));
