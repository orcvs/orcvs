import { readFileSync } from 'fs';
import { importFromString } from 'module-from-string'

export function sourceFromFile(filename: string) {
  const code = readFileSync(filename, { encoding: 'utf8' });
  return code;
}

export async function codify(source: string): Promise<(() => {})> {
  let filename = 'code.orcvs'
  if (source.includes('.orcvs.')) {
    filename = source;
    source = sourceFromFile(source);
  }
  const module = `
    export const code = (o) => {
      ${source}
    }
  `
  const { code } = await importFromString(module, { useCurrentGlobal: true, transformOptions: { loader: 'ts' },  filename });
  return code;
}
