// import { OnPulse } from "./pulsar";

export function Event() {
  let _listeners: { [name: string]: Function[] } = {};
  let _eventBuffer: string[] = [];

  function clear(event?: string) {
    if (event) {
      _listeners[event] = [];
    } else {
      _listeners = {}
    }
  }

  function listen(event: string, callback: Function) {
    _listeners[event] = (_listeners[event] || []).concat(callback);
  }

  function send(event: string) {
    _eventBuffer.push(event);
  }

  function tick() {
    for (const listeners of Object.values(_listeners)) {
      for(const listener of listeners) {
        listener.call(undefined);
      }
    }
    _eventBuffer = [];
  }

  return {
    clear,
    listen,
    send,
    tick
  }
}