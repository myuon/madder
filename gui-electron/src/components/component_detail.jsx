import React, { Component } from 'react';
import withStyles from '@material-ui/core/styles/withStyles';
import TextField from '@material-ui/core/TextField';

const styles = theme => ({
  root: {
    display: 'flex',
    flexWrap: 'wrap',
    margin: '10px',
  },
});

class ComponentDetail extends Component {
	constructor(props) {
		super(props);

		this.state = {
			component: null
		};
	}

	render() {
		return (
			(this.state.component != null)
				? <div key={this.state.component.id}>
	          <p>id: {this.state.component.id}</p>
	          <p>component_type: {this.state.component.component_type}</p>
	          <p>start_time: {this.state.component.start_time}</p>
	          <p>length: {this.state.component.length}</p>
	          <p>attributes: {this.state.component.attributes.toString()}</p>
	          <p>effect: {this.state.component.effect.toString()}</p>

	          <TextField
	            label="id"
	            value={this.state.component.id}>
	          </TextField>
	          <TextField
	            label="component_type"
	            value={this.state.component.component_type}>
	          </TextField>
	          <TextField
	            label="start_time"
	            value={this.state.component.start_time}>
	          </TextField>
	          <TextField
	            label="length"
	            value={this.state.component.length}>
	          </TextField>
	        </div>
	      : <div></div>
		);
	}
}

export default withStyles(styles)(ComponentDetail);