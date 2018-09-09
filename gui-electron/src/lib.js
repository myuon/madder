const unreachable = info => {
	throw new Error("unreachable!: " + info);
};

export const cast_as = (value, type) => {
	if (value instanceof type) {
		return value;
	} else {
		throw new Error(`TypeError: ${value} does not have type ${type}`);
	}
};

export class Component {
	constructor(id, component_type, start_time, length, attributes, effect) {
		this.id = id;
		this.component_type = component_type;
		this.start_time = start_time;
		this.length = length;
		this.attributes = attributes;
		this.effect = effect;
	}

	static fromObject(value) {
		return new Component(
			value.id,
			value.component_type,
			value.start_time,
			value.length,
			value.attributes,
			value.effect
		);
	}
}

export class Receiver {
	constructor(kind, callback = null) {
		this.kind = kind;
		this.callback = callback;
	}

	static discard = () => {
		return new Receiver("discard");
	};

	static receive = callback => {
		return new Receiver("receive", callback);
	};
}

export class Request {
	constructor(method, path, entity) {
		this.method = method;
		this.path = path;
		this.entity = entity;
	}

	static Get = path => {
		return new Request("Get", path, {});
	};

	static Create = (path, entity) => {
		return new Request("Create", path, entity);
	};

	static Update = (path, entity) => {
		return new Request("Update", path, entity);
	};

	static Delete = (path, entity) => {
		return new Request("Delete", path, entity);
	};
}

export class Communicator {
	constructor() {
		this.wsc = new WebSocket("ws://localhost:3000");
		this.receiverQueue = [];
		this.opened = false;

		this.wsc.onopen = () => {
			console.log("connected!");
			this.opened = true;
		};

		this.wsc.onmessage = event => {
			const r = this.receiverQueue.shift();

			if (r.kind === "discard") {
				return;
			} else if (r.kind === "receive") {
				r.callback(event.data);
			} else {
				unreachable(r);
			}
		};
	}

	send(_request, _receiver) {
		const request = cast_as(_request, Request);
		const receiver = cast_as(_receiver, Receiver);

		if (this.opened) {
			this.wsc.send(JSON.stringify(request));
			this.receiverQueue.push(receiver);
		}
	}
}
