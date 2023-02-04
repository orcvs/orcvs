import { importFromString } from 'module-from-string'

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

export async function codify(source: string) {
  const module = `
    export const code = () => {
      ${source}
    }
  `
  const { code } = await importFromString(module, { useCurrentGlobal: true });
  return code;
}
