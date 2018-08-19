import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './components/App';
import { Communicator } from './lib';
import registerServiceWorker from './registerServiceWorker';

const comm = new Communicator();
ReactDOM.render(<App comm={comm} />, document.getElementById('app'));
registerServiceWorker();