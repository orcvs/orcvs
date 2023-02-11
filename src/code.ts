import { readFileSync } from 'fs';
import { importFromString } from 'module-from-string'
import { Bangable } from './library'


// declare global {
//   var orcvs: IOrcvs;
//   var bang: any
// }

// export function registerGlobal(orcvs: IOrcvs) {
//   globalThis.orcvs = orcvs;
//   globalThis.bang = (pattern: string, fn: any) => { 
//     orcvs.bang(pattern, fn);
//   }
// }

export function sourceFromFile(filename: string) {
  const code = readFileSync(filename, { encoding: 'utf8' });
  return code;
}


export async function codify(source: string): Promise<(() => {})> {
  if (source.includes('.orcvs.')) {
    source = sourceFromFile(source);
  }
  const module = `
    export const code = () => {
      ${source}
    }
  `
  const { code } = await importFromString(module, { useCurrentGlobal: true });
  return code;
}
