# Correct the defects inherited from before the crate split

**Goal:** Record and correct the defects that were already in `main` before the `orcvs`/`shell` crate split. None of these defects come from the split. A three-reviewer pass on the split branch found them, but each one is present in `main` at the same lines.

## Why

The crate split moved files. It did not change the code in them. The review that accompanied the split therefore reported many defects that the split neither caused nor touched. Those defects were correct findings, but they were out of scope for that branch. This effort holds them so that they are not lost.

Two groups need different treatment.

The first group changes what the user sees. The MIDI adapter goes silent after one failed send. A panic in a Tick kills the editor. A WASM start failure reports nothing. The service worker caches file names that the build never writes. Each of these is a defect a user can meet today.

The second group is inherited from the `eframe` template that the console started from. The MIDI client is named "Orca". The web manifest names an "egui Template PWA". Three GitHub workflows sit in a directory that GitHub Actions never reads. These do not break the running application, but each one states something false about the project.

## Scope

Every issue in this effort was verified against the code, not taken from a review summary. Where a review overstated a finding, the issue states the smaller, true version. Issue 05 is the clearest case: the `unsafe` write is sound, and only its comment is wrong.

## Not in scope

The `Orcvs::playback_engine` accessor. That accessor came from the crate split, not from before it, and it belongs to the `crate-boundaries` effort.
