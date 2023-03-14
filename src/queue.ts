
export function Queue(length = 100) {
  let _queue: number[] = [];

  function push(t: number) {
    _queue.push(t);
    if (_queue.length > length) {
      _queue.shift()
    }
  }

  function peak(index = 0) {
    return _queue[index];
  }

  function percentile() {
    const sorted = _queue.sort((a, b) => a - b);
    const index = Math.ceil(0.90 * sorted.length);
    return sorted[index].toFixed(4);
  }

  function average() {
    return _queue.reduce( (a,e,i) => (a*i+e)/(i+1)).toFixed(4);
  }

  return {
    push,
    peak,
    percentile,
    average,
    get length() {
      return _queue.length;
    }
  }
}