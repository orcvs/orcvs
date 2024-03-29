

export function euclid(steps = 8, beats = 4, rotateSteps = 0): number[] {
	let ary = [];
	let d = -1;

	for (let i = 0; i < steps; i++){
		let v = Math.floor(i * (beats / steps));
		ary[i] = v !== d ? 1 : 0;
		d = v;
	}
	if (rotateSteps){
		return rotate(ary, rotateSteps);
	}
	return ary;
}

export function rotate<T>(ary: T[], steps = 0): T[] {
  const len = ary.length;
  const offset = ((steps % len) + len) % len;
  return ary.slice(offset).concat(ary.slice(0,offset));
}

export function zip<T>(...arrays: T[][]): T[] {
  if (!arrays.length){ return [0] as T[]; }

  const ary = [] as T[];
  let len = 0;

  for (let a of arrays) {
    len = Math.max(a.length, len);
  }

  for (let i = 0; i < len; i++) {
    for (var j = 0; j < arrays.length; j++){
      let v = arrays[j][i];
			if (v !== undefined){ ary.push(v); }
    }
  }

  return ary;
}