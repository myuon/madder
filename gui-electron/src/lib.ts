export interface Component {
  component_type: string,
  attributes: object[],
  effect: object[],
  id: string,
  length: number,
  start_time: number,
}

interface Discard {
  kind: "discard",
}
interface Hold<D> {
  kind: "hold",
  callback: D
}
type Receiver<D> = Discard | Hold<D>

export const discard: () => Discard = () => {
  return { kind: "discard" };
}
export const hold: <D>(callback: D) => Hold<D> = (callback: any) => {
  return { kind: "hold", callback: callback };
}

export class Communicator {
  private wsc: WebSocket;
  private receiverQueue: Receiver<(response: string) => void>[];

  constructor() {
    this.wsc = new WebSocket('ws://localhost:3000');
    this.receiverQueue = [];

    this.wsc.onopen = () => {
      console.log('connected!');
    }

    this.wsc.onmessage = (event: MessageEvent) => {
      const r = this.receiverQueue.shift();
      
      if (r.kind == "discard") {
        return;
      } else if (r.kind == "hold") {
        r.callback(event.data);
      } else {
        throw new Error("unreachable!");
      }
    }
  }

  send(request: string, receiver: Receiver<(response: string) => void>) {
    this.wsc.send(request);
    this.receiverQueue.push(receiver);
  }
}
