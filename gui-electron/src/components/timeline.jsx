import * as React from "react";

export default class Timeline extends React.Component {
	render() {
		const style = {
			position: "relative",
			height: "10rem",
			overflowY: "scroll"
		};

		return (
			<div className="timeline" style={style}>
				{Array.from(this.props.fetchComponents().values()).map(
					(comp, index) => {
						const style = {
							position: "absolute",
							top: index * 20,
							left: comp.start_time,
							width: comp.length,
							display: "block",
							backgroundColor:
								this.props.fetchSelected() === comp.id ? "#fcc" : "#f99"
						};

						return (
							<div
								key={comp.id}
								style={style}
								onClick={() => {
									this.props.onSelectComponent(comp.id);
								}}
							>
								{comp.id.slice(0, 5)}
							</div>
						);
					}
				)}
			</div>
		);
	}
}
