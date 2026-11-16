import { execFileSync } from 'node:child_process';
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
 * marker, an amendment heading the date comparison cannot read, or an Evidence
 * line naming a file that does not hold its symbol would all land unnoticed.
 *
 * `--since <ref>` adds the one rule a single tree cannot show: a change to
 * `docs/adr/` since `<ref>` moves every topic's marker, unless that marker
 * already names the newest ADR and date. An amendment dated a marker's own day
 * is otherwise invisible to the date comparison.
 */

const RULE_PREFIXES = ['LANG', 'EVAL', 'PLAY', 'CONS', 'SITE'];
const MARKER = /^> Snapshot: incorporates ADRs through (\d{4}) and ADR changes dated through (\d{4}-\d\d-\d\d)\.$/m;
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
function adrsUnder(root: string): { name: string; number: number; latest: string; undated: string[] }[] {
  const adrDir = join(root, 'docs/adr');
  return readdirSync(adrDir)
    .filter((name) => /^\d{4}-.*\.md$/.test(name))
    .flatMap((name) => {
      const text = readFileSync(join(adrDir, name), 'utf8');
      // A heading naming an amendment in any other form carries no date the
      // comparison can read, so the change it records could never be newer.
      const undated = [...text.matchAll(/^#{1,6} Amendment\b.*$/gm)]
        .map((m) => m[0])
        .filter((heading) => !/^## Amendment, \d{4}-\d\d-\d\d\b/.test(heading));
      const status = /^Status:[\s\S]*?(?:\n\n|(?![\s\S]))/m.exec(text)?.[0] ?? '';
      if (/^Status: (?:proposed|rejected)\b/.test(status)) return [];
      const amendments = [...text.matchAll(/^## Amendment, (\d{4}-\d\d-\d\d)/gm)].map((m) => m[1]);
      const dates = [...(status.match(DATE) ?? []), ...amendments].sort();
      // '' stands for "no dated change": it sorts before every ISO date, so an
      // ADR without one is never newer than a marker's date.
      return [{ name, number: Number(name.slice(0, 4)), latest: dates.at(-1) ?? '', undated }];
    });
}

/** The marker a topic should carry: the highest ADR number and the latest ADR change date. */
function newestMarker(root: string): { through: number; dated: string } {
  const adrs = adrsUnder(root);
  return {
    through: Math.max(0, ...adrs.map((adr) => adr.number)),
    dated: adrs.map((adr) => adr.latest).sort().at(-1) ?? '',
  };
}

/**
 * Every topic whose marker a change left in place although the change touched
 * `docs/adr/`. `changed` lists the paths the change touches, `before` maps each
 * topic to its marker line before the change (absent for a new topic), and
 * `root` holds the tree after it.
 */
export function unmovedMarkers(root: string, changed: string[], before: Map<string, string>): string[] {
  // A proposed or rejected ADR decides nothing, so changing one owes no marker.
  const decided = new Set(adrsUnder(root).map((adr) => `docs/adr/${adr.name}`));
  const decides = (path: string) => decided.has(path) || !existsSync(join(root, path));
  if (!changed.some((path) => /^docs\/adr\/\d{4}-.*\.md$/.test(path) && decides(path))) return [];
  const dir = join(root, 'docs/requirements');
  const newest = newestMarker(root);
  const problems: string[] = [];
  for (const name of readdirSync(dir).filter((n) => n.endsWith('.md') && n !== 'README.md').sort()) {
    const snapshot = MARKER.exec(readFileSync(join(dir, name), 'utf8'));
    if (!snapshot || snapshot[0] !== before.get(name)) continue;
    if (Number(snapshot[1]) === newest.through && snapshot[2] === newest.dated) continue;
    problems.push(`${name}: the change touches docs/adr/ but leaves this snapshot marker where it was`);
  }
  return problems;
}

/** `unmovedMarkers` for the change from git revision `base` to the working tree under `root`. */
function unmovedSince(root: string, base: string): string[] {
  const git = (...args: string[]) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' });
  const changed = git('diff', '--name-only', '--no-renames', base, '--').split('\n').filter(Boolean);
  const before = new Map<string, string>();
  for (const path of git('ls-tree', '--name-only', base, 'docs/requirements/').split('\n')) {
    if (!path.endsWith('.md')) continue;
    const snapshot = MARKER.exec(git('show', `${base}:${path}`));
    if (snapshot) before.set(path.slice('docs/requirements/'.length), snapshot[0]);
  }
  return unmovedMarkers(root, changed, before);
}

/** Every problem in the requirements under `root`, one line each. */
export function check(root: string): string[] {
  const dir = join(root, 'docs/requirements');
  const adrs = adrsUnder(root);
  const names = readdirSync(dir).filter((name) => name.endsWith('.md')).sort();
  const texts = new Map(names.map((name) => [name, readFileSync(join(dir, name), 'utf8')]));
  const problems: string[] = [];
  const defined = new Map<string, string>();
  for (const adr of adrs) {
    for (const heading of adr.undated) {
      problems.push(`docs/adr/${adr.name}: "${heading}" is not an \`## Amendment, YYYY-MM-DD\` heading`);
    }
  }

  for (const [name, text] of texts) {
    if (name === 'README.md') continue;
    const snapshot = MARKER.exec(text);
    if (!snapshot) {
      problems.push(`${name}: no snapshot marker`);
    } else {
      const [through, dated] = [Number(snapshot[1]), snapshot[2]];
      if (!adrs.some((adr) => adr.number === through)) {
        problems.push(`${name}: the snapshot marker incorporates ADRs through ${snapshot[1]}, which no ADR file holds`);
      }
      for (const adr of adrs) {
        if (adr.number > through || adr.latest > dated) {
          problems.push(`${name}: ADR ${adr.name.slice(0, 4)} is newer than the snapshot marker, which does not incorporate it`);
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
    const sourceAdrs = /^> Source ADRs:([\s\S]*?)\n\n/m.exec(text);
    const listed = new Set(sourceAdrs ? sourceAdrs[1].match(/\b\d{4}\b/g) ?? [] : []);
    const body = sourceAdrs ? text.slice(sourceAdrs.index + sourceAdrs[0].length) : text;
    // An open decision may link a proposed ADR, which no rule restates yet.
    const decided = decisions >= 0 ? text.slice(0, decisions) : text;
    const rules = sourceAdrs ? decided.slice(sourceAdrs.index + sourceAdrs[0].length) : decided;
    for (const adr of new Set([...rules.matchAll(/\.\.\/adr\/(\d{4})-/g)].map((m) => m[1]))) {
      if (!listed.has(adr)) problems.push(`${name}: a rule cites ADR ${adr}, which its Source ADRs do not list`);
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
  const since = process.argv.indexOf('--since');
  const base = process.argv[since + 1];
  if (since >= 0 && !base) {
    console.error('usage: node scripts/check-requirements.ts [--since <git revision>]');
    process.exit(2);
  }
  const problems = since >= 0 ? unmovedSince(root, base) : check(root);
  for (const problem of problems) console.error(problem);
  if (problems.length > 0) process.exit(1);
}
