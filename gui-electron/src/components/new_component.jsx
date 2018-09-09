import React from "react";
import PropTypes from "prop-types";
import { withStyles } from "@material-ui/core/styles";
import Dialog from "@material-ui/core/Dialog";
import DialogTitle from "@material-ui/core/DialogTitle";
import FormControl from "@material-ui/core/FormControl";
import InputLabel from "@material-ui/core/InputLabel";
import MenuItem from "@material-ui/core/MenuItem";
import Select from "@material-ui/core/Select";
import TextField from "@material-ui/core/TextField";
import Button from "@material-ui/core/Button";
import SendIcon from "@material-ui/icons/Send";
const dialog = window.require("electron").remote.dialog;

const styles = theme => ({
	root: {
		display: "flex",
		flexWrap: "wrap"
	},
	formControl: {
		margin: theme.spacing.unit,
		minWidth: 120
	},
	selectEmpty: {
		marginTop: theme.spacing.unit * 2
	},
	textField: {
		margin: theme.spacing.unit,
		width: 200
	},
	button: {
		margin: theme.spacing.unit
	},
	extendedIcon: {
		marginLeft: theme.spacing.unit
	}
});

class NewComponent extends React.Component {
	constructor(props) {
		super(props);

		this.state = {
			component_type: "Video",
			start_time: 0,
			length: 0,
			data_path: ""
		};
	}

	componentWillMount() {
		this.initialize();
	}

	initialize() {
		this.setState({
			component_type: "Video",
			start_time: 0,
			length: 100,
			data_path: ""
		});
	}

	render() {
		const { classes } = this.props;

		return (
			<Dialog
				onClose={() => this.props.onClose()}
				area-labelledby="new-component-dialog"
				open={this.props.open}
			>
				<DialogTitle id="new-component-dialog">
					Create new component
				</DialogTitle>
				<div>
					<form className={classes.root}>
						<FormControl className={classes.formControl}>
							<InputLabel>component_type</InputLabel>
							<Select
								value={this.state.component_type}
								onChange={event =>
									this.setState({ component_type: event.target.value })
								}
							>
								<MenuItem value={"Video"}>Video</MenuItem>
								<MenuItem value={"Image"}>Image</MenuItem>
								<MenuItem value={"Sound"}>Sound</MenuItem>
							</Select>
						</FormControl>

						<TextField
							label="start_time"
							className={classes.textField}
							value={this.state.start_time}
							onChange={event =>
								this.setState({ start_time: parseInt(event.target.value, 10) })
							}
						/>
						<TextField
							label="length"
							className={classes.textField}
							value={this.state.length}
							onChange={event =>
								this.setState({ length: parseInt(event.target.value, 10) })
							}
						/>

						<Button
							vairant="contained"
							className={classes.button}
							onClick={() => {
								dialog.showOpenDialog(null, {}, paths => {
									if (paths != null && paths.length > 0) {
										this.setState({ data_path: paths[0] });
									}
								});
							}}
						>
							file: {this.state.entity === "" ? "empty" : this.state.data_path}
						</Button>

						<Button
							variant="contained"
							color="primary"
							className={classes.button}
							onClick={() => {
								this.props.onSubmit(this.state);
								this.initialize();
								this.props.onClose();
							}}
						>
							Submit
							<SendIcon className={classes.extendedIcon} />
						</Button>
					</form>
				</div>
			</Dialog>
		);
	}
}

NewComponent.propTypes = {
	onClose: PropTypes.func
};

export default withStyles(styles)(NewComponent);
