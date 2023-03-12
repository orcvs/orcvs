
const memozizable = ['lerp','cycle','wave', 'seq', 'euclid'];

const cache = Cache();

export  function isMemoizable(text: string) {
  return memozizable.some( s => text === s);
}

export function key() {
  return Math.random().toString(36).slice(2);
}

export function dememoize() {
  cache.clear();
}

export function memoize(key: string, ...args: any[]) {
  if (cache.has(key)) {
    return cache.get(key);
  }
  const fn = args[0] as Function;
  const a = args.slice(1);

  const value = fn.apply(null, a);
  cache.set(key, value);
  return value;
}

export function Cache() {
  let _cache:{ [name: string]: any }  = {};

  function has(key: string) {
    return (key in _cache);
  }

  function set(key: string, value: any) {
    _cache[key] = value;
  }

  function get(key: string) {
   return _cache[key];
  }

  function clear() {
    _cache = {};
  }

  return {
    has,
    set,
    get,
    clear,
  }
}