window.BENCHMARK_DATA = {
  "lastUpdate": 1788399700368,
  "repoUrl": "https://github.com/orcvs/orcvs",
  "entries": {
    "lang": [
      {
        "commit": {
          "author": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "078af88d26f25ff304d57a788c19725a43fb1b48",
          "message": "Merge pull request #9 from orcvs/criterion\n\nGate lang performance against its own history",
          "timestamp": "2026-09-01T14:06:05+10:00",
          "tree_id": "0ed4e4ac6f60a0843dec88434b34f29d4463acb9",
          "url": "https://github.com/orcvs/orcvs/commit/078af88d26f25ff304d57a788c19725a43fb1b48"
        },
        "date": 1788236375526,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 154,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 482,
            "range": "± 14",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "committer": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "distinct": true,
          "id": "c709e6da88938c0d661af230b7dd740ce930631d",
          "message": "Enforce raw play operand contracts",
          "timestamp": "2026-09-02T10:59:54+10:00",
          "tree_id": "5c1c333640e7b8a0a9e2afb7fd8344a65c545861",
          "url": "https://github.com/orcvs/orcvs/commit/c709e6da88938c0d661af230b7dd740ce930631d"
        },
        "date": 1788311562070,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 118,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 82,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 40,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 346,
            "range": "± 4",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "committer": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "distinct": true,
          "id": "f7d4c69592db8c11ea2fed3492aaa35d0f4e170a",
          "message": "Centralize typed operand extraction",
          "timestamp": "2026-09-02T11:43:44+10:00",
          "tree_id": "0225340eb16d4217a20e07d11a20888762b2a4b9",
          "url": "https://github.com/orcvs/orcvs/commit/f7d4c69592db8c11ea2fed3492aaa35d0f4e170a"
        },
        "date": 1788313574819,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 121,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 81,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 359,
            "range": "± 5",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "committer": {
            "email": "tobyhede@gmail.com",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "distinct": true,
          "id": "e1e1815bccd5ba9e5d8b73da99c8766169db7c92",
          "message": "Correct issue metadata and exclude prototypes from review\n\nA CodeRabbit pass on this branch reported findings, none of them in the\nbranch's own diff. The two code findings live in the Language Map and are\nrecorded as language-map/05 rather than folded into unrelated work.\n\nTwo issue files carried stale metadata. The native MIDI issue said \"prefactor\"\nwhere it meant refactor. The MIDI output family issue still read\nready-for-agent with every checklist item ticked and an Implemented\ndeclaration in its comments; it is resolved.\n\ndocs/tooling.md opened its benchmark paragraph with \"A third tier measures\nrather than checks\", which the rest of the same paragraph contradicts: it\ngoes on to say the workflow fails past a threshold, bench.yml sets\nfail-on-alert, and the next paragraph calls it a gate. The tier does check;\nit just checks in CI rather than under mise run check. The paragraph is\nreflowed to the file's width, so the diff is wider than the one sentence\nthat changed.\n\nA further finding was against a syntax prototype, which review should not\nhave been reading. A prototype explores one design question and may hold\ncompeting spellings on purpose while that question is open; \"correcting\" one\nis not a fix but a decision, and not review's to make. Prototypes are now\nexcluded through reviews.path_filters, and docs/agents/syntax-prototypes.md\nrecords the rule and the narrow case for lifting it.\n\nThe filter was verified rather than assumed. Identical bait — two misspellings\nin one sentence of visible prose — was placed in a prototype and in a control\nfile under .scratch/, then reviewed in a single pass. The control was flagged\nand the prototype was not. The prototype still appears in reviewedFiles, so\npath_filters suppresses findings without keeping content off the wire; the\ndoc states that distinction.\n\nClaude-Session: https://claude.ai/code/session_01FLLHPBUWJ91eXJLHtPbCCH",
          "timestamp": "2026-09-03T10:13:40+10:00",
          "tree_id": "2b2cfa1269274c7c1b63575494106617ae407aee",
          "url": "https://github.com/orcvs/orcvs/commit/e1e1815bccd5ba9e5d8b73da99c8766169db7c92"
        },
        "date": 1788399699660,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 81,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 44,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 265,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2130,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 8368,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 32837,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 29676,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 228834,
            "range": "± 1459",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 2325490,
            "range": "± 15990",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 30899,
            "range": "± 518",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 234056,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 2328257,
            "range": "± 6160",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}