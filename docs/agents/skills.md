# Repository Skills

The repository's own skills live in `.agents/skills/`, one directory per skill with a `SKILL.md`
carrying the name and routing description. That directory is the source of truth and the only place
a skill should be edited.

`.claude/skills/` holds a symlink per skill pointing back at `.agents/skills/`:

```text
.claude/skills/rust-change -> ../../.agents/skills/rust-change
```

Claude Code discovers project skills under `.claude/skills/` and nowhere else, so a skill without a
symlink is invisible to it — `AGENTS.md` can name the skill and the file can be read directly, but
`/rust-change` is not offered and the model cannot route to it.

The symlinks are tracked. This matters more than it looks: `git worktree add` populates a new
worktree from the index, so an untracked symlink exists only in the checkout that created it and
every worktree silently loses the skill. That is the state this file was written to end. Committing
them costs five 120000-mode blobs holding a relative path each.

Adding a skill means adding both halves:

```sh
ln -s "../../.agents/skills/<name>" ".claude/skills/<name>"
git add .agents/skills/<name> .claude/skills/<name>
```

The targets are relative and resolve from the repository root, so they work in the main checkout and
in every worktree without rewriting.
