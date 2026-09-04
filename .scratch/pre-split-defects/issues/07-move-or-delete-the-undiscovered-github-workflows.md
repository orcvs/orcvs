# 07 — Move or delete the undiscovered GitHub workflows

**What to fix:** Three workflow files sit in `shell/.github/workflows/`. GitHub Actions reads
workflows only from the repository root. These three have never run, and each one names paths and a
binary that do not exist.

**Status:** resolved

- [x] Every workflow file in the repository either runs or is deleted.
- [x] No workflow names a binary or a directory that the build does not produce.
- [x] The tooling contract covers any workflow that is kept.

## Comments

`shell/.github/workflows/` holds `pages.yml`, `rust.yml`, and `typos.yml`. The repository root holds
only `.github/workflows/test.yml`, which is the workflow that actually runs.

On `main` the same three files sit in `console/.github/workflows/`, so the crate split only renamed
the directory. The files have never been discovered, on either branch.

Each file is also wrong in its own right, which is what a dead file becomes.

`pages.yml:39` runs `./trunk build --release --public-url $public_url` from the repository root, and
line 45 deploys `folder: dist`. The Trunk project is in `shell/`, and Trunk writes `shell/dist`. A
root build finds no `index.html`.

`rust.yml:151` copies
`target/${{ matrix.TARGET }}/release/eframe_template${{ matrix.EXTENSION }}`. The workspace has
never built a binary called `eframe_template`. The binary is `shell`. Lines 155, 156, 163, and 164
name the same file for the upload and the release asset.

`typos.yml` is not broken in the same way. It is only unreachable.

Triage is needed because the answer depends on what you want.

Delete all three if `test.yml` already covers the work. It runs the tiered checks, and typo checking
can join it.

Move them to `.github/workflows/` if you want GitHub Pages deployment and release binaries. Then
correct the paths and the binary name in the same change, because both are wrong, and add the
assertions to `scripts/check-tooling-contract.sh` that keep them correct.

Do not move them without correcting them. A workflow that runs and fails is worse than one that
never runs.

## Resolution

All three were deleted. `shell/.github/` no longer exists, and `.github/workflows/test.yml` is the
only workflow in the repository. Nothing now names `eframe_template` or builds Trunk from the
repository root.

GitHub Pages deployment and release binaries were not adopted. If either is wanted later, it is new
work against a correct path and binary name, not a move of these files.
