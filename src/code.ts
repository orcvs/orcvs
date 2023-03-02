import { readFileSync } from 'fs';
import { importFromString } from 'module-from-string'
import { Logger } from './logger';


const logger = Logger.child({
  source: 'Code'
});

export function sourceFromFile(filename: string) {
  const code = readFileSync(filename, { encoding: 'utf8' });
  return code;
}

export async function codify(source: string): Promise<(() => {})> {
  let filename = 'code.orcvs.js'
  if (['.js', '.orcvs.'].some( s => source.includes(s))) {
    filename = source;  
    source = sourceFromFile(source);
  } else {
    logger.info('Loading source as string'); 
  }

  const module = `
    export const code = (o) => {
      ${source}
    }
  `
  const { code } = await importFromString(module, { useCurrentGlobal: true, transformOptions: { loader: 'ts' },  filename });
  return code;
}
