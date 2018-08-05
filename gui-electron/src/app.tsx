import * as React from "react";
import * as ReactDOM from "react-dom";
import Button from '@material-ui/core/Button';

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

class Timeline extends React.Component<{com: Communicator}, {components: string}> {
  constructor(props: any) {
    super(props);

    this.state = {components: 'init'};
  }

  updateComponents() {
    com.send(`{
      "method": "Get",
      "path": "/component",
      "entity": {}
    }`, hold((res: string) => {
      this.setState({
        components: res
      })
    }));
  }

  render() {
    return (
      <p>
        {this.state.components}
      </p>
    )
  }
}

class App extends React.Component<{com: Communicator}> {
  private timeline: React.RefObject<Timeline>;

  constructor(props: any) {
    super(props);

    this.timeline = React.createRef();

    window.onload = (event: Event) => {
      this.timeline.current.updateComponents();
    }
  }

  handleClick = () => {
    com.send(`{
      "method": "Create",
      "path": "/component",
      "entity": {
        "component_type": "Video",
        "start_time": 0,
        "length": 100
      }
    }`, discard());

    this.timeline.current.updateComponents();
  }

  render() {
    return (
      <div>
        <Button onClick={this.handleClick}>Create Component</Button>
        <Timeline com={this.props.com} ref={this.timeline} />
      </div>
    );
  }
}

const com = new Communicator();
ReactDOM.render(<App com={com} />, document.getElementById("app"));
