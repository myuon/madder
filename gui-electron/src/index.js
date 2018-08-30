import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './components/App';
import { Communicator, Request, Reciever } from './lib';
import registerServiceWorker from './registerServiceWorker';
const { ipcRenderer } = window.require('electron');

const comm = new Communicator();
ReactDOM.render(<App comm={comm} />, document.getElementById('app'));

// file read/write
ipcRenderer.on('open-yaml', (event, arg) => {
	comm.send(Request.Update(
		'/project/yaml',
		arg
	), Reciever.discard());
});

ipcRenderer.on('request-save-yaml', (event, arg) => {
	comm.send(Request.Get('/project/yaml'), Reciever.recieve((data) => {
		ipcRenderer.send('response-save-yaml', data);
	}));
});

registerServiceWorker();
