import * as React from "react";
import * as ReactDOM from "react-dom";
import Button from '@material-ui/core/Button';
import Slider from '@material-ui/lab/Slider';
import * as electron from 'electron';

interface Discard {
  kind: "discard",
}
interface Hold<D> {
  kind: "hold",
  callback: D
}
type Receiver<D> = Discard | Hold<D>

const discard: () => Discard = () => {
  return { kind: "discard" };
}
const hold: <D>(callback: D) => Hold<D> = (callback: any) => {
  return { kind: "hold", callback: callback };
}

class Communicator {
  private wsc: WebSocket;
  private receiverQueue: Receiver<(response: string) => void>[];

  constructor() {
    this.wsc = new WebSocket('ws://localhost:3000');
    this.receiverQueue = [];

    this.wsc.onopen = () => {
      console.log('connected!');
    }

    this.wsc.onmessage = (event: MessageEvent) => {
      const r = this.receiverQueue.shift();
      
      if (r.kind == "discard") {
        return;
      } else if (r.kind == "hold") {
        r.callback(event.data);
      } else {
        throw new Error("unreachable!");
      }
    }
  }

  send(request: string, receiver: Receiver<(response: string) => void>) {
    this.wsc.send(request);
    this.receiverQueue.push(receiver);
  }
}

interface Component {
  component_type: string,
  attributes: any[],
  effect: any[],
  id: string,
  length: number,
  start_time: number,
}

class Timeline extends React.Component<{com: Communicator, detailed: React.RefObject<ComponentDetail>}, {components: Map<string, Component>, selected: string}> {
  constructor(props: any) {
    super(props);

    this.state = {
      components: new Map(),
      selected: null
    };
  }

  updateComponents() {
    com.send(`{
      "method": "Get",
      "path": "/component",
      "entity": {}
    }`, hold((res: string) => {
      const comps: Component[] = JSON.parse(res);
      let cmap = new Map<string, Component>();
      comps.forEach((v) => {
        cmap.set(v.id, v);
      });

      this.setState({
        components: cmap
      });
    }));
  }

  render() {
    return (
      <div className="timeline">
        {Array.from(this.state.components.values()).map((comp, index) => {
          const style = {
            position: "absolute",
            top: index * 20,
            left: comp.start_time,
            width: comp.length,
            display: "block",
            backgroundColor: this.state.selected == comp.id ? "#fcc" : "#f99",
          };

          return <div key={comp.id} style={style} onClick={() => {
            this.setState({selected: comp.id});
            this.props.detailed.current.setState({ comp: comp });
          }}>{comp.id.slice(0,5)}</div>;
        })}
      </div>
    );
  }
}

class ComponentDetail extends React.Component<{}, {comp: Component}> {
  constructor(props: any) {
    super(props);

    this.state = { comp: null };
  }

  render() {
    return (
      (this.state.comp != null) ?
        <div key={this.state.comp.id}>
          <p>id: {this.state.comp.id}</p>
          <p>component_type: {this.state.comp.component_type}</p>
          <p>start_time: {this.state.comp.start_time}</p>
          <p>length: {this.state.comp.length}</p>
          <p>attributes: {this.state.comp.attributes.toString()}</p>
          <p>effect: {this.state.comp.effect.toString()}</p>
        </div>
      : <div></div>
    );
  }
}

class Screen extends React.Component<{com: Communicator}> {
  private screen: React.RefObject<HTMLCanvasElement>;
  private src: string;

  constructor(props: any) {
    super(props);

    this.screen = React.createRef();
    this.src = "";
  }

  renderScreen(value: number) {
    com.send(`{
      "method": "Get",
      "path": "/screen/${value}",
      "entity": ${value}
    }`, hold((res: any) => {
      this.src = JSON.parse(res);
    }));
  }

  render() {
    return (
      <div>
        <img src={this.src} />
      </div>
    );
  }
}

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
        <Slider value={this.state.value} min={0} max={10000} step={10} aria-labelledby="label" onChange={this.onChange} />
        <Timeline com={this.props.com} detailed={this.component_detail} ref={this.timeline} />
        <ComponentDetail ref={this.component_detail} />
      </div>
    );
  }
}

const com = new Communicator();
ReactDOM.render(<App com={com} />, document.getElementById("app"));
