import React from "react";
import ReactDOM from "react-dom";
import "./index.css";
import App from "./components/App";
import { Communicator, Request, Receiver } from "./lib";
import registerServiceWorker from "./registerServiceWorker";
const { ipcRenderer } = window.require("electron");

const comm = new Communicator();
const app = ReactDOM.render(
	<App comm={comm} />,
	document.getElementById("app")
);

// file read/write
ipcRenderer.on("open-yaml", (event, arg) => {
	comm.send(
		Request.Update("/project/yaml", arg),
		Receiver.receive(data => {
			app.forceUpdate();
		})
	);
});

ipcRenderer.on("request-save-yaml", (event, arg) => {
	comm.send(
		Request.Get("/project/yaml"),
		Receiver.receive(data => {
			ipcRenderer.send("response-save-yaml", data);
		})
	);
});

registerServiceWorker();
