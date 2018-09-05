import * as React from "react";

export default class Ruler extends React.Component {
	constructor(props) {
		super(props);

		this.canvas = React.createRef();
	}

	componentDidMount() {
		const context = this.canvas.current.getContext("2d");

		context.fillText("0", 0, 30);
		context.fillText("1000", 700, 30);
	}

	render() {
		return <canvas width="800" height="50" ref={this.canvas} />;
	}
}
