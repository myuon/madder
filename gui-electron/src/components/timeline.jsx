import * as React from 'react';
import { cast_as, Component, Reciever, Request } from '../lib';

export default class Timeline extends React.Component {
	constructor(props) {
		super(props);

		this.state = {
			components: new Map(),
			selected: null,
		}
	}

	updateComponents() {
		this.props.comm.send(Request.Get('/component'), Reciever.recieve((response) => {
			const comps = JSON.parse(response);

			let cmap = new Map();
			comps.forEach((_comp) => {
				let comp = cast_as(Component.fromObject(_comp), Component);
				cmap.set(comp.id, comp);
			});

			this.setState({
				components: cmap
			});
		}));
	}

	render() {
		const style = {
	  	position: 'relative',
	  	height: '10rem',
	  	overflowY: 'scroll'
	  };

    return (
      <div className="timeline" style={style}>
        {Array.from(this.state.components.values()).map((comp, index) => {
          const style = {
            position: 'absolute',
            top: index * 20,
            left: comp.start_time,
            width: comp.length,
            display: "block",
            backgroundColor: this.state.selected === comp.id ? "#fcc" : "#f99",
          };

          return <div key={comp.id} style={style} onClick={() => {
            this.setState({selected: comp.id});
            this.props.detailed.current.setState({component: comp});
          }}>{comp.id.slice(0,5)}</div>;
        })}
      </div>
    );
	}
}
