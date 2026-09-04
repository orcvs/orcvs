window.BENCHMARK_DATA = {
  "lastUpdate": 1788504841191,
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
          "id": "9f8d6a9f6006ca6f4cbbeb132656f97ea9a54fdd",
          "message": "Specify the Orcvs evaluation machine\n\nNames the machine that evaluates an Expression and states what it guarantees:\na right-to-left walk over a prefix Atom sequence against an Operand Stack that\nlives for one Expression and holds no memory of its own.\n\nOrcvs is not a virtual machine and the ADR says so. There is no bytecode,\nbecause the Atom sequence is re-derived from character Source on every Tick,\nand no control flow, because activation and Portals are Source-resident. ADR\n0003 already makes the Source Snapshot the complete language state; this keeps\nthe Evaluator consistent with it.\n\nThe instruction set becomes one declaration that everything else derives from.\nToday it is spread across four places that cannot check each other: operand\ntypes in the Function definitions, accepted types in ADR 0021, pervasive rules\nin ADR 0007, and operand order in each Function body. The ADR fixes that the\ndeclaration is single, not what form it takes, and leaves the value model open\nbetween ADR 0007 and ADR 0026.\n\nAsking what bounds the machine relies on found a reachable panic, recorded as\ninherited-defects issue 15. A 64-Cell Expression parses to 32 Atoms and\noverflows the 16-slot Operand Stack. The defect predates every current branch,\nso only the record lands here.\n\nClaude-Session: https://claude.ai/code/session_01YJxB65AK7Dbt2c4w3SsJtg",
          "timestamp": "2026-09-03T20:58:05+10:00",
          "tree_id": "dfc88297cbcd2f05693305f7cfa03ac5e5ca3f57",
          "url": "https://github.com/orcvs/orcvs/commit/9f8d6a9f6006ca6f4cbbeb132656f97ea9a54fdd"
        },
        "date": 1788439442559,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 125,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 503,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 53,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2761,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10662,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 41105,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 38954,
            "range": "± 697",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 277604,
            "range": "± 5632",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 2986693,
            "range": "± 68949",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 39911,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 282221,
            "range": "± 5746",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 2982620,
            "range": "± 30473",
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
          "id": "dc37ebe05b96c27d4f35d80677b1cfb145a0b92e",
          "message": "Re-measure the rebuild path against commits that still exist\n\nRewriting this effort's history to fold its review follow-ups moved every SHA\nabove `85f8dc1`, and issue 06 named two of them: the commit it called the tip,\nand one it cited when auditing performance claims. Neither is reachable any\nmore.\n\nRelabelling would have been enough to make the file read correctly and not\nenough to make it true, so the measurement was taken again. A number whose\ncommit cannot be checked out is not evidence.\n\nThe conclusion is unchanged and now rests on two independent sessions. Every\nclean figure reproduces the original within noise, and four times the Cells\nstill costs the old code 6.6x then 9.7x against the new code's 3.5x then 3.8x.\n\nRecorded rather than smoothed over: the first `pre` run began at load 7.29,\nsits 2% to 4% above its own second run, and returned one 12% interval that is\ndiscarded under the rule this ticket already set. And the original measurement\nturns out to have been taken from a working tree matching no commit at all —\nthe last review follow-up was committed while the benchmark was running. Its\nchanges were comments and test-only code, so the old numbers were\nrepresentative; the new table is the stronger claim.\n\nChanged: .scratch/source-module-depth/issues/06-measure-the-rebuild-path.md\nTests added or updated: none; this is a measurement\nCommands run: cargo bench --package orcvs --bench source --locked, twice per\n  commit at f8f7bb6 and at 31a421c — completed, numbers in the issue\nRisks: none; no code changed\n\nClaude-Session: https://claude.ai/code/session_01FY6ATVDG3G6xh4NKbHZgEp",
          "timestamp": "2026-09-03T22:48:57+10:00",
          "tree_id": "3c69a7a6f822047888762af98fbff9d33a686660",
          "url": "https://github.com/orcvs/orcvs/commit/dc37ebe05b96c27d4f35d80677b1cfb145a0b92e"
        },
        "date": 1788482782002,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 141,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 64,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 492,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 55,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2878,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11293,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44674,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14273,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 51697,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 210082,
            "range": "± 1973",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 15240,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 51320,
            "range": "± 547",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 213473,
            "range": "± 1388",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tobyhede@info-architects.net",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0fd20b8e32a2b1a7577698c5aaaab3f625523321",
          "message": "Merge pull request #10 from orcvs/01-thread-tick-and-position-into-interpretation\n\nThread Tick and Position into interpretation",
          "timestamp": "2026-09-04T13:24:02+10:00",
          "tree_id": "90c0e48bc4ef0bb8df723f00e1e61f9d7c614901",
          "url": "https://github.com/orcvs/orcvs/commit/0fd20b8e32a2b1a7577698c5aaaab3f625523321"
        },
        "date": 1788492627677,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 116,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 468,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2823,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11341,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44640,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14084,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 51469,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 204081,
            "range": "± 2049",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14880,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 50812,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 204618,
            "range": "± 1069",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tobyhede@info-architects.net",
            "name": "Toby Hede",
            "username": "tobyhede"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "95e39b5b9b80aad46e607d798d243df6e52deb9c",
          "message": "Merge pull request #12 from orcvs/evaluation-machine\n\nSpecify the Orcvs evaluation machine, and carry each MIDI operand domain in its type",
          "timestamp": "2026-09-04T16:47:50+10:00",
          "tree_id": "522fcc0f6098c0dbe3c642c2cd3ec7527878d9e0",
          "url": "https://github.com/orcvs/orcvs/commit/95e39b5b9b80aad46e607d798d243df6e52deb9c"
        },
        "date": 1788504840216,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 97,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 51,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 397,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2652,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10499,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 40627,
            "range": "± 1082",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 13508,
            "range": "± 1128",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 47893,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 190804,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 13979,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 47741,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 188831,
            "range": "± 2818",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}