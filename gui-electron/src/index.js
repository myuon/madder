import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './components/App';
import { Communicator, Request, Reciever } from './lib';
import registerServiceWorker from './registerServiceWorker';

const comm = new Communicator();
const app = ReactDOM.render(<App comm={comm} />, document.getElementById('app'));

if (window.require != null) {
	const { ipcRenderer } = window.require('electron');

	// file read/write
	ipcRenderer.on('open-yaml', (event, arg) => {
		comm.send(Request.Update(
			'/project/yaml',
			arg
		), Reciever.recieve((data) => {
			app.forceUpdate();
		}));
	})

	ipcRenderer.on('request-save-yaml', (event, arg) => {
		comm.send(Request.Get('/project/yaml'), Reciever.recieve((data) => {
			ipcRenderer.send('response-save-yaml', data);
		}));
	});
}

registerServiceWorker();
