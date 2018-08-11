import * as React from 'react';
import withStyles, { WithStyles, StyleRulesCallback } from '@material-ui/core/styles/withStyles';
import { Communicator, Component, hold } from '../lib';
import { ComponentDetail } from './component_detail';
import { CSSProperties } from '@material-ui/core/styles/withStyles';

const styles: StyleRulesCallback<"root"> = theme => ({
  root: {
  },
});

export class Timeline extends React.Component<{com: Communicator, detailed: React.RefObject<ComponentDetail>}, {components: Map<string, Component>, selected: string}> {
  constructor(props: any) {
    super(props);

    this.state = {
      components: new Map(),
      selected: null
    };
  }

  updateComponents() {
    this.props.com.send(`{
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
          const style: CSSProperties = {
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

export default withStyles(styles)(Timeline);
