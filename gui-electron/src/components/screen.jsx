import React, { Component } from 'react';
import { Reciever, Request } from '../lib';

export default class Screen extends Component {
	constructor(props) {
		super(props);

		this.screen = React.createRef();
		this.src = "";
	}

	renderScreen(value: number) {
		this.props.comm.send(Request.Get(`/screen/${value}`), Reciever.recieve((response) => {
			this.src = JSON.parse(response);

			const context = this.screen.current.getContext('2d');
			const image = new Image();
			image.onload = () => {
				context.drawImage(image, 0, 0, 640, 480);
			};
			image.src = this.src;
		}));
	}

	render() {
		return (
      <div>
        <canvas ref={this.screen} width="640px" height="480px"></canvas>
      </div>
		);
	}
}
