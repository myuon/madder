import * as React from "react";
import { Request, Reciever } from "../lib";
import TextField from "@material-ui/core/TextField";

export default class ComponentDetail extends React.Component {
	render() {
		const component = this.props.fetchSelectedComponent();

		return component != null ? (
			<div key={component.id}>
				<TextField label="id" value={component.id} disabled />
				<TextField
					label="component_type"
					value={component.component_type}
					disabled
				/>
				<TextField
					label="start_time"
					value={component.start_time}
					onChange={event =>
						this.props.updateCurrentComponentAttribute(
							"start_time",
							event.target.value
						)
					}
				/>
				<TextField
					label="length"
					value={component.length}
					onChange={event =>
						this.props.updateCurrentComponentAttribute(
							"length",
							event.target.value
						)
					}
				/>
				<TextField
					label="attributes"
					value={JSON.stringify(component.attributes)}
					disabled
				/>
				<TextField
					label="effect"
					value={JSON.stringify(component.effect)}
					disabled
				/>
			</div>
		) : (
			<div />
		);
	}
}
