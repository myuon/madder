import * as React from 'react';
import withStyles, { WithStyles, StyleRulesCallback } from '@material-ui/core/styles/withStyles';
import TextField from '@material-ui/core/TextField/index';
import { Component } from '../types';

const styles: StyleRulesCallback<"root"> = theme => ({
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  margin: {
    margin: theme.spacing.unit,
  },
  textField: {
    flexBasis: 200,
  }
});

class ComponentDetail extends React.Component<WithStyles<'root'>, {comp: Component}> {
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

          <TextField
            label="id"
            value={this.state.comp.id}>
          </TextField>
          <TextField
            label="component_type"
            value={this.state.comp.component_type}>
          </TextField>
          <TextField
            label="start_time"
            value={this.state.comp.start_time}>
          </TextField>
          <TextField
            label="length"
            value={this.state.comp.length}>
          </TextField>
        </div>
      : <div></div>
    );
  }
}

export default withStyles(styles)(ComponentDetail);
