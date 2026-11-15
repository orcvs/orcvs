import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, normalize, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

/**
 * Holds `docs/requirements/` to the snapshot contract its index states.
 *
 * Each topic document restates accepted ADRs as of a recorded commit, and
 * accepting or amending an ADR updates it in the same change. Nothing else
 * reads these documents mechanically, so without this check a duplicated rule
 * ID, a rule cited by another document that no longer exists, a broken link or
 * anchor, an ADR a rule cites but the marker omits, or an Evidence line naming
 * a file that does not hold its symbol would all land unnoticed.
 */

const RULE_PREFIXES = ['LANG', 'EVAL', 'PLAY', 'CONS', 'SITE'];
const MARKER = /^> Snapshot: checked against `[0-9a-f]{7,40}` \(\d{4}-\d\d-\d\d\); incorporates ADRs through \d{4}\.$/m;
const SOURCE_FILE = /^[\w./-]+\.(rs|ts|js|astro|html|toml|yml|md|sh)$/;

/** GitHub's heading anchor: lowercase, punctuation dropped, spaces to hyphens. */
export function slug(heading: string): string {
  return heading
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N} _-]/gu, '')
    .replace(/ /g, '-');
}

function anchorsOf(text: string): Set<string> {
  const anchors = new Set<string>();
  let inFence = false;
  for (const line of text.split('\n')) {
    if (line.startsWith('```')) inFence = !inFence;
    const heading = !inFence && /^#{1,6} (.+)$/.exec(line);
    if (heading) anchors.add(slug(heading[1]));
  }
  return anchors;
}

/** Every problem in the requirements under `root`, one line each. */
export function check(root: string): string[] {
  const dir = join(root, 'docs/requirements');
  const names = readdirSync(dir).filter((name) => name.endsWith('.md')).sort();
  const texts = new Map(names.map((name) => [name, readFileSync(join(dir, name), 'utf8')]));
  const problems: string[] = [];
  const defined = new Map<string, string>();

  for (const [name, text] of texts) {
    if (name === 'README.md') continue;
    if (!MARKER.test(text)) problems.push(`${name}: no snapshot marker`);
    for (const match of text.matchAll(/^### ([A-Z]+-\d{3})\b/gm)) {
      const id = match[1];
      if (defined.has(id)) problems.push(`${name}: ${id} is also defined in ${defined.get(id)}`);
      defined.set(id, name);
    }
  }

  const rule = new RegExp(`\\b((?:${RULE_PREFIXES.join('|')})-\\d{3})\\b`, 'g');
  for (const [name, text] of texts) {
    const decisions = text.indexOf('\n## Open decisions');
    for (const match of text.matchAll(rule)) {
      // An open decision may hold an ID in reserve for a rule not yet binding.
      if (defined.has(match[1]) || (decisions >= 0 && match.index > decisions)) continue;
      problems.push(`${name}: cites ${match[1]}, which no requirement defines`);
    }

    for (const match of text.matchAll(/\]\(([^)\s]+)\)/g)) {
      const [target, anchor] = match[1].split('#');
      if (/^[a-z]+:/.test(target)) continue;
      const path = target === '' ? join(dir, name) : normalize(join(dir, target));
      if (!existsSync(path)) {
        problems.push(`${name}: links to missing ${target}`);
      } else if (anchor && path.endsWith('.md') && !anchorsOf(readFileSync(path, 'utf8')).has(anchor)) {
        problems.push(`${name}: links to missing anchor ${target}#${anchor}`);
      }
    }

    if (name === 'README.md') continue;
    const marker = /^> Source ADRs:([\s\S]*?)\n\n/m.exec(text);
    const listed = new Set(marker ? marker[1].match(/\b\d{4}\b/g) ?? [] : []);
    const body = marker ? text.slice(marker.index + marker[0].length) : text;
    for (const adr of new Set([...body.matchAll(/\.\.\/adr\/(\d{4})-/g)].map((m) => m[1]))) {
      if (!listed.has(adr)) problems.push(`${name}: a rule cites ADR ${adr}, which the marker does not list`);
    }

    // An Evidence line names a file, then symbols or tests in it, in backticks.
    for (const line of body.split('\n')) {
      let file: string | null = null;
      for (const token of line.matchAll(/`([^`]+)`/g)) {
        const value = token[1];
        if (SOURCE_FILE.test(value)) {
          file = existsSync(join(root, value)) ? readFileSync(join(root, value), 'utf8') : null;
          if (file === null && line.startsWith('Evidence:')) problems.push(`${name}: Evidence names missing ${value}`);
        } else if (file !== null && /^[A-Za-z_][\w:]*$/.test(value)) {
          const symbol = value.split('::').pop() as string;
          if (!file.includes(symbol)) problems.push(`${name}: \`${value}\` is not in the file named before it`);
        }
      }
    }
  }
  return problems;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const root = resolve(dirname(new URL(import.meta.url).pathname), '..');
  const problems = check(root);
  for (const problem of problems) console.error(problem);
  if (problems.length > 0) process.exit(1);
}
