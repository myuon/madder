const unreachable = () => {
	throw new Error('unreachable!');
};

export const cast_as = (value, type) => {
	if (value instanceof type) {
		return value;
	} else {
		throw new Error(`TypeError: ${value} does not have type ${type}`);
	}
}

export class Component {
	constructor(id, component_type, start_time, length, attributes, effect) {
		this.id = id;
		this.component_type = component_type;
		this.start_time = start_time;
		this.length = length;
		this.attributes = attributes;
		this.effect = effect;
	}
}

export class Reciever {
	constructor(kind, callback = null) {
		this.kind = kind;
		this.callback = callback;
	}

	static discard = () => {
		return new Reciever('discard');
	};

	static recieve = (callback) => {
		return new Reciever('recieve', callback);
	};
}

export class Communicator {
	constructor() {
		this.wsc = new WebSocket('ws://localhost:3000');
		this.recieverQueue = [];

		this.wsc.onopen = () => {
			console.log('connected!');
		};

		this.wsc.onmessage = (event) => {
			const r = this.recieverQueue.shift();

			if (r.kind === 'discard') {
				return;
			} else if (r.kind === 'recieve') {
				r.callback(event.data);
			} else {
				unreachable();
			}
		};
	}

	send(request, _reciever) {
		const reciever = cast_as(_reciever, Reciever);

		this.wsc.send(request);
		this.recieverQueue.push(reciever);
	}
}