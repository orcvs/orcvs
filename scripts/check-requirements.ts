import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, normalize, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

/**
 * Holds `docs/requirements/` to the snapshot contract its index states.
 *
 * Each topic document restates accepted ADRs through a recorded ADR number and
 * date, and accepting or amending an ADR updates it in the same change. Nothing
 * else reads these documents mechanically, so without this check a duplicated
 * rule ID, a rule cited by another document that no longer exists, a broken
 * link or anchor, an ADR a rule cites but the marker omits, an ADR newer than a
 * marker that the marker does not name as behind, or an Evidence line naming a
 * file that does not hold its symbol would all land unnoticed.
 */

const RULE_PREFIXES = ['LANG', 'EVAL', 'PLAY', 'CONS', 'SITE'];
const MARKER = /^> Snapshot: incorporates ADRs through (\d{4}) and ADR changes dated through (\d{4}-\d\d-\d\d)\.$/m;
/** An ADR number, not the year of a date. */
const ADR_NUMBER = /(?<![\d-])\d{4}(?![\d-])/g;
const DATE = /\d{4}-\d\d-\d\d/g;
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

/**
 * Each ADR in `docs/adr/` with its number and the latest date its Status
 * paragraph or an `## Amendment, <date>` heading carries. A marker's date is
 * compared by day, so a change dated the marker's own day counts as included.
 * A proposed or rejected ADR decides nothing, so it is left out; accepting a
 * proposal dates its Status paragraph, which makes it newer than the markers.
 */
function adrsUnder(root: string): { number: number; latest: string }[] {
  const adrDir = join(root, 'docs/adr');
  return readdirSync(adrDir)
    .filter((name) => /^\d{4}-.*\.md$/.test(name))
    .flatMap((name) => {
      const text = readFileSync(join(adrDir, name), 'utf8');
      const status = /^Status:[\s\S]*?(?:\n\n|(?![\s\S]))/m.exec(text)?.[0] ?? '';
      if (/^Status: (?:proposed|rejected)\b/.test(status)) return [];
      const amendments = [...text.matchAll(/^## Amendment, (\d{4}-\d\d-\d\d)/gm)].map((m) => m[1]);
      const dates = [...(status.match(DATE) ?? []), ...amendments].sort();
      // '' stands for "no dated change": it sorts before every ISO date, so an
      // ADR without one is never newer than a marker's date.
      return [{ number: Number(name.slice(0, 4)), latest: dates.at(-1) ?? '' }];
    });
}

/** Every problem in the requirements under `root`, one line each. */
export function check(root: string): string[] {
  const dir = join(root, 'docs/requirements');
  const adrs = adrsUnder(root);
  const highest = Math.max(0, ...adrs.map((adr) => adr.number));
  const names = readdirSync(dir).filter((name) => name.endsWith('.md')).sort();
  const texts = new Map(names.map((name) => [name, readFileSync(join(dir, name), 'utf8')]));
  const problems: string[] = [];
  const defined = new Map<string, string>();

  for (const [name, text] of texts) {
    if (name === 'README.md') continue;
    const marker = MARKER.exec(text);
    if (!marker) {
      problems.push(`${name}: no snapshot marker`);
    } else {
      const [through, dated] = [Number(marker[1]), marker[2]];
      const behind = new Set((/^> Behind:(.*)$/m.exec(text)?.[1] ?? '').match(ADR_NUMBER)?.map(Number));
      if (through > highest) problems.push(`${name}: the marker incorporates ADRs through ${marker[1]}, above the highest ADR`);
      for (const adr of adrs) {
        if ((adr.number > through || adr.latest > dated) && !behind.has(adr.number)) {
          const id = String(adr.number).padStart(4, '0');
          problems.push(`${name}: ADR ${id} is newer than the marker, which neither incorporates it nor names it in a Behind line`);
        }
      }
    }
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
    // An open decision may link a proposed ADR, which no rule restates yet.
    const decided = decisions >= 0 ? text.slice(0, decisions) : text;
    const rules = marker ? decided.slice(marker.index + marker[0].length) : decided;
    for (const adr of new Set([...rules.matchAll(/\.\.\/adr\/(\d{4})-/g)].map((m) => m[1]))) {
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
