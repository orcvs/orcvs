# 09 — Add the website skill

**What to build:** `.agents/skills/website/SKILL.md`, symlinked from `.claude/skills/` as
`docs/agents/skills.md` describes, so an agent changing the site knows the layout, the commands,
the `<Console>` embed contract, the URL parameters, and which checks a site change owes. It tells
agents to check pages in the pre-installed Playwright Chromium and take screenshots, and to pin any
outside skill before using it. See `../spec.md` (Agent tooling). Use the writing-for-agents skill.

**Blocked by:** 03, 04

**Status:** ready-for-agent

- [ ] `CLAUDE.md`'s verification list names the site gates, as it does for the other areas.
- [ ] The skill's routing description says when to use it and when the `egui` skill applies
      instead.
