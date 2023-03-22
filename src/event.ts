
let _observers: { [name: string]: Function[] } = {}

export function send(event: string) {
  const observers = _observers[event];

  for (const observer of observers) {
    observer.call(undefined);
  }
}

export function listen(event: string, callback: Function) {
  _observers[event] = (_observers[event] || []).concat(callback);
}

export function clear(event?: string) {
  if (event) {
    _observers[event] = [];
  } else {
    _observers = {}
  }
}