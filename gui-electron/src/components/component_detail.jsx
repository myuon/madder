import * as React from 'react';
import { Request, Reciever } from '../lib';
import TextField from '@material-ui/core/TextField';

export default class ComponentDetail extends React.Component {
	constructor(props) {
		super(props);

		this.state = {
			component: null
		};
	}

	onChangeAttribute(key, event) {
		if (key === "start_time") {
			let comp = this.state.component;
			comp.start_time = event.target.value;

			this.setState({
				component: comp
			});

			this.props.comm.send(Request.Update(
				`/component/${comp.id}/attribute/${key}`,
				event.target.value
			), Reciever.discard());
		}
	}

	render() {
		return (
			(this.state.component != null)
				? <div key={this.state.component.id}>
	          <TextField
	            label="id"
	            value={this.state.component.id}
	            disabled>
	          </TextField>
	          <TextField
	            label="component_type"
	            value={this.state.component.component_type}
	            disabled>
	          </TextField>
	          <TextField
	            label="start_time"
	            value={this.state.component.start_time}
	            onChange={(value) => this.onChangeAttribute("start_time", value)}>
	          </TextField>
	          <TextField
	            label="length"
	            value={this.state.component.length}>
	          </TextField>
	          <TextField
	            label="attributes"
	            value={JSON.stringify(this.state.component.attributes)}
	            disabled>
	          </TextField>
	          <TextField
	            label="effect"
	            value={JSON.stringify(this.state.component.effect)}
	            disabled>
	          </TextField>
	        </div>
	      : <div></div>
		);
	}
}
