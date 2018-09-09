import "./App.css";
import { Component, Receiver, Request, cast_as } from "../lib.js";
import React from "react";
import Button from "@material-ui/core/Button";
import Slider from "@material-ui/lab/Slider";
import Screen from "./screen";
import Timeline from "./timeline";
import ComponentDetail from "./component_detail";
import Ruler from "./ruler";
import AddIcon from "@material-ui/icons/Add";
import NewComponent from "./new_component";

const remote = window.require ? window.require("electron").remote : null;

// App will manage component-related state
// and share the state to all other children components
class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      value: 0,
      components: new Map(),
      selected: null,
      open: false
    };

    this.timeline = React.createRef();
    this.componentDetail = React.createRef();
    this.screen = React.createRef();

    window.onload = event => {
      // After fetching components from server,
      // Components which contains components information need to be updated
      this.updateComponents();
      this.screen.current.renderScreen(0);
    };
  }

  updateComponents() {
    this.props.comm.send(
      Request.Get("/component"),
      Receiver.receive(response => {
        const comps = JSON.parse(response);

        let cmap = new Map();
        comps.forEach(_comp => {
          let comp = cast_as(Component.fromObject(_comp), Component);
          cmap.set(comp.id, comp);
        });

        this.setState({
          components: cmap
        });
      })
    );
  }

  // When updating the component state,
  // some React.Components need to be rerendered
  shouldComponentUpdate() {
    this.timeline.current.forceUpdate();
    this.componentDetail.current.forceUpdate();

    return true;
  }

  createNewComponent = (component, callback) => {
    this.props.comm.send(
      Request.Create("/component", component),
      Receiver.receive(data => {
        this.updateComponents();

        if (callback != null) {
          callback(data);
        }
      })
    );
  };

  updatePosition = (event, value) => {
    this.setState({ value: value });
    this.screen.current.renderScreen(value);
    console.log("position: ", value);
  };

  updateCurrentComponentAttribute = (key, value) => {
    let components = this.state.components;
    let current = this.state.components.get(this.state.selected);
    if (key === "start_time") {
      let start_time = parseInt(value, 10);
      current.start_time = start_time;

      components.set(current.id, current);
      this.setState({
        components: components
      });

      // Error handling...
      this.props.comm.send(
        Request.Update(`/component/${current.id}`, { start_time: start_time }),
        Receiver.discard()
      );
    } else if (key === "length") {
      let length = parseInt(value, 10);
      current.length = length;

      components.set(current.id, current);
      this.setState({
        components: components
      });

      // Error handling...
      this.props.comm.send(
        Request.Update(`/component/${current.id}`, { length: length }),
        Receiver.discard()
      );
    } else {
      throw new Error(`Invalid key: ${key}`);
    }
  };

  render() {
    return (
      <div className="App">
        <Screen comm={this.props.comm} ref={this.screen} />
        <Ruler />
        <Slider
          ref={this.slider}
          min={0}
          max={1000}
          value={this.state.value}
          step={10}
          aria-labelledby="label"
          onChange={this.updatePosition}
        />
        <Timeline
          ref={this.timeline}
          fetchComponents={() => this.state.components}
          fetchSelected={() => this.state.selected}
          onSelectComponent={id => this.setState({ selected: id })}
        />
        <ComponentDetail
          ref={this.componentDetail}
          fetchSelectedComponent={() =>
            this.state.components.get(this.state.selected)
          }
          updateCurrentComponentAttribute={this.updateCurrentComponentAttribute}
        />
        <Button
          variant="contained"
          color="primary"
          aria-label="Delete"
          onClick={() => this.setState({ open: true })}
        >
          <AddIcon />
          New Component
        </Button>
        <NewComponent
          open={this.state.open}
          onClose={() => this.setState({ open: false })}
          onSubmit={this.createNewComponent}
        />
      </div>
    );
  }
}

export default App;
