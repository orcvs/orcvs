window.BENCHMARK_DATA = {
  "lastUpdate": 1789960270495,
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
      },
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
          "id": "d90059d7769098fecfd75f8debc905c096b52ee6",
          "message": "Merge pull request #13 from orcvs/15-bound-the-operand-stack\n\nBound the Operand Stack by the Expression length",
          "timestamp": "2026-09-04T20:23:24+10:00",
          "tree_id": "8a01637bc931607eac5335d0ae40336ea99df409",
          "url": "https://github.com/orcvs/orcvs/commit/d90059d7769098fecfd75f8debc905c096b52ee6"
        },
        "date": 1788517803924,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 324,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 42,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 84,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2131,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 8384,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 33205,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 10769,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 38543,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 151684,
            "range": "± 1185",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 11005,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 38965,
            "range": "± 1506",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 155062,
            "range": "± 1454",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5f9e3fec23e131646dc2e1fb5cf53449ddd4f860",
          "message": "Merge pull request #15 from orcvs/02-schedule-timed-play-note-off\n\nSchedule each Timed Play's Note Off at the Tick its length names",
          "timestamp": "2026-09-04T23:20:36+10:00",
          "tree_id": "83dd61c8f68b181b33b57e1aad8cbd1adfdf87d0",
          "url": "https://github.com/orcvs/orcvs/commit/5f9e3fec23e131646dc2e1fb5cf53449ddd4f860"
        },
        "date": 1788528428413,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 126,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 69,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 485,
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 111,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2906,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11328,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44980,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14385,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 51601,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 207518,
            "range": "± 1284",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14645,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 50993,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 206104,
            "range": "± 1483",
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
          "id": "469971dfbede4970590416e9d5360732f0f3f48c",
          "message": "Merge pull request #14 from orcvs/01-square-centred-source-grid\n\nPresent the Source Grid as a square, centred viewport",
          "timestamp": "2026-09-04T23:40:09+10:00",
          "tree_id": "cfee9170fd225a0db9ed99686520410c62eb5760",
          "url": "https://github.com/orcvs/orcvs/commit/469971dfbede4970590416e9d5360732f0f3f48c"
        },
        "date": 1788529583506,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 129,
            "range": "± 12",
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
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 515,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 53,
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
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2757,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10655,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 40975,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14094,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 51618,
            "range": "± 1247",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 210230,
            "range": "± 1669",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14360,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 52149,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 210388,
            "range": "± 1826",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f75d5584167d5d519e3d517eab9fcd14b0cd6e4b",
          "message": "Merge pull request #17 from orcvs/04-plan-complete-sequence-writes-through-portals\n\nPlan complete Sequence writes through Portals",
          "timestamp": "2026-09-05T12:35:39+10:00",
          "tree_id": "fde96922f02e16aedd748d1ea3b040f949c00938",
          "url": "https://github.com/orcvs/orcvs/commit/f75d5584167d5d519e3d517eab9fcd14b0cd6e4b"
        },
        "date": 1788576111848,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 99,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 49,
            "range": "± 0",
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
            "value": 405,
            "range": "± 4",
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
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 94,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2610,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10259,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 40274,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 13958,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 47817,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 189321,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14510,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 47786,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 190077,
            "range": "± 650",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8cb169ebf2f557a290ee13974d520f47e8b6ba50",
          "message": "Merge pull request #16 from orcvs/02-broadcast-atomic-functions-over-sequences\n\nExtend the Atomic Functions pervasively over Sequences",
          "timestamp": "2026-09-05T13:02:08+10:00",
          "tree_id": "da33f42fe0d54d282af93dd4203204ce0eec2ea9",
          "url": "https://github.com/orcvs/orcvs/commit/8cb169ebf2f557a290ee13974d520f47e8b6ba50"
        },
        "date": 1788577695595,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 117,
            "range": "± 2",
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
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 477,
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
            "value": 2886,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11515,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45422,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14787,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 53437,
            "range": "± 1436",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 211245,
            "range": "± 4795",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14743,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 52357,
            "range": "± 1375",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 209890,
            "range": "± 5085",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2f3ed99c2ad6e84aa2936571fbc09c24fb1dceba",
          "message": "Merge pull request #19 from orcvs/07-decide-whether-a-grid-still-answers-what-fits\n\nStop asking a Grid what fits in a row",
          "timestamp": "2026-09-05T16:13:42+10:00",
          "tree_id": "2323614d67286f1a77247c4762a8c852fd395c5c",
          "url": "https://github.com/orcvs/orcvs/commit/2f3ed99c2ad6e84aa2936571fbc09c24fb1dceba"
        },
        "date": 1788589192373,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 116,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 74,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 482,
            "range": "± 8",
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
            "value": 2886,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11422,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45488,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14429,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 51752,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 210255,
            "range": "± 1308",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14638,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 51379,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 208397,
            "range": "± 1654",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "13ed0444c62275fd36842bd01d578a3730463734",
          "message": "Merge pull request #21 from orcvs/01-close-verification-gaps\n\nMake every check the repository owns actually run",
          "timestamp": "2026-09-05T21:15:36+10:00",
          "tree_id": "ffb9d739477cf3721551d59cb539f1b563bb5e6b",
          "url": "https://github.com/orcvs/orcvs/commit/13ed0444c62275fd36842bd01d578a3730463734"
        },
        "date": 1788607953026,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 129,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 72,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 87,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 519,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 48,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 97,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2509,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9525,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 36961,
            "range": "± 1130",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14388,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 49999,
            "range": "± 1684",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 209346,
            "range": "± 6699",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14222,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 52490,
            "range": "± 1448",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 212404,
            "range": "± 5266",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f58b4a9be868399e72508aefaeb37fe590ffb3b8",
          "message": "Merge pull request #18 from orcvs/05-extend-terminal-output-functions-over-sequences\n\nExtend the Terminal Output Functions over Sequences",
          "timestamp": "2026-09-06T16:56:28+10:00",
          "tree_id": "e3fe0717fafe18909eb2d65a41426e02c6c1580c",
          "url": "https://github.com/orcvs/orcvs/commit/f58b4a9be868399e72508aefaeb37fe590ffb3b8"
        },
        "date": 1788678146784,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 57,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 274,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 1605,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 6430,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 26446,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 8612,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 30850,
            "range": "± 1532",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 128963,
            "range": "± 3562",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 8645,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 31552,
            "range": "± 3058",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 130180,
            "range": "± 2691",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f7c7a74beccbe385a4edb366a60aa202cacf38e6",
          "message": "Merge pull request #26 from orcvs/03-own-monophonic-voices-per-channel\n\nOwn Monophonic voices per channel",
          "timestamp": "2026-09-06T17:42:01+10:00",
          "tree_id": "b505b3a8eee83a9fd7f5829f897e5ec32957d075",
          "url": "https://github.com/orcvs/orcvs/commit/f7c7a74beccbe385a4edb366a60aa202cacf38e6"
        },
        "date": 1788680907873,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 119,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 64,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 91,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 479,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 110,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2859,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11310,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44770,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14229,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 50832,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 206658,
            "range": "± 2541",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14408,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 50257,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 204775,
            "range": "± 1359",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "300c244877504e80487ceebb5fe3f02c7cd27c11",
          "message": "Merge pull request #25 from orcvs/04-send-control-change-and-pitch-bend\n\nSend Control Change and Pitch Bend",
          "timestamp": "2026-09-06T22:33:14+10:00",
          "tree_id": "a36ad611122eb332522e4691e05f761620d14a6a",
          "url": "https://github.com/orcvs/orcvs/commit/300c244877504e80487ceebb5fe3f02c7cd27c11"
        },
        "date": 1788698378533,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 120,
            "range": "± 1",
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
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 494,
            "range": "± 6",
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
            "value": 2826,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11421,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45188,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 14520,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 53319,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 212467,
            "range": "± 4530",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 14870,
            "range": "± 420",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 53233,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 210152,
            "range": "± 2977",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d000741baa5159f249ee9dec8bd676ff2bd9365",
          "message": "Merge pull request #29 from orcvs/03-parser-totality-on-ascii-input\n\nParser totality on ASCII input",
          "timestamp": "2026-09-06T22:59:16+10:00",
          "tree_id": "96ff5410b9b67d440ea67310d20371f2a3b05928",
          "url": "https://github.com/orcvs/orcvs/commit/2d000741baa5159f249ee9dec8bd676ff2bd9365"
        },
        "date": 1788699934224,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 75,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 43,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 60,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 320,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 34,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 69,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 1860,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 7927,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 29388,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 9263,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 32418,
            "range": "± 1460",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 133243,
            "range": "± 6454",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 9587,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 32817,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 134435,
            "range": "± 3368",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "747ed1d4a101b75f58b34f42bf972378751a0e4f",
          "message": "Merge pull request #27 from orcvs/06-optimize-tick-scheduling\n\nExecute a musical Tick in dependency order",
          "timestamp": "2026-09-09T06:25:15+10:00",
          "tree_id": "501a7436733a741bcb789d60ef32fb27edb30557",
          "url": "https://github.com/orcvs/orcvs/commit/747ed1d4a101b75f58b34f42bf972378751a0e4f"
        },
        "date": 1788899686766,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 134,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 70,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 411,
            "range": "± 3",
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2809,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11475,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44981,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5490,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 18679,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 73869,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5987,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 19658,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 73640,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 10683,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 38216,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 160064,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 676408,
            "range": "± 4407",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 9712,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 56913,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 256289,
            "range": "± 2366",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1111239,
            "range": "± 40108",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f131e5eac8d06622d509af7bab1d3c5e19c41cdc",
          "message": "Merge pull request #30 from orcvs/07-cell-indexed-parse\n\nParse Source into positioned expressions and execute live typed ticks",
          "timestamp": "2026-09-09T10:42:15+10:00",
          "tree_id": "1e2ac7ecb4ca524e280a4b5785591b7d2f907800",
          "url": "https://github.com/orcvs/orcvs/commit/f131e5eac8d06622d509af7bab1d3c5e19c41cdc"
        },
        "date": 1788915201586,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 185,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 107,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 820,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 260,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 537,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 922,
            "range": "± 6",
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2826,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11315,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44594,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5988,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 21939,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 90125,
            "range": "± 1112",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6142,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22885,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 87201,
            "range": "± 734",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19772,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76197,
            "range": "± 1141",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 315435,
            "range": "± 2061",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1293984,
            "range": "± 20575",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12151,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 74081,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 330520,
            "range": "± 10724",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1362167,
            "range": "± 22646",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "72bb2cc192a870c13b6263341c266dd45d07d97d",
          "message": "Merge pull request #24 from orcvs/dependabot/rust_toolchain/rust-toolchain-1.98.1\n\nBump rust-toolchain from 1.98.0 to 1.98.1",
          "timestamp": "2026-09-09T11:09:31+10:00",
          "tree_id": "3e91d333d141bbdb851804a57e801032e93ac25a",
          "url": "https://github.com/orcvs/orcvs/commit/72bb2cc192a870c13b6263341c266dd45d07d97d"
        },
        "date": 1788916863076,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 185,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 818,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 71,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 111,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 253,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 518,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 909,
            "range": "± 3",
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
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2807,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11241,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44564,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5977,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22119,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 90323,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6336,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22880,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 91827,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19640,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77125,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 322051,
            "range": "± 5230",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1315162,
            "range": "± 17670",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12318,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75917,
            "range": "± 1407",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 333712,
            "range": "± 1714",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1380367,
            "range": "± 20098",
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
          "id": "f25bd092360d8b8db747ef388ea915189c9ddd84",
          "message": "Merge pull request #31 from orcvs/08-memory-verification\n\nVerify how much memory Orcvs uses",
          "timestamp": "2026-09-09T13:10:09+10:00",
          "tree_id": "c6b6956278a0c664c201ef9aba72edfd647f78d7",
          "url": "https://github.com/orcvs/orcvs/commit/f25bd092360d8b8db747ef388ea915189c9ddd84"
        },
        "date": 1788924721024,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 187,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 77,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 109,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 824,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 274,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 521,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 902,
            "range": "± 6",
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2827,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11553,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44955,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6425,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22850,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 92071,
            "range": "± 2351",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6749,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23504,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 94689,
            "range": "± 2257",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20643,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78269,
            "range": "± 1434",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 323406,
            "range": "± 1647",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1327866,
            "range": "± 18000",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12612,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75858,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 332288,
            "range": "± 3446",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1370554,
            "range": "± 15434",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "da0f87b4637646f9d285e07b4510cdff7acc0d57",
          "message": "Merge pull request #36 from orcvs/09-widen-function-kind\n\nWiden the Function kind to value or effect",
          "timestamp": "2026-09-09T13:43:17+10:00",
          "tree_id": "2c685c512e091af9f4dca0a8c3a6d924b3438e0e",
          "url": "https://github.com/orcvs/orcvs/commit/da0f87b4637646f9d285e07b4510cdff7acc0d57"
        },
        "date": 1788926035735,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 123,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 50,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 63,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 512,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 44,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 75,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 168,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 335,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 588,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 68,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 1887,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 7437,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 29451,
            "range": "± 1474",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 4260,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 15162,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 57884,
            "range": "± 2083",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 4348,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 15350,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 57311,
            "range": "± 2948",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 12506,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 49782,
            "range": "± 2014",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 202595,
            "range": "± 12575",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 926826,
            "range": "± 57288",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 7770,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 47920,
            "range": "± 1500",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 217913,
            "range": "± 6350",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 995696,
            "range": "± 31739",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dc38019bb6151cbace510774e1a75a281009a9d6",
          "message": "Merge pull request #32 from orcvs/12-safety-action-reset\n\nClear controllers and the bend in the safety action",
          "timestamp": "2026-09-09T14:12:50+10:00",
          "tree_id": "a79299f57aa06cd891c7c9ee66b45f8cd111bae2",
          "url": "https://github.com/orcvs/orcvs/commit/dc38019bb6151cbace510774e1a75a281009a9d6"
        },
        "date": 1788927826044,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 139,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 59,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 66,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 610,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 51,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 191,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 363,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 641,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 37,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2124,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 8609,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 33930,
            "range": "± 863",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5556,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 19509,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 71868,
            "range": "± 1997",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5477,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 18782,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 71517,
            "range": "± 1064",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 15035,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 58899,
            "range": "± 1057",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 242947,
            "range": "± 1348",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1066974,
            "range": "± 10290",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 9415,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 57306,
            "range": "± 2926",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 262448,
            "range": "± 3493",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1170131,
            "range": "± 22335",
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
          "id": "74d2aa3cf31f14ca30c46dd9234cfc0c0f42fca5",
          "message": "Merge pull request #37 from orcvs/10-native-midi-seam\n\nName the native MIDI decision once",
          "timestamp": "2026-09-09T14:45:29+10:00",
          "tree_id": "c57b150fe12d4ef1329c560fda822fe1dfc3bea6",
          "url": "https://github.com/orcvs/orcvs/commit/74d2aa3cf31f14ca30c46dd9234cfc0c0f42fca5"
        },
        "date": 1788929840572,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 184,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 827,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 272,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 518,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 906,
            "range": "± 6",
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2812,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11392,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44681,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5953,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 21598,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 90906,
            "range": "± 1699",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6093,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22172,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 92561,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19834,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77192,
            "range": "± 1139",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 323705,
            "range": "± 1966",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1309040,
            "range": "± 22135",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12514,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75891,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 334357,
            "range": "± 3428",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1379397,
            "range": "± 17863",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2404c2afe348f0ec26d2572df1da2269a5555acd",
          "message": "Merge pull request #35 from orcvs/deepen-portal-and-evaluator-seams\n\nDeepen Portal relationships and test through the Evaluator",
          "timestamp": "2026-09-09T15:25:34+10:00",
          "tree_id": "e1ac1c2c0894d7da324e4ecd3bfdb302da754693",
          "url": "https://github.com/orcvs/orcvs/commit/2404c2afe348f0ec26d2572df1da2269a5555acd"
        },
        "date": 1788932203865,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 186,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 818,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 269,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 521,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 909,
            "range": "± 16",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2884,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11668,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45578,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5996,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 21850,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 89606,
            "range": "± 5234",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6241,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22397,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 91299,
            "range": "± 2043",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20156,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77681,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 321388,
            "range": "± 4237",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1312454,
            "range": "± 35016",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12669,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75846,
            "range": "± 532",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 332539,
            "range": "± 2256",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1392788,
            "range": "± 19168",
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
          "id": "51b0fc5fb6346ec1505333adaf078ef036f040e7",
          "message": "Merge pull request #43 from orcvs/verify-02-add-source-bang-activation-and-expiry\n\nTest the non-root Bang contact only prose claimed",
          "timestamp": "2026-09-09T16:43:29+10:00",
          "tree_id": "5b064cb1022a1b05a859be45fe1b38b360e5d3de",
          "url": "https://github.com/orcvs/orcvs/commit/51b0fc5fb6346ec1505333adaf078ef036f040e7"
        },
        "date": 1788936878548,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 156,
            "range": "± 1",
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
            "value": 74,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 684,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 55,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 91,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 193,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 417,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 739,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2457,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9660,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 38310,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5556,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 20620,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 78227,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6130,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 21126,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 76526,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 17670,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 68395,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 278434,
            "range": "± 5314",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1205852,
            "range": "± 6910",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10826,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 67029,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 303955,
            "range": "± 1321",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1327494,
            "range": "± 17215",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b332ac7e906b709e129078f8de8c357c2066079c",
          "message": "Merge pull request #40 from orcvs/02-grid-position-round-trip\n\nEncode the Grid laws the glossary already states",
          "timestamp": "2026-09-09T16:59:25+10:00",
          "tree_id": "f80f23c09cdbf42f8310ca0ff31a3cf0fd4978aa",
          "url": "https://github.com/orcvs/orcvs/commit/b332ac7e906b709e129078f8de8c357c2066079c"
        },
        "date": 1788937843015,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 171,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 62,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 680,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 242,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 495,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 870,
            "range": "± 4",
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
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2649,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10414,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 40445,
            "range": "± 928",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5690,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 19612,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 81465,
            "range": "± 648",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5806,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 20424,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 81264,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 18553,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 71244,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 297308,
            "range": "± 1485",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1300536,
            "range": "± 13003",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 11516,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 68937,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 317895,
            "range": "± 1652",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1383756,
            "range": "± 9079",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "468497d8fd57d84db978f378f2ea7eb43665a7ad",
          "message": "Merge pull request #41 from orcvs/bind-tick-lookup-ownership\n\nBind Tick Lookup to the computations it indexes",
          "timestamp": "2026-09-09T17:39:27+10:00",
          "tree_id": "183acd4c0e999290d882949507f6f8fa66ee3843",
          "url": "https://github.com/orcvs/orcvs/commit/468497d8fd57d84db978f378f2ea7eb43665a7ad"
        },
        "date": 1788940216725,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 191,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 822,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 259,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 543,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 930,
            "range": "± 14",
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
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 123,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2900,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11647,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45630,
            "range": "± 1151",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6009,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22114,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 90347,
            "range": "± 1190",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6117,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22757,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 88021,
            "range": "± 873",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20057,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77413,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 322570,
            "range": "± 2102",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1328975,
            "range": "± 27391",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12344,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76046,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 336480,
            "range": "± 2450",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1403971,
            "range": "± 14034",
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
          "id": "53230041992a7aa5156356e949b335dae2b8b1bd",
          "message": "Merge pull request #38 from orcvs/11-persist-source\n\nRestore the Source revision through eframe storage",
          "timestamp": "2026-09-09T18:23:21+10:00",
          "tree_id": "9abd15d8d737a1434d39f75a6b0465df61d82ed2",
          "url": "https://github.com/orcvs/orcvs/commit/53230041992a7aa5156356e949b335dae2b8b1bd"
        },
        "date": 1788943577833,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 209,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 104,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 852,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 113,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 270,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 519,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 991,
            "range": "± 29",
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
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 113,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2741,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10657,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 41020,
            "range": "± 1062",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6053,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22466,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 88191,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6721,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22701,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 86687,
            "range": "± 1090",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19998,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77266,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 320704,
            "range": "± 2075",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1342656,
            "range": "± 19646",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12537,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76540,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 340867,
            "range": "± 14598",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1441874,
            "range": "± 16979",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1c1bcb6cb37484b2b0ff5d6fff182f4ce5842e63",
          "message": "Merge pull request #44 from orcvs/deepen-tick-execution\n\nOwn Tick-local state transitions behind execution seam",
          "timestamp": "2026-09-09T19:05:36+10:00",
          "tree_id": "f932e73578204104624e75b0d73b4953fc5e610f",
          "url": "https://github.com/orcvs/orcvs/commit/1c1bcb6cb37484b2b0ff5d6fff182f4ce5842e63"
        },
        "date": 1788945394246,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 184,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 77,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 824,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 273,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 524,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 911,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 37,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 106,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2810,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11457,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44786,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6058,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22309,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 93236,
            "range": "± 1258",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6432,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23022,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 91105,
            "range": "± 1338",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19556,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77885,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 318030,
            "range": "± 1966",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1295676,
            "range": "± 21849",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12163,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 74848,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 334403,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1388332,
            "range": "± 13057",
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
          "id": "80d39c3951483eafd4da2616c05bca81221ba0ab",
          "message": "Merge pull request #42 from orcvs/02-put-midir-behind-a-native-midi-feature\n\nPut midir behind a native-midi feature",
          "timestamp": "2026-09-09T20:14:58+10:00",
          "tree_id": "a7928df46f9bb12b9227314cfb4c317248f45341",
          "url": "https://github.com/orcvs/orcvs/commit/80d39c3951483eafd4da2616c05bca81221ba0ab"
        },
        "date": 1788950211888,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 190,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 82,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 103,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 823,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 279,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 539,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 936,
            "range": "± 23",
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
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2842,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11524,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45568,
            "range": "± 3528",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6051,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 21995,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 90193,
            "range": "± 1087",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6367,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22663,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 93156,
            "range": "± 1237",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20002,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77178,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 325899,
            "range": "± 3458",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1314169,
            "range": "± 14246",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12406,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76432,
            "range": "± 1025",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 340969,
            "range": "± 2945",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1408106,
            "range": "± 20639",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2c564cab550e0dffb9848d9fda0139ffecf7b9b6",
          "message": "Merge pull request #45 from orcvs/08-bring-the-benchmark-workflow-inside-its-budget\n\nBring the benchmark run inside the budget its thresholds can use",
          "timestamp": "2026-09-09T21:13:24+10:00",
          "tree_id": "01d5bd1dfc8dd32f12107ae215087310e09a7746",
          "url": "https://github.com/orcvs/orcvs/commit/2c564cab550e0dffb9848d9fda0139ffecf7b9b6"
        },
        "date": 1788953220830,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 185,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 100,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 801,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 72,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 266,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 502,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 936,
            "range": "± 2",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 121,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2780,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11353,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44791,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5939,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22311,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 90059,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6299,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 22355,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 91060,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19587,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 75842,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 317182,
            "range": "± 1814",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1292106,
            "range": "± 3118",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12243,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75507,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 332080,
            "range": "± 1228",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1392897,
            "range": "± 4468",
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
          "id": "5de3f349e6b8cba3ec0a26a2d70f162fc5680cf8",
          "message": "Merge pull request #48 from orcvs/05-exhaustive-arithmetic-and-note-conversion\n\nProve the byte-arithmetic and Note-conversion laws exhaustively",
          "timestamp": "2026-09-09T23:12:33+10:00",
          "tree_id": "a078d90db023dcd5b8538633b5ee1ce2aae114bf",
          "url": "https://github.com/orcvs/orcvs/commit/5de3f349e6b8cba3ec0a26a2d70f162fc5680cf8"
        },
        "date": 1788959704106,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 208,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 857,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 114,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 272,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 513,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 957,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
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
            "value": 2767,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 10677,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 40988,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6213,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 22591,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 88868,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6288,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23168,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 87374,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19162,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76043,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 317910,
            "range": "± 1010",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1328248,
            "range": "± 5833",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12424,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75147,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 334720,
            "range": "± 2061",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1457743,
            "range": "± 3789",
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
          "id": "ec2258ae1cde158af10f76325f5dd60e72b6e0d4",
          "message": "Merge pull request #47 from orcvs/04-prove-the-expression-span-law\n\nProve the Expression Span law as a property",
          "timestamp": "2026-09-09T23:17:03+10:00",
          "tree_id": "f3313cfabb5619ff73af76a9424fd9f5583bcec1",
          "url": "https://github.com/orcvs/orcvs/commit/ec2258ae1cde158af10f76325f5dd60e72b6e0d4"
        },
        "date": 1788959974805,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 196,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 104,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 827,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 260,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 494,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 943,
            "range": "± 11",
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
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2791,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11357,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 44711,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7998,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28455,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 111750,
            "range": "± 589",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 8419,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 29045,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 111701,
            "range": "± 764",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20561,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79361,
            "range": "± 591",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 328816,
            "range": "± 1473",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1339633,
            "range": "± 5051",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12905,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78790,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 348250,
            "range": "± 7308",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1440562,
            "range": "± 7247",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "80c2eb69a9d773025cb057a52fa3df949e715f36",
          "message": "Merge pull request #50 from orcvs/07-make-the-comment-a-parser-unit\n\nSpell the Comment as a Parser unit",
          "timestamp": "2026-09-10T13:26:21+10:00",
          "tree_id": "4c0fabf2e5c279041a7ead4b7966a41f3e530024",
          "url": "https://github.com/orcvs/orcvs/commit/80c2eb69a9d773025cb057a52fa3df949e715f36"
        },
        "date": 1789010932669,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 186,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 814,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 270,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 499,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 957,
            "range": "± 147",
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
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3127,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12787,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 50860,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6408,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24211,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 106050,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6668,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24370,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 108027,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19546,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77431,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 315040,
            "range": "± 1170",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1262314,
            "range": "± 12989",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12433,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76111,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 343080,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1358247,
            "range": "± 3056",
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
          "id": "d3bdda8a671826e492e8bfdcf95ec2cb3e7373df",
          "message": "Merge pull request #49 from orcvs/06-project-a-sequence-result-into-source\n\nProject a Sequence result into Source",
          "timestamp": "2026-09-10T13:34:02+10:00",
          "tree_id": "978c93d19fbc6caa2a2609fd239222b390b9fd35",
          "url": "https://github.com/orcvs/orcvs/commit/d3bdda8a671826e492e8bfdcf95ec2cb3e7373df"
        },
        "date": 1789011389752,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 188,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 83,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 834,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 113,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 278,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 512,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 895,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3024,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11923,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45920,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6701,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24793,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 103977,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6932,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24703,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 90254,
            "range": "± 2088",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20189,
            "range": "± 265",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76808,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 315906,
            "range": "± 3346",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1315311,
            "range": "± 60904",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12685,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75306,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 345501,
            "range": "± 8049",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1441159,
            "range": "± 12775",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "03ecea63b816e9963b820a385bdf0d49473b6233",
          "message": "Merge pull request #51 from orcvs/02-add-clock-delay-and-euclidean\n\nAdd the Clock, Delay, and Euclidean Functions",
          "timestamp": "2026-09-10T14:18:06+10:00",
          "tree_id": "16edd31a9d99af7ad26c1f52721abade99488b19",
          "url": "https://github.com/orcvs/orcvs/commit/03ecea63b816e9963b820a385bdf0d49473b6233"
        },
        "date": 1789014105599,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 187,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 833,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 273,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 508,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 981,
            "range": "± 16",
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
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3134,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12765,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 51457,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6501,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24399,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 92298,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6629,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24595,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 106829,
            "range": "± 512",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20082,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78433,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 314486,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1276734,
            "range": "± 7973",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13221,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76224,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 345250,
            "range": "± 1674",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1378735,
            "range": "± 6281",
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
          "id": "b1d67b7bbb75513def5d7110842c74be30b9f8cd",
          "message": "Merge pull request #52 from orcvs/01-give-a-late-tick-one-policy\n\nGive a late Tick one policy",
          "timestamp": "2026-09-10T14:54:47+10:00",
          "tree_id": "4809dbb66221c29a0e9bb358420b574471186a6a",
          "url": "https://github.com/orcvs/orcvs/commit/b1d67b7bbb75513def5d7110842c74be30b9f8cd"
        },
        "date": 1789016236556,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 192,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 82,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 876,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 116,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 273,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 515,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 899,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3009,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11847,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45711,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7213,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 25994,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 94762,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7475,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26409,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 94693,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20380,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77817,
            "range": "± 806",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 313387,
            "range": "± 1190",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1293409,
            "range": "± 3976",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13028,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 75177,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 348333,
            "range": "± 1119",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1429294,
            "range": "± 4185",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c8669dcc41225952595813ba78abcdd0255e166",
          "message": "Merge pull request #53 from orcvs/01-re-site-the-injected-value-tests\n\nMove the injected-value tests below the planning entry point, and delete the seam",
          "timestamp": "2026-09-10T16:28:16+10:00",
          "tree_id": "14babad2e753889e678d375cfabe051be8f52653",
          "url": "https://github.com/orcvs/orcvs/commit/7c8669dcc41225952595813ba78abcdd0255e166"
        },
        "date": 1789021845640,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 206,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 875,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 116,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 273,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 516,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 977,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3005,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11835,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 46003,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7117,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 25436,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 106250,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7188,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26000,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 107276,
            "range": "± 4825",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19424,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76945,
            "range": "± 848",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 314941,
            "range": "± 1082",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1301305,
            "range": "± 2079",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12989,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 77000,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 350357,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1454405,
            "range": "± 8725",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3c4424d18ef95eab30ff5e8f034b817e15453aa9",
          "message": "Merge pull request #54 from orcvs/04-build-a-destination-carrying-schedule\n\nBuild a destination-carrying schedule a test can construct",
          "timestamp": "2026-09-10T17:48:55+10:00",
          "tree_id": "d821c532b4642e0e01b6a3a98f744c7b29167c91",
          "url": "https://github.com/orcvs/orcvs/commit/3c4424d18ef95eab30ff5e8f034b817e15453aa9"
        },
        "date": 1789026682219,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 187,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 104,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 832,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 273,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 505,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 977,
            "range": "± 1",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3100,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12715,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 50920,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6387,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24241,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 91406,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6593,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24685,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 99136,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20732,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79623,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 320142,
            "range": "± 1496",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1307901,
            "range": "± 7670",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12968,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78162,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 352651,
            "range": "± 9621",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1413067,
            "range": "± 5728",
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
          "id": "3406af1b6653b0ddd19cf911ebf74a00c830c0e2",
          "message": "Merge pull request #57 from orcvs/09-give-a-computations-reservation-one-home\n\nGive a computation's reservation one home",
          "timestamp": "2026-09-10T19:49:37+10:00",
          "tree_id": "81a135e5e429c929edbd3228c69f15e462c15878",
          "url": "https://github.com/orcvs/orcvs/commit/3406af1b6653b0ddd19cf911ebf74a00c830c0e2"
        },
        "date": 1789033922811,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 189,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 94,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 832,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 273,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 517,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 915,
            "range": "± 4",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3162,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12729,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 50792,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6586,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24632,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 107738,
            "range": "± 965",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6750,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24663,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 100220,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20670,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77424,
            "range": "± 2418",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 319748,
            "range": "± 1642",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1287428,
            "range": "± 13398",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13113,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 77138,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 348740,
            "range": "± 8962",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1394548,
            "range": "± 9083",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b3937425a075deb58a8215560e26e33c5cde087a",
          "message": "Merge pull request #55 from orcvs/05-migrate-the-ordering-and-dependency-destination-tests\n\nMigrate the ordering and dependency destination tests",
          "timestamp": "2026-09-10T20:52:02+10:00",
          "tree_id": "d85ffec4661d85b5ea341384b988efc82ad22e70",
          "url": "https://github.com/orcvs/orcvs/commit/b3937425a075deb58a8215560e26e33c5cde087a"
        },
        "date": 1789037655679,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 126,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 56,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 507,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 166,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 319,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 691,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 35,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 1946,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 7988,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 31141,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 4575,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 16123,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 65063,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 4677,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 16408,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 55439,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 13021,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 50943,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 203225,
            "range": "± 12833",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1040429,
            "range": "± 5360",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 8487,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 50184,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 232050,
            "range": "± 928",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1000269,
            "range": "± 1557",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0a3468510e3804446bb3e9dd39034e0335990446",
          "message": "Merge pull request #56 from orcvs/06-migrate-the-remaining-destination-tests\n\nMigrate the remaining destination tests, and say what a Tick evaluated",
          "timestamp": "2026-09-10T22:37:56+10:00",
          "tree_id": "c92b85e9d97397652caf5686c015deac8dada5e3",
          "url": "https://github.com/orcvs/orcvs/commit/0a3468510e3804446bb3e9dd39034e0335990446"
        },
        "date": 1789044032715,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 184,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 828,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 116,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 275,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 509,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 914,
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
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3114,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12820,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 50778,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6766,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 25516,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 99436,
            "range": "± 617",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6802,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26231,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 98713,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20602,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81153,
            "range": "± 4116",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 320066,
            "range": "± 2023",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1306850,
            "range": "± 10984",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13435,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 77986,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 353590,
            "range": "± 3209",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1428241,
            "range": "± 14892",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "68e5eba38ed97428bc53bc18d510f9b6df926a2b",
          "message": "Merge pull request #58 from orcvs/08-delete-the-configuration-and-its-parameter\n\nDelete the scheduler configuration and the parameter that carried it",
          "timestamp": "2026-09-10T22:59:23+10:00",
          "tree_id": "7e2c58e3c239e13e53aca9483266e65b83feaebf",
          "url": "https://github.com/orcvs/orcvs/commit/68e5eba38ed97428bc53bc18d510f9b6df926a2b"
        },
        "date": 1789045326968,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 189,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 78,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 832,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 279,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 505,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 975,
            "range": "± 3",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3089,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12758,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 51258,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6548,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24370,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 107342,
            "range": "± 633",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6582,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24659,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 108690,
            "range": "± 584",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19970,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79574,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 320934,
            "range": "± 763",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1302498,
            "range": "± 6576",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13165,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76634,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 350329,
            "range": "± 1266",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1394952,
            "range": "± 24113",
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
          "id": "e6a53d85ef1b48305b0795030bce1598888ba1a5",
          "message": "Merge pull request #61 from orcvs/01-rename-the-shell-crate-to-console\n\nRename the shell crate to console",
          "timestamp": "2026-09-11T12:00:38+10:00",
          "tree_id": "43260d1bee18eec4b6c9009c7ffc60a649ff17d3",
          "url": "https://github.com/orcvs/orcvs/commit/e6a53d85ef1b48305b0795030bce1598888ba1a5"
        },
        "date": 1789092671014,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 173,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 67,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 83,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 711,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 58,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 217,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 425,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 730,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2340,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9216,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 35671,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6470,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 23307,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 91950,
            "range": "± 3304",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6507,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23251,
            "range": "± 946",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 98720,
            "range": "± 4079",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 15843,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 61850,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 253978,
            "range": "± 1806",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1172635,
            "range": "± 31814",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10218,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 59825,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 278289,
            "range": "± 1608",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1274673,
            "range": "± 17615",
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
          "id": "2f76622d5652c1a7ca1f94d97972c54b0d4f976e",
          "message": "Merge pull request #60 from orcvs/02-concentrate-clock-loop\n\nConcentrate the Playback clock loop",
          "timestamp": "2026-09-11T13:17:26+10:00",
          "tree_id": "78025ff01e30cb007a769e035417e194a97decbf",
          "url": "https://github.com/orcvs/orcvs/commit/2f76622d5652c1a7ca1f94d97972c54b0d4f976e"
        },
        "date": 1789096793341,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 188,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 842,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 71,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 272,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 504,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 975,
            "range": "± 2",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3121,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12893,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 51376,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7227,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 25621,
            "range": "± 527",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 111545,
            "range": "± 2392",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6979,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26157,
            "range": "± 705",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 98719,
            "range": "± 2108",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20840,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78918,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 316093,
            "range": "± 1094",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1284285,
            "range": "± 7042",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12985,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78746,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 348614,
            "range": "± 2252",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1388972,
            "range": "± 13160",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "84f03be5d44533bab7076d28dd98d578e3da9fc7",
          "message": "Merge pull request #62 from orcvs/03-migrate-self-banging-functions-to-the-function-table\n\nAdd the Self-Banging Functions `^^ vv << >>`",
          "timestamp": "2026-09-11T14:12:24+10:00",
          "tree_id": "2804758db841910065960c6966dc0d2de9550136",
          "url": "https://github.com/orcvs/orcvs/commit/84f03be5d44533bab7076d28dd98d578e3da9fc7"
        },
        "date": 1789100111357,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 185,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 78,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 105,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 880,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 276,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 513,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 976,
            "range": "± 2",
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
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3136,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12889,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 51030,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6310,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24451,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 100574,
            "range": "± 877",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6654,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24628,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 100873,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20465,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77526,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 318181,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1284898,
            "range": "± 2442",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13327,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 80033,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 358955,
            "range": "± 1246",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1438557,
            "range": "± 11939",
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
          "id": "42fe2bd62ec68dcc6ee3062c1761ac63643fccb0",
          "message": "Merge pull request #63 from orcvs/06-add-the-directional-bang-functions\n\nAdd the Directional Bang Functions `*^ *v *< *>`",
          "timestamp": "2026-09-11T14:35:06+10:00",
          "tree_id": "6165ccbb6e8bb291db2c634f4abcb7a5da39c0a6",
          "url": "https://github.com/orcvs/orcvs/commit/42fe2bd62ec68dcc6ee3062c1761ac63643fccb0"
        },
        "date": 1789101440181,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 164,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 731,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 222,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 414,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 776,
            "range": "± 7",
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
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2348,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9178,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 35553,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5820,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 21098,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 86844,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5936,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 21182,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 87715,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 16034,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 59858,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 245104,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1020195,
            "range": "± 3387",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10486,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 62294,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 280992,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1156921,
            "range": "± 3915",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e3c18ad48f91b666887a3bd0a680631ed1a2ab92",
          "message": "Merge pull request #68 from orcvs/04-coalesce-cell-backgrounds\n\nCoalesce Cell backgrounds, and hold the fill they stand on",
          "timestamp": "2026-09-11T15:33:02+10:00",
          "tree_id": "791600ce3e569b52b4e2c89b1ee9ee0c08756949",
          "url": "https://github.com/orcvs/orcvs/commit/e3c18ad48f91b666887a3bd0a680631ed1a2ab92"
        },
        "date": 1789104937223,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 195,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 849,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 117,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 277,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 525,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1012,
            "range": "± 12",
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
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3152,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12619,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 50987,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6605,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24321,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 92519,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6622,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24362,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 100150,
            "range": "± 1090",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20231,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79555,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 321689,
            "range": "± 2607",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1306583,
            "range": "± 9198",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13675,
            "range": "± 758",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 81085,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 361780,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1452393,
            "range": "± 17346",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ffdd4c04538a5fa413b06f75fe6ee040abd84ca",
          "message": "Merge pull request #71 from orcvs/function-replacement\n\nName which fact a refused Function replacement changes",
          "timestamp": "2026-09-11T17:34:13+10:00",
          "tree_id": "d6cf1b6ce5cae228f0028afa6c4e8048a24a9e61",
          "url": "https://github.com/orcvs/orcvs/commit/2ffdd4c04538a5fa413b06f75fe6ee040abd84ca"
        },
        "date": 1789112201535,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 206,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 894,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 280,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 538,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 992,
            "range": "± 1",
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
            "value": 45,
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
            "value": 3029,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11849,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45981,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6886,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24601,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 97454,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7093,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 25031,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 97755,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20092,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 74473,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 312891,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1291468,
            "range": "± 6756",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13352,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78084,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 357370,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1477052,
            "range": "± 4454",
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
          "id": "38b2eb364477ddbead8d56280f6508a9aa7ba2bf",
          "message": "Merge pull request #73 from orcvs/fix-the-output-failure-latch\n\nScope the output failure latch to the run that recorded it",
          "timestamp": "2026-09-11T18:22:34+10:00",
          "tree_id": "62724ef942c7e47cb4d9f1562653e785c88d2fee",
          "url": "https://github.com/orcvs/orcvs/commit/38b2eb364477ddbead8d56280f6508a9aa7ba2bf"
        },
        "date": 1789115109619,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 198,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 104,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 850,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 116,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 276,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 529,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 949,
            "range": "± 2",
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
            "value": 104,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3149,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12803,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 51251,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6633,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24400,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 92501,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6643,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24583,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 108020,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20549,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78280,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 314368,
            "range": "± 1544",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1270293,
            "range": "± 8282",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13401,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78842,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 356007,
            "range": "± 2172",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1435050,
            "range": "± 9527",
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
          "id": "d2d2ac60c6d8c7a29b0be75d0bf2cef034d0afb9",
          "message": "Point the Self-Banging advice at the method that exists\n\ntakes_no_operand's doc linked Function::source_write, which no crate declares.\nIt is the only rustdoc error in the workspace and it fails the merge tier's\ngate under RUSTDOCFLAGS=\"-D warnings\", so full-gate has been red on main rather\nthan on any one branch — PR #71 and PR #73 each merged and each inherited it.\n\nFunction::source_effect is the method meant, but naming it was not the whole\nfix. Both groups declare an effect, so a caller told to ask source_effect and\nnothing more would have been given a second untrue sentence in place of the\nfirst. The effect table decides it: every Self-Banging arm declares\nSourceBundle::Advance and every Directional Bang arm SourceBundle::Emit, so the\nbundle is what tells them apart and the sentence now says to read it.\n\nPushed straight to main. The gate is one of the three required status checks and\nwas failing before this change on every branch equally, so a pull request of its\nown could not have turned it green any sooner.\n\nCloses .scratch/lang-foundations/issues/09.\n\nClaude-Session: https://claude.ai/code/session_017SYs8gt3hywhcyxxM2L13j",
          "timestamp": "2026-09-11T18:44:28+10:00",
          "tree_id": "72243a435d8340110470399cce1a23f0f1174d41",
          "url": "https://github.com/orcvs/orcvs/commit/d2d2ac60c6d8c7a29b0be75d0bf2cef034d0afb9"
        },
        "date": 1789116443023,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 169,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 80,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 692,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 97,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 247,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 462,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 880,
            "range": "± 1",
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
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2943,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11772,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45764,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6523,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 23191,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 85709,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6706,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23304,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 86282,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19818,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 73853,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 297601,
            "range": "± 1161",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1237310,
            "range": "± 3178",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12493,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 74409,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 342548,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1386988,
            "range": "± 2692",
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
          "id": "7425a1a7b1eac79f04e13668e9660c3f980df67f",
          "message": "Merge pull request #74 from orcvs/renumber-the-pulse-adr\n\nGive the pulse decision a number of its own",
          "timestamp": "2026-09-11T09:11:53Z",
          "tree_id": "5699c61c0efb0ac74289b0568d880c78393e9ddf",
          "url": "https://github.com/orcvs/orcvs/commit/7425a1a7b1eac79f04e13668e9660c3f980df67f"
        },
        "date": 1789118307528,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 206,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 87,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 883,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 118,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 287,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 535,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1002,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3052,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11861,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45966,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6804,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24890,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 106442,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7038,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 25018,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 106476,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 19792,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 74812,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 310906,
            "range": "± 1626",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1285463,
            "range": "± 5483",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13318,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78341,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 361373,
            "range": "± 1272",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1467000,
            "range": "± 3464",
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
          "id": "f81962c14b253edc6760e381b7c6fc3a81437253",
          "message": "Merge pull request #72 from orcvs/function-replacement-04-05\n\nGenerate the replacement facts, and prove the width term where the widths live",
          "timestamp": "2026-09-11T09:52:23Z",
          "tree_id": "0c9ffe8a065b10faa5e958cf35f42a4cd7cc3414",
          "url": "https://github.com/orcvs/orcvs/commit/f81962c14b253edc6760e381b7c6fc3a81437253"
        },
        "date": 1789120731996,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 208,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 897,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 121,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 282,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 540,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 937,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 51,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3007,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11890,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 45797,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6824,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24841,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 91026,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7037,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 25328,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 105858,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20053,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76401,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 313349,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1283100,
            "range": "± 3696",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13274,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 77871,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 360588,
            "range": "± 954",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1464015,
            "range": "± 66800",
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
          "id": "2d2674916e679e4503fe6842353b009b62b4072e",
          "message": "Merge pull request #75 from orcvs/observable-schedule\n\nSay which Turn a computation took",
          "timestamp": "2026-09-12T02:49:03Z",
          "tree_id": "6dbe0cd00cc5a196d7a211f6385dc758cc76911f",
          "url": "https://github.com/orcvs/orcvs/commit/2d2674916e679e4503fe6842353b009b62b4072e"
        },
        "date": 1789182961498,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 164,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 743,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 96,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 221,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 415,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 768,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 42,
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
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2335,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9267,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 35591,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5805,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 21123,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 75968,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5810,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 21244,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 87181,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 16072,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 60978,
            "range": "± 1445",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 248798,
            "range": "± 995",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1022326,
            "range": "± 7968",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10574,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 61668,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 283316,
            "range": "± 1119",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1170562,
            "range": "± 18403",
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
          "id": "e32c1ba4b0771e26c26fba1c4d097a989d772cab",
          "message": "Merge pull request #78 from orcvs/stop-running-the-benchmarks-as-tests\n\nStop running the benchmarks as tests in the merge tier",
          "timestamp": "2026-09-12T05:05:07Z",
          "tree_id": "ad7cb054bef8cba6cee24dfd8d7b0b77f906e568",
          "url": "https://github.com/orcvs/orcvs/commit/e32c1ba4b0771e26c26fba1c4d097a989d772cab"
        },
        "date": 1789190998008,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 133,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 61,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 590,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 172,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 339,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 614,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 36,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 1958,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 8040,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 31391,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 4547,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 16352,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 61833,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 4695,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 16575,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 62596,
            "range": "± 4343",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 13258,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 49944,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 199776,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 892712,
            "range": "± 40660",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 8742,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 51790,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 255418,
            "range": "± 10438",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1041766,
            "range": "± 9716",
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
          "id": "cda06323d391a8c861f4a234a44e0a80642d6f44",
          "message": "Merge pull request #69 from orcvs/01-measure-what-a-render-frame-costs\n\nMeasure a Render Frame at the sizes a resizable Grid would reach",
          "timestamp": "2026-09-12T05:54:43Z",
          "tree_id": "527f3e34e5025d6efd09616996fa11a3bcd1695d",
          "url": "https://github.com/orcvs/orcvs/commit/cda06323d391a8c861f4a234a44e0a80642d6f44"
        },
        "date": 1789193078822,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 194,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 80,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 849,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 117,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 276,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 526,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1016,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 58,
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 221,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1390,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3196,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12905,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 51384,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 206365,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 821202,
            "range": "± 3047",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6409,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24150,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 98653,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6505,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23910,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 101461,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20201,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78275,
            "range": "± 753",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 317720,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1287081,
            "range": "± 7799",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13432,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 80383,
            "range": "± 584",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 360098,
            "range": "± 1933",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1419052,
            "range": "± 5447",
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
          "id": "8b9491bbeba82b12ec14d938dd912afa1dc1f45a",
          "message": "Merge pull request #80 from orcvs/cull-source-paint\n\nLet the Source Grid paint answer instead of take",
          "timestamp": "2026-09-12T10:57:22Z",
          "tree_id": "629d8690ec93288a17706c92be541cc7b0dd39aa",
          "url": "https://github.com/orcvs/orcvs/commit/8b9491bbeba82b12ec14d938dd912afa1dc1f45a"
        },
        "date": 1789211185890,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 164,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 745,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 96,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 226,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 414,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 721,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 42,
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
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 190,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1207,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2311,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9134,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 35124,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 139404,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 571303,
            "range": "± 826",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5713,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 20841,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 80435,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5896,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 20926,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 81050,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 16147,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 60702,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 247259,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1021113,
            "range": "± 6163",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10634,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 61461,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 282950,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1162848,
            "range": "± 3235",
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
          "id": "bd9ee4bb32af4d1fab30d060179d639056e1c6d5",
          "message": "Merge pull request #79 from orcvs/playback-actor\n\nGive the Playback Engine's state an owner",
          "timestamp": "2026-09-12T11:43:00Z",
          "tree_id": "926296a9322e46959cda572195c3fce3b5b5018a",
          "url": "https://github.com/orcvs/orcvs/commit/bd9ee4bb32af4d1fab30d060179d639056e1c6d5"
        },
        "date": 1789214015089,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 192,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 847,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 116,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 276,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 525,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1013,
            "range": "± 2",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 221,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1383,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3065,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12561,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 49872,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 200322,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 812770,
            "range": "± 1391",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7735,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 30129,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 119450,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 8079,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 29827,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 119357,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22343,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79916,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 322273,
            "range": "± 2679",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1347563,
            "range": "± 9591",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14235,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 82102,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 355241,
            "range": "± 2112",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1506758,
            "range": "± 3002",
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
          "id": "f67e04aba5948980581b83af4953b7edf528dc6e",
          "message": "Merge pull request #81 from orcvs/triage/render-frame-responsibility\n\nLand the resolved render-frame-responsibility tickets",
          "timestamp": "2026-09-13T00:29:27Z",
          "tree_id": "5ef412632fc9b47f366ecb78df7487355427a33a",
          "url": "https://github.com/orcvs/orcvs/commit/f67e04aba5948980581b83af4953b7edf528dc6e"
        },
        "date": 1789259962582,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 192,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 984,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 289,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 536,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1028,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 293,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1683,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 4410,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 18720,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 77059,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 313089,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 1307595,
            "range": "± 4897",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7451,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28596,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 115116,
            "range": "± 919",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7774,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28702,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 116225,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21771,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79930,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 321154,
            "range": "± 2031",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1343750,
            "range": "± 3840",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14046,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 82314,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 351429,
            "range": "± 2087",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1509597,
            "range": "± 5720",
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
          "id": "8d93e2a45b5143f80263f1d15b205ed76d01c787",
          "message": "Merge pull request #83 from orcvs/rustdoc-on-pull-request\n\nRun rustdoc on the pull-request tier",
          "timestamp": "2026-09-13T00:49:23Z",
          "tree_id": "40e07dff20a500b9eadb99677a67807f55bce505",
          "url": "https://github.com/orcvs/orcvs/commit/8d93e2a45b5143f80263f1d15b205ed76d01c787"
        },
        "date": 1789262641829,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 205,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 883,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 118,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 288,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 531,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 989,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
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
            "name": "source_read_revision/128x128",
            "value": 270,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1545,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 4290,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 16646,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 67045,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 272842,
            "range": "± 1747",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 1125520,
            "range": "± 17463",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7342,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 27397,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 111096,
            "range": "± 1343",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7619,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 27428,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 110698,
            "range": "± 1071",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21219,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 75621,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 312821,
            "range": "± 1252",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1300192,
            "range": "± 3775",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14060,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 79978,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 345330,
            "range": "± 2739",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1499528,
            "range": "± 6214",
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
          "id": "3c534cc19e2839b9b1c78cf3e41b658cad349429",
          "message": "Merge pull request #82 from orcvs/render-frame-responsibility-02\n\nSeparate glyph placement and record the crate-seam criterion",
          "timestamp": "2026-09-13T03:03:58Z",
          "tree_id": "e3c633f03bcbba0cee7ee815a310a5e94ea9d152",
          "url": "https://github.com/orcvs/orcvs/commit/3c534cc19e2839b9b1c78cf3e41b658cad349429"
        },
        "date": 1789269338054,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 167,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 691,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 245,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 464,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 888,
            "range": "± 5",
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
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1504,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 2426,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 9518,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 37927,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 218930,
            "range": "± 3792",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 627988,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7038,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 26298,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 130459,
            "range": "± 4725",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7353,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26031,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 130745,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20553,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 72705,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 323285,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1353158,
            "range": "± 11763",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13151,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 74335,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 348347,
            "range": "± 2873",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1576312,
            "range": "± 34997",
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
          "id": "b3d5f8918a5c71f4cae58bbf5207db3b5b7d66cc",
          "message": "Merge pull request #84 from orcvs/typed-source-paint-tokens\n\nCarry Tokens through the Render Frame and retire Glyph",
          "timestamp": "2026-09-13T03:57:06Z",
          "tree_id": "90909e00161fd10d611a77782f329ce227ad6a5a",
          "url": "https://github.com/orcvs/orcvs/commit/b3d5f8918a5c71f4cae58bbf5207db3b5b7d66cc"
        },
        "date": 1789272448526,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 208,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 904,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 118,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 286,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 529,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 982,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 256,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1558,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3398,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 13884,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 68694,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 350607,
            "range": "± 2867",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 1846875,
            "range": "± 14251",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7154,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 26883,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 111867,
            "range": "± 860",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7215,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 27390,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 112521,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20148,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 74215,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 310932,
            "range": "± 3115",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1303276,
            "range": "± 4493",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 12891,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76678,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 344251,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1486762,
            "range": "± 6779",
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
          "id": "3bad9af387dca4deb63517a2a6fbd5c2a4377a4b",
          "message": "Merge pull request #85 from orcvs/seam-names-and-frame-diagnostics\n\nCarry paint diagnostics and retire leftover Marker names",
          "timestamp": "2026-09-13T05:04:21Z",
          "tree_id": "81d66544a90850488a7f7ba8402163a0f7885358",
          "url": "https://github.com/orcvs/orcvs/commit/3bad9af387dca4deb63517a2a6fbd5c2a4377a4b"
        },
        "date": 1789276426315,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 194,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 79,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 847,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 117,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 276,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 531,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1011,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 35,
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
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 254,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1375,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5119,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20804,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 108571,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 464186,
            "range": "± 1708",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2669065,
            "range": "± 11158",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6915,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28270,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 114609,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7275,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28129,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 114030,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20673,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77482,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 313032,
            "range": "± 1357",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1309650,
            "range": "± 24832",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13132,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 79887,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 348276,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1482523,
            "range": "± 8042",
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
          "id": "d309bd2f638f0a2efd7b46d3830a45780c5f4b50",
          "message": "Merge pull request #88 from orcvs/04-add-deterministic-random\n\nAdd deterministic Random",
          "timestamp": "2026-09-13T06:58:52Z",
          "tree_id": "68eb95f817ddb2a703514119b4be03e736c6914e",
          "url": "https://github.com/orcvs/orcvs/commit/d309bd2f638f0a2efd7b46d3830a45780c5f4b50"
        },
        "date": 1789283274669,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 50,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 517,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 173,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 338,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 648,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 118,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1441,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3485,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12960,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 60091,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 329657,
            "range": "± 9192",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2100007,
            "range": "± 9352",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 4365,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 16377,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 76109,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 4418,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 16245,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 74223,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 12646,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 47774,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 213585,
            "range": "± 869",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1001312,
            "range": "± 7050",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 8557,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 50061,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 247140,
            "range": "± 15979",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1111617,
            "range": "± 38328",
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
          "id": "65f3608f8ded8738ce3092c6a7c31f08894d0b10",
          "message": "Merge pull request #86 from orcvs/03-add-visible-increment-and-interpolation\n\nAdd Increment and Interpolation with declared Portal inputs",
          "timestamp": "2026-09-13T17:59:12+10:00",
          "tree_id": "bfb5475911ca2c33b03a6842ae3793b5916f1571",
          "url": "https://github.com/orcvs/orcvs/commit/65f3608f8ded8738ce3092c6a7c31f08894d0b10"
        },
        "date": 1789286722536,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse",
            "value": 197,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 903,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 83,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 132,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 301,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 565,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1091,
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
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 252,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1373,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5130,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20235,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 106366,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 467674,
            "range": "± 2418",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2618783,
            "range": "± 17219",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6596,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 25335,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 103442,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6800,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 25625,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 104232,
            "range": "± 403",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20487,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76270,
            "range": "± 1770",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 307282,
            "range": "± 1549",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1260434,
            "range": "± 14517",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13272,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 78568,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 340969,
            "range": "± 2424",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1446506,
            "range": "± 8860",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 24870,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 103115,
            "range": "± 1099",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 426492,
            "range": "± 2587",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1756799,
            "range": "± 12679",
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
          "id": "f23fad22d2d073cdcd94d208eea1d8e1cc6f0330",
          "message": "Merge pull request #89 from orcvs/source-paint-09\n\nMeasure Paint derivation so viewport cost is on the series",
          "timestamp": "2026-09-13T08:13:18Z",
          "tree_id": "b2e67e4f2bac99251ff6fe5033dd2c6a23e2eca6",
          "url": "https://github.com/orcvs/orcvs/commit/f23fad22d2d073cdcd94d208eea1d8e1cc6f0330"
        },
        "date": 1789289336477,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1670,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1668,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 6849,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1680,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 29528,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1679,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 124741,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1671,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 523994,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1691,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 353,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 364,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 947,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 297,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 2769,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 166,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 8667,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 166,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 53300,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 166,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 126,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 544,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 73,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 177,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 363,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 666,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1441,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3460,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 12459,
            "range": "± 631",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 56367,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 307881,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2010660,
            "range": "± 13563",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 4547,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 16954,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 84361,
            "range": "± 2056",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 4681,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 17243,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 83226,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 12919,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 48631,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 218212,
            "range": "± 1212",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1000760,
            "range": "± 5091",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 8586,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 51020,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 253253,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1147567,
            "range": "± 6801",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 15730,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 65118,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 329402,
            "range": "± 1697",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1385260,
            "range": "± 4609",
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
          "id": "967b080cdad4fa6d3b83a9cf8f68dfa4a5f13b10",
          "message": "Merge pull request #90 from orcvs/portal-geometry\n\nGive Portal geometry one owner",
          "timestamp": "2026-09-13T08:26:54Z",
          "tree_id": "67546339d9a6c4866e459c70194950a5b0bc4bb0",
          "url": "https://github.com/orcvs/orcvs/commit/967b080cdad4fa6d3b83a9cf8f68dfa4a5f13b10"
        },
        "date": 1789289882912,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1899,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1898,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 8030,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1903,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 35008,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1909,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 149381,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1904,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 623318,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1903,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 465,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 473,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1110,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 388,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 3572,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 240,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 14628,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 229,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 62505,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 240,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 140,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 59,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 598,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 81,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 197,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 373,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 668,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 137,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1429,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 4043,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 14929,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 70646,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 361381,
            "range": "± 3593",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2034267,
            "range": "± 3187",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5334,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 19193,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 87047,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5448,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 19428,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 89710,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 15179,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 54264,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 239233,
            "range": "± 2379",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1098234,
            "range": "± 3655",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10215,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 57516,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 279140,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1259163,
            "range": "± 3320",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 18609,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 73502,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 367611,
            "range": "± 1263",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1509342,
            "range": "± 5806",
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
          "id": "398926b96c66f7ea444dfb044c9c0b4b7b87ae6f",
          "message": "Merge pull request #93 from orcvs/03-05-sequence-functions\n\nAdd structural and Range Sequence Functions (issues 03 & 05)",
          "timestamp": "2026-09-13T23:40:48Z",
          "tree_id": "bb7e22a675a01293b5cbe6e8edbd0c49a24c5fd3",
          "url": "https://github.com/orcvs/orcvs/commit/398926b96c66f7ea444dfb044c9c0b4b7b87ae6f"
        },
        "date": 1789343603891,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2548,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2548,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 10354,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2557,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 42597,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2564,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 174376,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2565,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 726876,
            "range": "± 2462",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2562,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 551,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 554,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1434,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 487,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4758,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 17940,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 302,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 72949,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 214,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 87,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 109,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 938,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 290,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 553,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 979,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 57,
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
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 269,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1555,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5369,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20901,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 103776,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 499787,
            "range": "± 1483",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2647133,
            "range": "± 342067",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7172,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 27588,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 116752,
            "range": "± 1009",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7523,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 27812,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 118198,
            "range": "± 932",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20544,
            "range": "± 2094",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 77177,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 322147,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1387739,
            "range": "± 11428",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13506,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 80487,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 352895,
            "range": "± 1639",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1569751,
            "range": "± 10920",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 25785,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 108751,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 463224,
            "range": "± 2723",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1993711,
            "range": "± 32875",
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
          "id": "d4013e6328c6a001cf7d44078bb52ed57ecbdb35",
          "message": "Merge pull request #92 from orcvs/dependabot/cargo/rust-dependencies-06ff8b9c65\n\nBump the rust-dependencies group with 2 updates",
          "timestamp": "2026-09-13T23:40:53Z",
          "tree_id": "79fa9d04f797df6c300557e70dea34b09b21e67a",
          "url": "https://github.com/orcvs/orcvs/commit/d4013e6328c6a001cf7d44078bb52ed57ecbdb35"
        },
        "date": 1789344246387,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2610,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2609,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 10690,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2620,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 43648,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2630,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 179501,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2625,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 738175,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2624,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 483,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 485,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1115,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 397,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 3454,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 293,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 12308,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 293,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 50898,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 293,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 216,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 935,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 75,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 294,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 555,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 991,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 112,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 273,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1538,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5438,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20777,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 106198,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 481461,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2576107,
            "range": "± 24975",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7275,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 27274,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 115214,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7326,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 27509,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 115367,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20374,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 76259,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 317163,
            "range": "± 1058",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1316422,
            "range": "± 2823",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13508,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 79425,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 350238,
            "range": "± 889",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1522287,
            "range": "± 7924",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 25682,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 106433,
            "range": "± 5953",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 452326,
            "range": "± 2415",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1886821,
            "range": "± 7374",
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
          "id": "3ccf1e4f62705be1c78619685c8688be0b048c13",
          "message": "Merge pull request #95 from orcvs/live-note-range-happy-path\n\nAdd live happy-path tests for Sequence Functions",
          "timestamp": "2026-09-14T00:31:28Z",
          "tree_id": "680e6dd11f803e17cc0d2dccb6cfef5d3fba117d",
          "url": "https://github.com/orcvs/orcvs/commit/3ccf1e4f62705be1c78619685c8688be0b048c13"
        },
        "date": 1789346714560,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2794,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2796,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 11861,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2728,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 51347,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2748,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 211673,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2751,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 844038,
            "range": "± 2219",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2753,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 555,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 544,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1106,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 411,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 3756,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 220,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 12884,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 225,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 46695,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 223,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 203,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 899,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 291,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 544,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1043,
            "range": "± 2",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 250,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1372,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5177,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20574,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 106839,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 463083,
            "range": "± 1620",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2871760,
            "range": "± 77610",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7015,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28652,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 120281,
            "range": "± 1531",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7377,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 29008,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 118973,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21147,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 80307,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 328234,
            "range": "± 913",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1373529,
            "range": "± 6035",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13705,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 82424,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 361792,
            "range": "± 1371",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1546898,
            "range": "± 7310",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 26152,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 109981,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 460760,
            "range": "± 1146",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1880554,
            "range": "± 13697",
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
          "id": "406a4642742085f95ecd09e0661e8902f043b40e",
          "message": "Merge pull request #94 from orcvs/03-add-the-self-banging-functions\n\nRaise ADR 0009's Terminal Output Portal refusal in shipped code",
          "timestamp": "2026-09-14T00:37:23Z",
          "tree_id": "eb051146f8d24e7b02aaf05b1e2b4cc66e7e0f81",
          "url": "https://github.com/orcvs/orcvs/commit/406a4642742085f95ecd09e0661e8902f043b40e"
        },
        "date": 1789347293899,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2724,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2724,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 11712,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2670,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 50927,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2692,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 210505,
            "range": "± 2624",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2684,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 836540,
            "range": "± 1290",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2679,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 535,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 543,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1109,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 418,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 3866,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 234,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 12959,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 221,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 47417,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 227,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 210,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 81,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 899,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 292,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 542,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1041,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 34,
            "range": "± 1",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 252,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1373,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5033,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 21081,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 108261,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 460752,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2585569,
            "range": "± 3837",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6964,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 27599,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 114288,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7248,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28333,
            "range": "± 1929",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 114175,
            "range": "± 852",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20428,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78257,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 317503,
            "range": "± 7472",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1330692,
            "range": "± 3658",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13357,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 80738,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 350574,
            "range": "± 1098",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1499253,
            "range": "± 2554",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 25590,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 107532,
            "range": "± 2059",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 450474,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1865572,
            "range": "± 5792",
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
          "id": "388f812f373704722231501eaf308b4fcbe763ee",
          "message": "Merge pull request #97 from orcvs/02-prove-product-persistence-paths\n\nProve product persistence through shipped storage paths",
          "timestamp": "2026-09-14T04:53:46Z",
          "tree_id": "8b9b2062476875933e1c8af8d5a1ce972569ec5a",
          "url": "https://github.com/orcvs/orcvs/commit/388f812f373704722231501eaf308b4fcbe763ee"
        },
        "date": 1789362526081,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2754,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2749,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 11828,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2692,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 51034,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2687,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 210148,
            "range": "± 743",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2693,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 834345,
            "range": "± 38188",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2683,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 536,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 537,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1107,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 418,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 3834,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 222,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 12732,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 219,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 47999,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 220,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 209,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 100,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 931,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 121,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 296,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 543,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1051,
            "range": "± 1",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 249,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1375,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5148,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20140,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 107730,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 458873,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2620929,
            "range": "± 13048",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6931,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28250,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 115206,
            "range": "± 935",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7262,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28147,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 116083,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20706,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 78370,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 317545,
            "range": "± 2633",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1330645,
            "range": "± 2524",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13064,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 80306,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 347780,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1499616,
            "range": "± 4549",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 25475,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 106122,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 447649,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1841504,
            "range": "± 4068",
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
          "id": "87fc79362e39c22feb42cc7d425de9dd420edaf6",
          "message": "Merge pull request #98 from orcvs/04-add-directional-jump-chains\n\nAdd directional Jump as ordinary Portal output",
          "timestamp": "2026-09-14T07:01:08Z",
          "tree_id": "32286075eec427a4c6cbd9487bea9185d7cd0cee",
          "url": "https://github.com/orcvs/orcvs/commit/87fc79362e39c22feb42cc7d425de9dd420edaf6"
        },
        "date": 1789370066248,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2277,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2275,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 9166,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2232,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 37591,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2346,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 155029,
            "range": "± 2500",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2276,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 632276,
            "range": "± 9548",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2209,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 456,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 451,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1193,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 378,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4062,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 263,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 15593,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 260,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 62730,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 265,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 195,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 875,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 115,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 277,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 490,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 877,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 47,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 242,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1302,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 4689,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 17673,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 93966,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 419304,
            "range": "± 4619",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2212242,
            "range": "± 151010",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6696,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 25460,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 110418,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6857,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26561,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 105435,
            "range": "± 919",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 18072,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 67903,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 285979,
            "range": "± 3709",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1166444,
            "range": "± 19728",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 11924,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 70603,
            "range": "± 1019",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 312445,
            "range": "± 3044",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1357585,
            "range": "± 15404",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 22870,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 96206,
            "range": "± 3994",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 411414,
            "range": "± 5217",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1703850,
            "range": "± 21826",
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
          "id": "45d18eb91c516e4933d07f20516d4463c136fea9",
          "message": "Merge pull request #99 from orcvs/kind-shaped-portal\n\nIntroduce Destinations silence so Terminal Output cannot be stuffed with a Portal",
          "timestamp": "2026-09-14T10:23:43Z",
          "tree_id": "67aea90da936ab1852c0e791acb001790c6e30bf",
          "url": "https://github.com/orcvs/orcvs/commit/45d18eb91c516e4933d07f20516d4463c136fea9"
        },
        "date": 1789382264197,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1610,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1611,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 6791,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1584,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 29965,
            "range": "± 1852",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1656,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 144936,
            "range": "± 9416",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1654,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 535577,
            "range": "± 3419",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1657,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 346,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 354,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 921,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 279,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 2656,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 161,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 8517,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 161,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 59768,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 160,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 132,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 58,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 644,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 191,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 367,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 678,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 134,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1436,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3400,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 14443,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 55756,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 315938,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 1959461,
            "range": "± 10045",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 4354,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 16112,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 77941,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 4486,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 16176,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 74154,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 12730,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 45748,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 215591,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 987070,
            "range": "± 1606",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 8499,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 49230,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 248778,
            "range": "± 1059",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1122804,
            "range": "± 3301",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 16603,
            "range": "± 1510",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 64021,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 327662,
            "range": "± 693",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1410661,
            "range": "± 2682",
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
          "id": "6baf9572a82fd52e6778e4841493236f7129baa7",
          "message": "Merge pull request #96 from orcvs/05-add-halt-root-locking\n\nAdd Halt root locking",
          "timestamp": "2026-09-14T23:26:20Z",
          "tree_id": "47d6a0d259213f0fd53823719319075148701dd8",
          "url": "https://github.com/orcvs/orcvs/commit/6baf9572a82fd52e6778e4841493236f7129baa7"
        },
        "date": 1789429236355,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1982,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1976,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 8335,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1958,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 36781,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2029,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 156051,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2029,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 645291,
            "range": "± 2695",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2027,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 412,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 414,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 903,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 342,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 2757,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 195,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 11722,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 192,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 61028,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 197,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 149,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 69,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 653,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 206,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 391,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 716,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 179,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1432,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 4032,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 15140,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 72136,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 370401,
            "range": "± 2468",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2186076,
            "range": "± 9172",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5595,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 19970,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 89496,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 5615,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 19893,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 91796,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 15925,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 56457,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 249985,
            "range": "± 827",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1125885,
            "range": "± 2363",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10529,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 60089,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 296515,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1289818,
            "range": "± 5055",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 19737,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 77036,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 382451,
            "range": "± 1292",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1581496,
            "range": "± 7351",
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
          "id": "6aac9fd69f1179fa34a2ee1c986b3f1d194fd8f4",
          "message": "Merge pull request #100 from orcvs/fix/portal-root-locks\n\nDistinguish Portal root locks from Cell writes",
          "timestamp": "2026-09-15T06:20:50Z",
          "tree_id": "d360d35d191fafbd4748eb8632244ab71e4437d1",
          "url": "https://github.com/orcvs/orcvs/commit/6aac9fd69f1179fa34a2ee1c986b3f1d194fd8f4"
        },
        "date": 1789454082617,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2856,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2854,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 12342,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2804,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 52698,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2795,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 216855,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2794,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 855478,
            "range": "± 1662",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2792,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 526,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 535,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1341,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 458,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4776,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 288,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 17260,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 289,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 65157,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 210,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 928,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 290,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 549,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1056,
            "range": "± 3",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 254,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1367,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5068,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20070,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 106640,
            "range": "± 3549",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 452181,
            "range": "± 1263",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 3043056,
            "range": "± 121721",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6978,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28027,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 114479,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7309,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28837,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 115523,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22654,
            "range": "± 639",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 84945,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 339402,
            "range": "± 1861",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1471524,
            "range": "± 11213",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14746,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 86703,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 374628,
            "range": "± 1863",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1617284,
            "range": "± 12616",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 28833,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 117903,
            "range": "± 884",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 489079,
            "range": "± 19514",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2176054,
            "range": "± 30228",
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
          "id": "1a68a192156769a511f21c9721eaaf77a6a4a196",
          "message": "Merge pull request #101 from orcvs/cursor-effects\n\nAdd animated cursor effects and theme controls",
          "timestamp": "2026-09-16T00:41:49Z",
          "tree_id": "757b00a4fcefe6c96c8b8e26626a7be6048dae3e",
          "url": "https://github.com/orcvs/orcvs/commit/1a68a192156769a511f21c9721eaaf77a6a4a196"
        },
        "date": 1789520209512,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1858,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1860,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 7628,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1867,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 31315,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1872,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 128332,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1871,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 536388,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1870,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 324,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 324,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1184,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4812,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 17581,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 71939,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 302,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4378,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 219,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 99,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 118,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 971,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 129,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 299,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 567,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1066,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 119,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 283,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1532,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5400,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20704,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 106347,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 490083,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2506534,
            "range": "± 11854",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7988,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 29367,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 120716,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7859,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 29453,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 121244,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21519,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79559,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 327123,
            "range": "± 948",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1367753,
            "range": "± 4364",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14276,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 82949,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 370687,
            "range": "± 1059",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1545052,
            "range": "± 6130",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27749,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 115292,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 487723,
            "range": "± 1328",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2041339,
            "range": "± 5292",
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
          "id": "e0e60ff2a50eee1521f05ea1785f5a39e1655b20",
          "message": "Fix PWA icon dimensions",
          "timestamp": "2026-09-16T11:02:05+10:00",
          "tree_id": "2f7f8f94c8382404df85ce87d61c8410484f6340",
          "url": "https://github.com/orcvs/orcvs/commit/e0e60ff2a50eee1521f05ea1785f5a39e1655b20"
        },
        "date": 1789528473604,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1951,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1956,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 8649,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1952,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 37106,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1952,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 162607,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1953,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 660426,
            "range": "± 1998",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1960,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 310,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 310,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1070,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4343,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 288,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 16566,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 64526,
            "range": "± 2316",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 288,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 5194,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 210,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 924,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 290,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 550,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1065,
            "range": "± 8",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 255,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1378,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5181,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20381,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 106317,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 468063,
            "range": "± 877",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2633925,
            "range": "± 4758",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7072,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28214,
            "range": "± 1378",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 115764,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7512,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28569,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 117442,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21842,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81385,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 331223,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1373278,
            "range": "± 3205",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14108,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 83479,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 361536,
            "range": "± 1539",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1540414,
            "range": "± 3417",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27698,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 114754,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 479036,
            "range": "± 1545",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2039979,
            "range": "± 86633",
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
          "id": "ce8b9483014e69d21985b08c2ecdcdd63aeebdda",
          "message": "Merge pull request #103 from orcvs/midi-selection-ownership\n\nDeepen MIDI selection ownership within Playback",
          "timestamp": "2026-09-16T06:50:41Z",
          "tree_id": "6ded6a5a67183aab61737e4d41692bfa46b139fe",
          "url": "https://github.com/orcvs/orcvs/commit/ce8b9483014e69d21985b08c2ecdcdd63aeebdda"
        },
        "date": 1789545069168,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1942,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1944,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 8631,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1924,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 37035,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1930,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 160997,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1929,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 642078,
            "range": "± 991",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1931,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 246,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 247,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 802,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 225,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 3278,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 223,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 12341,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 226,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 45172,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 221,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4099,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 208,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 934,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 290,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 550,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1003,
            "range": "± 16",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 252,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1369,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5103,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20869,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 107994,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 466381,
            "range": "± 999",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2645329,
            "range": "± 29497",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6562,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 26480,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 105003,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6732,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 26417,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 106048,
            "range": "± 3151",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21455,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79104,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 315554,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1294314,
            "range": "± 6726",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14229,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 81897,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 349468,
            "range": "± 2589",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1473334,
            "range": "± 9564",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 26927,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 110597,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 456349,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1938213,
            "range": "± 14612",
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
          "id": "71e41c4f6fc6f61e37f15536935a4cbdc6fa4b34",
          "message": "Merge pull request #104 from orcvs/egui-agent-workflow\n\nGive the console an egui skill, opt-in inspection, and kittest coverage",
          "timestamp": "2026-09-16T08:03:48Z",
          "tree_id": "4abd37857aacef485ce65ceed1e8d06efeb945f0",
          "url": "https://github.com/orcvs/orcvs/commit/71e41c4f6fc6f61e37f15536935a4cbdc6fa4b34"
        },
        "date": 1789548287888,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1853,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1836,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 7919,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1809,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 34219,
            "range": "± 811",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1829,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 149498,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1813,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 604681,
            "range": "± 1988",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1834,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 312,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 310,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1071,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4295,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 288,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 16503,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 64302,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4141,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 221,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 102,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 932,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 313,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 566,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1018,
            "range": "± 1",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 250,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1374,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5129,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20500,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 107555,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 466783,
            "range": "± 1232",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2745669,
            "range": "± 48100",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7300,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28487,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 118213,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7482,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28472,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 117744,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22009,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81922,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 332294,
            "range": "± 1528",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1394255,
            "range": "± 6402",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14588,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 85277,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 363273,
            "range": "± 1539",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1551647,
            "range": "± 9650",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 28342,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 116921,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 484010,
            "range": "± 1445",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2072051,
            "range": "± 19465",
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
          "id": "208c50aeb4fb04a1e2b76e789fa648e7d986ada9",
          "message": "Merge pull request #102 from orcvs/console-panel\n\nPut playback readouts and controls on a static console panel",
          "timestamp": "2026-09-17T00:59:13Z",
          "tree_id": "f61ffd0a5f2799d2d34c359cc34e59ff24971885",
          "url": "https://github.com/orcvs/orcvs/commit/208c50aeb4fb04a1e2b76e789fa648e7d986ada9"
        },
        "date": 1789607979429,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 905,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 896,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 3993,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 934,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 16549,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 909,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 71007,
            "range": "± 1992",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 880,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 275175,
            "range": "± 1331",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 929,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 174,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 167,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 543,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 153,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 2138,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 156,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 10082,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 155,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 36010,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 150,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 2634,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 120,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 47,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 500,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 40,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 150,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 301,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 543,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 149,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 542,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 3043,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 11344,
            "range": "± 743",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 56007,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 252466,
            "range": "± 4731",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 1380142,
            "range": "± 103842",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 3740,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 13920,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 58132,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 3886,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 14333,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 57923,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 11589,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 40470,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 161863,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 683497,
            "range": "± 1678",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 7577,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 44355,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 187324,
            "range": "± 3649",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 810830,
            "range": "± 5648",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 15299,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 59021,
            "range": "± 803",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 249492,
            "range": "± 5424",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 985616,
            "range": "± 18456",
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
          "id": "710ec9585d3fdc884fcbe97a509f5d41dafc56ea",
          "message": "Merge pull request #105 from orcvs/source-view\n\nPresent the Source as a bounded space at a stated Zoom",
          "timestamp": "2026-09-18T06:02:28Z",
          "tree_id": "9a6486f29f16dda5ab5c36a32142deea42f4cd44",
          "url": "https://github.com/orcvs/orcvs/commit/710ec9585d3fdc884fcbe97a509f5d41dafc56ea"
        },
        "date": 1789712179321,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 1817,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 1799,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 8078,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 1789,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 34647,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 1794,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 150915,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 1796,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 605127,
            "range": "± 953",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 1786,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 310,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 310,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1070,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 4333,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 16545,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 64480,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4935,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 207,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 923,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 295,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 557,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1056,
            "range": "± 1",
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
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 296,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1495,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5316,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20121,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 107193,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 464695,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2750067,
            "range": "± 58022",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6885,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 27044,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 110932,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7145,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 27321,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 110762,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21779,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 79311,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 322437,
            "range": "± 1180",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1300183,
            "range": "± 4467",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13877,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 82823,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 355738,
            "range": "± 2292",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1479067,
            "range": "± 7046",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27080,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 110955,
            "range": "± 453",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 457223,
            "range": "± 1324",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1962419,
            "range": "± 12883",
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
          "id": "6101ca94cc77e632b58b588ebb93f432b857f0f9",
          "message": "Merge pull request #108 from orcvs/sector-navigation\n\nStep the Cursor a Sector at a time with Tab, and keep every key from the Source while a popup is open",
          "timestamp": "2026-09-19T08:28:24Z",
          "tree_id": "de0456066df350e619ab92afdc31df224a08320e",
          "url": "https://github.com/orcvs/orcvs/commit/6101ca94cc77e632b58b588ebb93f432b857f0f9"
        },
        "date": 1789807359979,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2165,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2165,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 8993,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2178,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 37243,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 2182,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 155917,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 2165,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 629884,
            "range": "± 1291",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 2187,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 389,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 389,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1389,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 5588,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 21683,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 84705,
            "range": "± 1869",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 288,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4442,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 13071,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 211,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 919,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 296,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 564,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1060,
            "range": "± 7",
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
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 298,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1488,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 5118,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 20063,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 104089,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 463983,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2747548,
            "range": "± 10574",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6984,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28231,
            "range": "± 759",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 115661,
            "range": "± 2526",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7293,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28101,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 115851,
            "range": "± 2536",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21884,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81948,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 330692,
            "range": "± 14181",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1399966,
            "range": "± 67878",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14256,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 84414,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 361675,
            "range": "± 1058",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1549996,
            "range": "± 3311",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27980,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 115666,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 476878,
            "range": "± 2251",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2061377,
            "range": "± 6123",
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
          "id": "22ca27184a1e4a9012ce734e3a0517f63c2e197c",
          "message": "Merge pull request #109 from orcvs/syntax-highlighting\n\nSource Paint and a Function reference",
          "timestamp": "2026-09-19T10:27:19Z",
          "tree_id": "86d77ac616f1a4fe8c29b11b8f99e9ccfab2c62c",
          "url": "https://github.com/orcvs/orcvs/commit/22ca27184a1e4a9012ce734e3a0517f63c2e197c"
        },
        "date": 1789814518054,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 12224,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 12506,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 50140,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 12500,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 197261,
            "range": "± 976",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 12328,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 790456,
            "range": "± 4241",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 11960,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 3232981,
            "range": "± 13251",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 11931,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 782,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 751,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2511,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 827,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 8787,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 749,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 32555,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 723,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 125777,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 806,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4448,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 13037,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 208,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 931,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 289,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 549,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1004,
            "range": "± 1",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 295,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1494,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 13780,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 54762,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 233092,
            "range": "± 2960",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 933367,
            "range": "± 11173",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 3769253,
            "range": "± 45346",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7450,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28532,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 117084,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7664,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28650,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 117712,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22476,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 85750,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 333597,
            "range": "± 2752",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1371473,
            "range": "± 2230",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14330,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 84359,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 365453,
            "range": "± 2895",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1542374,
            "range": "± 7307",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27672,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 115884,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 477402,
            "range": "± 2607",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2031987,
            "range": "± 4372",
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
          "id": "88b75f2295eb5c88f1b465804738db1101249d32",
          "message": "Merge pull request #106 from orcvs/source-view-margin\n\nDouble the default Grid, add a two-Cell margin, and show a grab pointer while Panning",
          "timestamp": "2026-09-20T00:47:41Z",
          "tree_id": "cb9212514850714ac9e5114c304daa8141e0e369",
          "url": "https://github.com/orcvs/orcvs/commit/88b75f2295eb5c88f1b465804738db1101249d32"
        },
        "date": 1789866160269,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 11474,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 11384,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 46769,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 11444,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 184913,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 11457,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 765413,
            "range": "± 2243",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 11192,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 3169625,
            "range": "± 13206",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 11237,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 622,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 593,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2179,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 634,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 7981,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 585,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 30226,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 560,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 154652,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 619,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4120,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 14915,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 239,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 117,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 874,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 56,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 104,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 344,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 590,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1156,
            "range": "± 1",
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
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 144,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1361,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 17056,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 62688,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 258881,
            "range": "± 1680",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 1104020,
            "range": "± 1459",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 5160201,
            "range": "± 121944",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6605,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24580,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 128464,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6904,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 24785,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 127701,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 20538,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 74420,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 330108,
            "range": "± 884",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1387926,
            "range": "± 7242",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 13150,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 76518,
            "range": "± 2753",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 368218,
            "range": "± 7394",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1566845,
            "range": "± 10556",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 26075,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 107192,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 481844,
            "range": "± 1150",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2050940,
            "range": "± 25976",
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
          "id": "77b57659adbd5b0109a7dfe33d3dd148d717776e",
          "message": "Merge pull request #110 from orcvs/untracked-cleanup\n\nCommit the untracked working files that are repository state, and ignore the ones that are not",
          "timestamp": "2026-09-20T01:23:54Z",
          "tree_id": "0949301900ab868c2641fb0319abc89e948df446",
          "url": "https://github.com/orcvs/orcvs/commit/77b57659adbd5b0109a7dfe33d3dd148d717776e"
        },
        "date": 1789868282848,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 11847,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 11678,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 47160,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 11693,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 192750,
            "range": "± 1159",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 11908,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 778386,
            "range": "± 4244",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 11944,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 3196366,
            "range": "± 73310",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 12066,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 725,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 679,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2402,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 722,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 8584,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 682,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 31973,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 664,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 121417,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 744,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4720,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 13528,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 207,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 954,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 296,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 555,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1064,
            "range": "± 2",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 252,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1373,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 13684,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 55073,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 228161,
            "range": "± 5090",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 896156,
            "range": "± 12713",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 3980487,
            "range": "± 121486",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6694,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 27573,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 110885,
            "range": "± 924",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7091,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 27540,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 110777,
            "range": "± 920",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22181,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81446,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 324845,
            "range": "± 969",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1328828,
            "range": "± 7029",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14104,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 84101,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 359771,
            "range": "± 2310",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1508389,
            "range": "± 20014",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27713,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 113092,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 471817,
            "range": "± 5088",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1995049,
            "range": "± 32591",
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
          "id": "c6b25c0ff7d1ab051c8a096fd291a15e6ff04fa2",
          "message": "Merge pull request #111 from orcvs/syntax-highlighting-07-follow-ups\n\nClose the syntax-highlighting 01-04 review follow-ups, and split theming out of the console restyle",
          "timestamp": "2026-09-20T03:21:59Z",
          "tree_id": "60c1aa23e665d682da24c204dab0daa307b38668",
          "url": "https://github.com/orcvs/orcvs/commit/c6b25c0ff7d1ab051c8a096fd291a15e6ff04fa2"
        },
        "date": 1789875358480,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 8878,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 8853,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 36194,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 8918,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 148380,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 8972,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 607405,
            "range": "± 1897",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 9048,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 2514554,
            "range": "± 5430",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 9058,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 498,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 473,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1703,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 498,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 6080,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 459,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 22555,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 441,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 88219,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 505,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 3515,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 10895,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 175,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 778,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 67,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 111,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 246,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 455,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 847,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 42,
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
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 215,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1188,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 10824,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 41055,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 170546,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 684428,
            "range": "± 1283",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 2980187,
            "range": "± 64799",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 6312,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 24104,
            "range": "± 1149",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 99214,
            "range": "± 3395",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6601,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 23842,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 99021,
            "range": "± 3268",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 17635,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 61790,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 257359,
            "range": "± 3056",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1067016,
            "range": "± 5988",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 11687,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 66233,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 287977,
            "range": "± 2666",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1210394,
            "range": "± 9022",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 22363,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 91180,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 379851,
            "range": "± 4853",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1603529,
            "range": "± 11649",
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
          "id": "ff4844df8b435b9242a7a80f6646776a0924794d",
          "message": "Merge pull request #112 from orcvs/parser-borrow-and-tracing-cleanup\n\nAccept immutable Source text, and keep test-only tracing out of shipped graphs",
          "timestamp": "2026-09-20T04:26:04Z",
          "tree_id": "938675cd642252032d60eb73e22eab7882bc9036",
          "url": "https://github.com/orcvs/orcvs/commit/ff4844df8b435b9242a7a80f6646776a0924794d"
        },
        "date": 1789879228731,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 9195,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 8874,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 37172,
            "range": "± 2200",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 9216,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 152302,
            "range": "± 3599",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 9155,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 637131,
            "range": "± 32939",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 9565,
            "range": "± 454",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 2579881,
            "range": "± 117727",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 9182,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 515,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 473,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 1740,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 578,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 6474,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 496,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 24970,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 451,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 146910,
            "range": "± 3535",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 522,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 3408,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 11718,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 149,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 66,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 68,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 642,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 91,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 205,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 404,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 718,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/16x16",
            "value": 40,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/32x32",
            "value": 40,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 124,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1622,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 17078,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 64547,
            "range": "± 2106",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 272885,
            "range": "± 12273",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 1129681,
            "range": "± 36349",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 4358289,
            "range": "± 169285",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 5877,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 20504,
            "range": "± 821",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 96204,
            "range": "± 3244",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 6096,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 20664,
            "range": "± 906",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 94623,
            "range": "± 3395",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 17137,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 61214,
            "range": "± 3183",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 280786,
            "range": "± 12633",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1179266,
            "range": "± 37998",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 10907,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 65003,
            "range": "± 2441",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 320597,
            "range": "± 13340",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1333347,
            "range": "± 18605",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 20338,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 87156,
            "range": "± 4026",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 393036,
            "range": "± 12089",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 1670254,
            "range": "± 85052",
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
          "id": "3650c7d8c463cf1e48fa7d37c8e84b408ad234be",
          "message": "Merge pull request #113 from orcvs/syntax-highlighting-12-fitted-portal\n\nFit Sequence Output Portal highlights to their answers",
          "timestamp": "2026-09-20T09:50:37Z",
          "tree_id": "ec1add6169d12ac42de0758f85254e57e62bec47",
          "url": "https://github.com/orcvs/orcvs/commit/3650c7d8c463cf1e48fa7d37c8e84b408ad234be"
        },
        "date": 1789898694883,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 11794,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 11702,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 47208,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 11740,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 192336,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 11750,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 785236,
            "range": "± 45672",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 11864,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 3205954,
            "range": "± 6152",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 11912,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 691,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 697,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2072,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 661,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 7141,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 624,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 27718,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 665,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 120255,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 660,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 3634,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 12451,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 207,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 922,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 298,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 595,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1056,
            "range": "± 2",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 249,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1373,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 14410,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 56893,
            "range": "± 4310",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 239643,
            "range": "± 2543",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 922920,
            "range": "± 9006",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 3782733,
            "range": "± 141749",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7103,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28462,
            "range": "± 2400",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 115234,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7414,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28630,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 115779,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21980,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81645,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 326845,
            "range": "± 1658",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1374008,
            "range": "± 2434",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14305,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 84293,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 360903,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1532063,
            "range": "± 5447",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 27839,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 114481,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 474931,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2007097,
            "range": "± 5472",
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
          "id": "ea9a75067476b7fcdaa2a0e8d39d45f57622639a",
          "message": "Merge pull request #116 from orcvs/dependabot/github_actions/github-actions-abd2b60a87\n\nBump benchmark-action/github-action-benchmark from 1.22.1 to 1.22.2 in the github-actions group",
          "timestamp": "2026-09-21T00:15:17Z",
          "tree_id": "5881a4bbc5ecab1b901af8bbf0c20f4ce0336951",
          "url": "https://github.com/orcvs/orcvs/commit/ea9a75067476b7fcdaa2a0e8d39d45f57622639a"
        },
        "date": 1789950559208,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 11922,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 12009,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 47696,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 12030,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 193769,
            "range": "± 1236",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 12060,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 780534,
            "range": "± 1146",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 11845,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 3253265,
            "range": "± 28136",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 12007,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 696,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 689,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2102,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 666,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 7178,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 625,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 27913,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 665,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 126810,
            "range": "± 602",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 660,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 3934,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 13314,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 210,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 920,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 301,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 571,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1041,
            "range": "± 2",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 255,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1370,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 14511,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 56348,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 236930,
            "range": "± 2882",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 928287,
            "range": "± 10005",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 3914490,
            "range": "± 103962",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7057,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28141,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 117592,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7417,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28605,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 117200,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22445,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 84101,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 334461,
            "range": "± 1795",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1422526,
            "range": "± 27661",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14564,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 85718,
            "range": "± 1478",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 368547,
            "range": "± 1667",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1564216,
            "range": "± 6487",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 28535,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 118119,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 489454,
            "range": "± 1238",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2087346,
            "range": "± 8265",
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
          "id": "73b400e1ab597402d61b2bfc5115130467e5eec4",
          "message": "Merge pull request #117 from orcvs/dependabot/cargo/rust-dependencies-81b68b975f\n\nBump egui from 0.36.1 to 0.36.2 in the rust-dependencies group",
          "timestamp": "2026-09-21T01:38:13Z",
          "tree_id": "fa6caefd79d5fe238f41bdf0dd2c09038ed7e708",
          "url": "https://github.com/orcvs/orcvs/commit/73b400e1ab597402d61b2bfc5115130467e5eec4"
        },
        "date": 1789956983723,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 12562,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 12367,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 49358,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 12287,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 206236,
            "range": "± 2719",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 12304,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 823207,
            "range": "± 5642",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 12225,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 3374295,
            "range": "± 34952",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 12278,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 755,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 754,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2207,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 765,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 7448,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 701,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 29166,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 744,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 131829,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 766,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 5010,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 13999,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 211,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 922,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 317,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 582,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1049,
            "range": "± 2",
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
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/64x64",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 297,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1491,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 14355,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 56411,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 237488,
            "range": "± 2909",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 959700,
            "range": "± 13143",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 3856680,
            "range": "± 51409",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7147,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28486,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 117869,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7321,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28678,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 118444,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 22055,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 81930,
            "range": "± 1526",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 332915,
            "range": "± 2497",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1389298,
            "range": "± 3908",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14343,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 84913,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 363507,
            "range": "± 1802",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1545055,
            "range": "± 2795",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 28018,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 115902,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 478764,
            "range": "± 1120",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2046521,
            "range": "± 20106",
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
          "id": "1dccf489b2b8d817054c71b05393cfef645170c3",
          "message": "Merge pull request #115 from orcvs/paint-cell-cost-02-source-paint-facts\n\nAnswer Source Paint facts per Cell",
          "timestamp": "2026-09-21T02:57:24Z",
          "tree_id": "a71aa78945a7b0de773b01b49431d3c694e1dff4",
          "url": "https://github.com/orcvs/orcvs/commit/1dccf489b2b8d817054c71b05393cfef645170c3"
        },
        "date": 1789960269732,
        "tool": "cargo",
        "benches": [
          {
            "name": "paint_derive/fitted/16x16",
            "value": 2964,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/16x16",
            "value": 2963,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/32x32",
            "value": 13119,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/32x32",
            "value": 2941,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/64x64",
            "value": 58069,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/64x64",
            "value": 3039,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/128x128",
            "value": 253808,
            "range": "± 3372",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/128x128",
            "value": 3135,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/fitted/256x256",
            "value": 1099540,
            "range": "± 4092",
            "unit": "ns/iter"
          },
          {
            "name": "paint_derive/culled/256x256",
            "value": 3091,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/16x16",
            "value": 680,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/16x16",
            "value": 676,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/32x32",
            "value": 2124,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/32x32",
            "value": 651,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/64x64",
            "value": 7104,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/64x64",
            "value": 617,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/128x128",
            "value": 27384,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/128x128",
            "value": 666,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/fitted/256x256",
            "value": 118166,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "paint_background_runs/culled/256x256",
            "value": 658,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/frame and living area",
            "value": 4217,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "cursor effects/region lasso 20x12",
            "value": 13527,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "parse",
            "value": 212,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_invalid",
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "execute",
            "value": 114,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_source",
            "value": 929,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/3",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/7",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/15",
            "value": 297,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/31",
            "value": 568,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "parse_records/63",
            "value": 1045,
            "range": "± 2",
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
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/128x128",
            "value": 248,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "source_read_revision/256x256",
            "value": 1374,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/16x16",
            "value": 13948,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/32x32",
            "value": 54198,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/64x64",
            "value": 229349,
            "range": "± 2891",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/128x128",
            "value": 899660,
            "range": "± 11722",
            "unit": "ns/iter"
          },
          {
            "name": "source_render_frame/256x256",
            "value": 4591857,
            "range": "± 142244",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/16x16",
            "value": 7050,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/32x32",
            "value": 28184,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_valid/64x64",
            "value": 116234,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/16x16",
            "value": 7276,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/32x32",
            "value": 28381,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "source_edit_rebuild_invalid/64x64",
            "value": 114681,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/16x16",
            "value": 21875,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/32x32",
            "value": 82039,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/64x64",
            "value": 336505,
            "range": "± 1148",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick/128x128",
            "value": 1383749,
            "range": "± 15039",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/16x16",
            "value": 14225,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/32x32",
            "value": 84695,
            "range": "± 670",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/64x64",
            "value": 367184,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_edges/128x128",
            "value": 1547415,
            "range": "± 3310",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/16x16",
            "value": 28429,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/32x32",
            "value": 116122,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/64x64",
            "value": 477902,
            "range": "± 1389",
            "unit": "ns/iter"
          },
          {
            "name": "source_execute_tick_portal_inputs/128x128",
            "value": 2040446,
            "range": "± 8029",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "memory": [
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
          "id": "f25bd092360d8b8db747ef388ea915189c9ddd84",
          "message": "Merge pull request #31 from orcvs/08-memory-verification\n\nVerify how much memory Orcvs uses",
          "timestamp": "2026-09-09T13:10:09+10:00",
          "tree_id": "c6b6956278a0c664c201ef9aba72edfd647f78d7",
          "url": "https://github.com/orcvs/orcvs/commit/f25bd092360d8b8db747ef388ea915189c9ddd84"
        },
        "date": 1788924797781,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "da0f87b4637646f9d285e07b4510cdff7acc0d57",
          "message": "Merge pull request #36 from orcvs/09-widen-function-kind\n\nWiden the Function kind to value or effect",
          "timestamp": "2026-09-09T13:43:17+10:00",
          "tree_id": "2c685c512e091af9f4dca0a8c3a6d924b3438e0e",
          "url": "https://github.com/orcvs/orcvs/commit/da0f87b4637646f9d285e07b4510cdff7acc0d57"
        },
        "date": 1788926093563,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dc38019bb6151cbace510774e1a75a281009a9d6",
          "message": "Merge pull request #32 from orcvs/12-safety-action-reset\n\nClear controllers and the bend in the safety action",
          "timestamp": "2026-09-09T14:12:50+10:00",
          "tree_id": "a79299f57aa06cd891c7c9ee66b45f8cd111bae2",
          "url": "https://github.com/orcvs/orcvs/commit/dc38019bb6151cbace510774e1a75a281009a9d6"
        },
        "date": 1788927893422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "74d2aa3cf31f14ca30c46dd9234cfc0c0f42fca5",
          "message": "Merge pull request #37 from orcvs/10-native-midi-seam\n\nName the native MIDI decision once",
          "timestamp": "2026-09-09T14:45:29+10:00",
          "tree_id": "c57b150fe12d4ef1329c560fda822fe1dfc3bea6",
          "url": "https://github.com/orcvs/orcvs/commit/74d2aa3cf31f14ca30c46dd9234cfc0c0f42fca5"
        },
        "date": 1788929922186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2404c2afe348f0ec26d2572df1da2269a5555acd",
          "message": "Merge pull request #35 from orcvs/deepen-portal-and-evaluator-seams\n\nDeepen Portal relationships and test through the Evaluator",
          "timestamp": "2026-09-09T15:25:34+10:00",
          "tree_id": "e1ac1c2c0894d7da324e4ecd3bfdb302da754693",
          "url": "https://github.com/orcvs/orcvs/commit/2404c2afe348f0ec26d2572df1da2269a5555acd"
        },
        "date": 1788932283757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "51b0fc5fb6346ec1505333adaf078ef036f040e7",
          "message": "Merge pull request #43 from orcvs/verify-02-add-source-bang-activation-and-expiry\n\nTest the non-root Bang contact only prose claimed",
          "timestamp": "2026-09-09T16:43:29+10:00",
          "tree_id": "5b064cb1022a1b05a859be45fe1b38b360e5d3de",
          "url": "https://github.com/orcvs/orcvs/commit/51b0fc5fb6346ec1505333adaf078ef036f040e7"
        },
        "date": 1788936953294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b332ac7e906b709e129078f8de8c357c2066079c",
          "message": "Merge pull request #40 from orcvs/02-grid-position-round-trip\n\nEncode the Grid laws the glossary already states",
          "timestamp": "2026-09-09T16:59:25+10:00",
          "tree_id": "f80f23c09cdbf42f8310ca0ff31a3cf0fd4978aa",
          "url": "https://github.com/orcvs/orcvs/commit/b332ac7e906b709e129078f8de8c357c2066079c"
        },
        "date": 1788937920721,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "468497d8fd57d84db978f378f2ea7eb43665a7ad",
          "message": "Merge pull request #41 from orcvs/bind-tick-lookup-ownership\n\nBind Tick Lookup to the computations it indexes",
          "timestamp": "2026-09-09T17:39:27+10:00",
          "tree_id": "183acd4c0e999290d882949507f6f8fa66ee3843",
          "url": "https://github.com/orcvs/orcvs/commit/468497d8fd57d84db978f378f2ea7eb43665a7ad"
        },
        "date": 1788940297988,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "53230041992a7aa5156356e949b335dae2b8b1bd",
          "message": "Merge pull request #38 from orcvs/11-persist-source\n\nRestore the Source revision through eframe storage",
          "timestamp": "2026-09-09T18:23:21+10:00",
          "tree_id": "9abd15d8d737a1434d39f75a6b0465df61d82ed2",
          "url": "https://github.com/orcvs/orcvs/commit/53230041992a7aa5156356e949b335dae2b8b1bd"
        },
        "date": 1788943654517,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1c1bcb6cb37484b2b0ff5d6fff182f4ce5842e63",
          "message": "Merge pull request #44 from orcvs/deepen-tick-execution\n\nOwn Tick-local state transitions behind execution seam",
          "timestamp": "2026-09-09T19:05:36+10:00",
          "tree_id": "f932e73578204104624e75b0d73b4953fc5e610f",
          "url": "https://github.com/orcvs/orcvs/commit/1c1bcb6cb37484b2b0ff5d6fff182f4ce5842e63"
        },
        "date": 1788945401013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "80d39c3951483eafd4da2616c05bca81221ba0ab",
          "message": "Merge pull request #42 from orcvs/02-put-midir-behind-a-native-midi-feature\n\nPut midir behind a native-midi feature",
          "timestamp": "2026-09-09T20:14:58+10:00",
          "tree_id": "a7928df46f9bb12b9227314cfb4c317248f45341",
          "url": "https://github.com/orcvs/orcvs/commit/80d39c3951483eafd4da2616c05bca81221ba0ab"
        },
        "date": 1788950218850,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2c564cab550e0dffb9848d9fda0139ffecf7b9b6",
          "message": "Merge pull request #45 from orcvs/08-bring-the-benchmark-workflow-inside-its-budget\n\nBring the benchmark run inside the budget its thresholds can use",
          "timestamp": "2026-09-09T21:13:24+10:00",
          "tree_id": "01d5bd1dfc8dd32f12107ae215087310e09a7746",
          "url": "https://github.com/orcvs/orcvs/commit/2c564cab550e0dffb9848d9fda0139ffecf7b9b6"
        },
        "date": 1788953227996,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "5de3f349e6b8cba3ec0a26a2d70f162fc5680cf8",
          "message": "Merge pull request #48 from orcvs/05-exhaustive-arithmetic-and-note-conversion\n\nProve the byte-arithmetic and Note-conversion laws exhaustively",
          "timestamp": "2026-09-09T23:12:33+10:00",
          "tree_id": "a078d90db023dcd5b8538633b5ee1ce2aae114bf",
          "url": "https://github.com/orcvs/orcvs/commit/5de3f349e6b8cba3ec0a26a2d70f162fc5680cf8"
        },
        "date": 1788959712553,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "ec2258ae1cde158af10f76325f5dd60e72b6e0d4",
          "message": "Merge pull request #47 from orcvs/04-prove-the-expression-span-law\n\nProve the Expression Span law as a property",
          "timestamp": "2026-09-09T23:17:03+10:00",
          "tree_id": "f3313cfabb5619ff73af76a9424fd9f5583bcec1",
          "url": "https://github.com/orcvs/orcvs/commit/ec2258ae1cde158af10f76325f5dd60e72b6e0d4"
        },
        "date": 1788959982073,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 6117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 79,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 47971,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 9061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 247,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 173913,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 16,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 16485,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 928,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 658728,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "80c2eb69a9d773025cb057a52fa3df949e715f36",
          "message": "Merge pull request #50 from orcvs/07-make-the-comment-a-parser-unit\n\nSpell the Comment as a Parser unit",
          "timestamp": "2026-09-10T13:26:21+10:00",
          "tree_id": "4c0fabf2e5c279041a7ead4b7966a41f3e530024",
          "url": "https://github.com/orcvs/orcvs/commit/80c2eb69a9d773025cb057a52fa3df949e715f36"
        },
        "date": 1789010940091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "d3bdda8a671826e492e8bfdcf95ec2cb3e7373df",
          "message": "Merge pull request #49 from orcvs/06-project-a-sequence-result-into-source\n\nProject a Sequence result into Source",
          "timestamp": "2026-09-10T13:34:02+10:00",
          "tree_id": "978c93d19fbc6caa2a2609fd239222b390b9fd35",
          "url": "https://github.com/orcvs/orcvs/commit/d3bdda8a671826e492e8bfdcf95ec2cb3e7373df"
        },
        "date": 1789011397125,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "03ecea63b816e9963b820a385bdf0d49473b6233",
          "message": "Merge pull request #51 from orcvs/02-add-clock-delay-and-euclidean\n\nAdd the Clock, Delay, and Euclidean Functions",
          "timestamp": "2026-09-10T14:18:06+10:00",
          "tree_id": "16edd31a9d99af7ad26c1f52721abade99488b19",
          "url": "https://github.com/orcvs/orcvs/commit/03ecea63b816e9963b820a385bdf0d49473b6233"
        },
        "date": 1789014114218,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "b1d67b7bbb75513def5d7110842c74be30b9f8cd",
          "message": "Merge pull request #52 from orcvs/01-give-a-late-tick-one-policy\n\nGive a late Tick one policy",
          "timestamp": "2026-09-10T14:54:47+10:00",
          "tree_id": "4809dbb66221c29a0e9bb358420b574471186a6a",
          "url": "https://github.com/orcvs/orcvs/commit/b1d67b7bbb75513def5d7110842c74be30b9f8cd"
        },
        "date": 1789016244743,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c8669dcc41225952595813ba78abcdd0255e166",
          "message": "Merge pull request #53 from orcvs/01-re-site-the-injected-value-tests\n\nMove the injected-value tests below the planning entry point, and delete the seam",
          "timestamp": "2026-09-10T16:28:16+10:00",
          "tree_id": "14babad2e753889e678d375cfabe051be8f52653",
          "url": "https://github.com/orcvs/orcvs/commit/7c8669dcc41225952595813ba78abcdd0255e166"
        },
        "date": 1789021853816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3c4424d18ef95eab30ff5e8f034b817e15453aa9",
          "message": "Merge pull request #54 from orcvs/04-build-a-destination-carrying-schedule\n\nBuild a destination-carrying schedule a test can construct",
          "timestamp": "2026-09-10T17:48:55+10:00",
          "tree_id": "d821c532b4642e0e01b6a3a98f744c7b29167c91",
          "url": "https://github.com/orcvs/orcvs/commit/3c4424d18ef95eab30ff5e8f034b817e15453aa9"
        },
        "date": 1789026690048,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "3406af1b6653b0ddd19cf911ebf74a00c830c0e2",
          "message": "Merge pull request #57 from orcvs/09-give-a-computations-reservation-one-home\n\nGive a computation's reservation one home",
          "timestamp": "2026-09-10T19:49:37+10:00",
          "tree_id": "81a135e5e429c929edbd3228c69f15e462c15878",
          "url": "https://github.com/orcvs/orcvs/commit/3406af1b6653b0ddd19cf911ebf74a00c830c0e2"
        },
        "date": 1789033931019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b3937425a075deb58a8215560e26e33c5cde087a",
          "message": "Merge pull request #55 from orcvs/05-migrate-the-ordering-and-dependency-destination-tests\n\nMigrate the ordering and dependency destination tests",
          "timestamp": "2026-09-10T20:52:02+10:00",
          "tree_id": "d85ffec4661d85b5ea341384b988efc82ad22e70",
          "url": "https://github.com/orcvs/orcvs/commit/b3937425a075deb58a8215560e26e33c5cde087a"
        },
        "date": 1789037662326,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0a3468510e3804446bb3e9dd39034e0335990446",
          "message": "Merge pull request #56 from orcvs/06-migrate-the-remaining-destination-tests\n\nMigrate the remaining destination tests, and say what a Tick evaluated",
          "timestamp": "2026-09-10T22:37:56+10:00",
          "tree_id": "c92b85e9d97397652caf5686c015deac8dada5e3",
          "url": "https://github.com/orcvs/orcvs/commit/0a3468510e3804446bb3e9dd39034e0335990446"
        },
        "date": 1789044042080,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "68e5eba38ed97428bc53bc18d510f9b6df926a2b",
          "message": "Merge pull request #58 from orcvs/08-delete-the-configuration-and-its-parameter\n\nDelete the scheduler configuration and the parameter that carried it",
          "timestamp": "2026-09-10T22:59:23+10:00",
          "tree_id": "7e2c58e3c239e13e53aca9483266e65b83feaebf",
          "url": "https://github.com/orcvs/orcvs/commit/68e5eba38ed97428bc53bc18d510f9b6df926a2b"
        },
        "date": 1789045336039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "e6a53d85ef1b48305b0795030bce1598888ba1a5",
          "message": "Merge pull request #61 from orcvs/01-rename-the-shell-crate-to-console\n\nRename the shell crate to console",
          "timestamp": "2026-09-11T12:00:38+10:00",
          "tree_id": "43260d1bee18eec4b6c9009c7ffc60a649ff17d3",
          "url": "https://github.com/orcvs/orcvs/commit/e6a53d85ef1b48305b0795030bce1598888ba1a5"
        },
        "date": 1789092679303,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "2f76622d5652c1a7ca1f94d97972c54b0d4f976e",
          "message": "Merge pull request #60 from orcvs/02-concentrate-clock-loop\n\nConcentrate the Playback clock loop",
          "timestamp": "2026-09-11T13:17:26+10:00",
          "tree_id": "78025ff01e30cb007a769e035417e194a97decbf",
          "url": "https://github.com/orcvs/orcvs/commit/2f76622d5652c1a7ca1f94d97972c54b0d4f976e"
        },
        "date": 1789096800825,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "84f03be5d44533bab7076d28dd98d578e3da9fc7",
          "message": "Merge pull request #62 from orcvs/03-migrate-self-banging-functions-to-the-function-table\n\nAdd the Self-Banging Functions `^^ vv << >>`",
          "timestamp": "2026-09-11T14:12:24+10:00",
          "tree_id": "2804758db841910065960c6966dc0d2de9550136",
          "url": "https://github.com/orcvs/orcvs/commit/84f03be5d44533bab7076d28dd98d578e3da9fc7"
        },
        "date": 1789100119832,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "42fe2bd62ec68dcc6ee3062c1761ac63643fccb0",
          "message": "Merge pull request #63 from orcvs/06-add-the-directional-bang-functions\n\nAdd the Directional Bang Functions `*^ *v *< *>`",
          "timestamp": "2026-09-11T14:35:06+10:00",
          "tree_id": "6165ccbb6e8bb291db2c634f4abcb7a5da39c0a6",
          "url": "https://github.com/orcvs/orcvs/commit/42fe2bd62ec68dcc6ee3062c1761ac63643fccb0"
        },
        "date": 1789101446693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e3c18ad48f91b666887a3bd0a680631ed1a2ab92",
          "message": "Merge pull request #68 from orcvs/04-coalesce-cell-backgrounds\n\nCoalesce Cell backgrounds, and hold the fill they stand on",
          "timestamp": "2026-09-11T15:33:02+10:00",
          "tree_id": "791600ce3e569b52b4e2c89b1ee9ee0c08756949",
          "url": "https://github.com/orcvs/orcvs/commit/e3c18ad48f91b666887a3bd0a680631ed1a2ab92"
        },
        "date": 1789104946023,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2ffdd4c04538a5fa413b06f75fe6ee040abd84ca",
          "message": "Merge pull request #71 from orcvs/function-replacement\n\nName which fact a refused Function replacement changes",
          "timestamp": "2026-09-11T17:34:13+10:00",
          "tree_id": "d6cf1b6ce5cae228f0028afa6c4e8048a24a9e61",
          "url": "https://github.com/orcvs/orcvs/commit/2ffdd4c04538a5fa413b06f75fe6ee040abd84ca"
        },
        "date": 1789112210384,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "38b2eb364477ddbead8d56280f6508a9aa7ba2bf",
          "message": "Merge pull request #73 from orcvs/fix-the-output-failure-latch\n\nScope the output failure latch to the run that recorded it",
          "timestamp": "2026-09-11T18:22:34+10:00",
          "tree_id": "62724ef942c7e47cb4d9f1562653e785c88d2fee",
          "url": "https://github.com/orcvs/orcvs/commit/38b2eb364477ddbead8d56280f6508a9aa7ba2bf"
        },
        "date": 1789115119008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "d2d2ac60c6d8c7a29b0be75d0bf2cef034d0afb9",
          "message": "Point the Self-Banging advice at the method that exists\n\ntakes_no_operand's doc linked Function::source_write, which no crate declares.\nIt is the only rustdoc error in the workspace and it fails the merge tier's\ngate under RUSTDOCFLAGS=\"-D warnings\", so full-gate has been red on main rather\nthan on any one branch — PR #71 and PR #73 each merged and each inherited it.\n\nFunction::source_effect is the method meant, but naming it was not the whole\nfix. Both groups declare an effect, so a caller told to ask source_effect and\nnothing more would have been given a second untrue sentence in place of the\nfirst. The effect table decides it: every Self-Banging arm declares\nSourceBundle::Advance and every Directional Bang arm SourceBundle::Emit, so the\nbundle is what tells them apart and the sentence now says to read it.\n\nPushed straight to main. The gate is one of the three required status checks and\nwas failing before this change on every branch equally, so a pull request of its\nown could not have turned it green any sooner.\n\nCloses .scratch/lang-foundations/issues/09.\n\nClaude-Session: https://claude.ai/code/session_017SYs8gt3hywhcyxxM2L13j",
          "timestamp": "2026-09-11T18:44:28+10:00",
          "tree_id": "72243a435d8340110470399cce1a23f0f1174d41",
          "url": "https://github.com/orcvs/orcvs/commit/d2d2ac60c6d8c7a29b0be75d0bf2cef034d0afb9"
        },
        "date": 1789116451196,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "7425a1a7b1eac79f04e13668e9660c3f980df67f",
          "message": "Merge pull request #74 from orcvs/renumber-the-pulse-adr\n\nGive the pulse decision a number of its own",
          "timestamp": "2026-09-11T09:11:53Z",
          "tree_id": "5699c61c0efb0ac74289b0568d880c78393e9ddf",
          "url": "https://github.com/orcvs/orcvs/commit/7425a1a7b1eac79f04e13668e9660c3f980df67f"
        },
        "date": 1789118315688,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "f81962c14b253edc6760e381b7c6fc3a81437253",
          "message": "Merge pull request #72 from orcvs/function-replacement-04-05\n\nGenerate the replacement facts, and prove the width term where the widths live",
          "timestamp": "2026-09-11T09:52:23Z",
          "tree_id": "0c9ffe8a065b10faa5e958cf35f42a4cd7cc3414",
          "url": "https://github.com/orcvs/orcvs/commit/f81962c14b253edc6760e381b7c6fc3a81437253"
        },
        "date": 1789120740478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "2d2674916e679e4503fe6842353b009b62b4072e",
          "message": "Merge pull request #75 from orcvs/observable-schedule\n\nSay which Turn a computation took",
          "timestamp": "2026-09-12T02:49:03Z",
          "tree_id": "6dbe0cd00cc5a196d7a211f6385dc758cc76911f",
          "url": "https://github.com/orcvs/orcvs/commit/2d2674916e679e4503fe6842353b009b62b4072e"
        },
        "date": 1789182968979,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "e32c1ba4b0771e26c26fba1c4d097a989d772cab",
          "message": "Merge pull request #78 from orcvs/stop-running-the-benchmarks-as-tests\n\nStop running the benchmarks as tests in the merge tier",
          "timestamp": "2026-09-12T05:05:07Z",
          "tree_id": "ad7cb054bef8cba6cee24dfd8d7b0b77f906e568",
          "url": "https://github.com/orcvs/orcvs/commit/e32c1ba4b0771e26c26fba1c4d097a989d772cab"
        },
        "date": 1789191005175,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "cda06323d391a8c861f4a234a44e0a80642d6f44",
          "message": "Merge pull request #69 from orcvs/01-measure-what-a-render-frame-costs\n\nMeasure a Render Frame at the sizes a resizable Grid would reach",
          "timestamp": "2026-09-12T05:54:43Z",
          "tree_id": "527f3e34e5025d6efd09616996fa11a3bcd1695d",
          "url": "https://github.com/orcvs/orcvs/commit/cda06323d391a8c861f4a234a44e0a80642d6f44"
        },
        "date": 1789193087258,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "8b9491bbeba82b12ec14d938dd912afa1dc1f45a",
          "message": "Merge pull request #80 from orcvs/cull-source-paint\n\nLet the Source Grid paint answer instead of take",
          "timestamp": "2026-09-12T10:57:22Z",
          "tree_id": "629d8690ec93288a17706c92be541cc7b0dd39aa",
          "url": "https://github.com/orcvs/orcvs/commit/8b9491bbeba82b12ec14d938dd912afa1dc1f45a"
        },
        "date": 1789211193296,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "bd9ee4bb32af4d1fab30d060179d639056e1c6d5",
          "message": "Merge pull request #79 from orcvs/playback-actor\n\nGive the Playback Engine's state an owner",
          "timestamp": "2026-09-12T11:43:00Z",
          "tree_id": "926296a9322e46959cda572195c3fce3b5b5018a",
          "url": "https://github.com/orcvs/orcvs/commit/bd9ee4bb32af4d1fab30d060179d639056e1c6d5"
        },
        "date": 1789214024990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "f67e04aba5948980581b83af4953b7edf528dc6e",
          "message": "Merge pull request #81 from orcvs/triage/render-frame-responsibility\n\nLand the resolved render-frame-responsibility tickets",
          "timestamp": "2026-09-13T00:29:27Z",
          "tree_id": "5ef412632fc9b47f366ecb78df7487355427a33a",
          "url": "https://github.com/orcvs/orcvs/commit/f67e04aba5948980581b83af4953b7edf528dc6e"
        },
        "date": 1789259971508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "8d93e2a45b5143f80263f1d15b205ed76d01c787",
          "message": "Merge pull request #83 from orcvs/rustdoc-on-pull-request\n\nRun rustdoc on the pull-request tier",
          "timestamp": "2026-09-13T00:49:23Z",
          "tree_id": "40e07dff20a500b9eadb99677a67807f55bce505",
          "url": "https://github.com/orcvs/orcvs/commit/8d93e2a45b5143f80263f1d15b205ed76d01c787"
        },
        "date": 1789262650950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "3c534cc19e2839b9b1c78cf3e41b658cad349429",
          "message": "Merge pull request #82 from orcvs/render-frame-responsibility-02\n\nSeparate glyph placement and record the crate-seam criterion",
          "timestamp": "2026-09-13T03:03:58Z",
          "tree_id": "e3c633f03bcbba0cee7ee815a310a5e94ea9d152",
          "url": "https://github.com/orcvs/orcvs/commit/3c534cc19e2839b9b1c78cf3e41b658cad349429"
        },
        "date": 1789269347381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 5061,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 127,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46643,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 6613,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 358,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 171209,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 9717,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1174,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 653376,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "b3d5f8918a5c71f4cae58bbf5207db3b5b7d66cc",
          "message": "Merge pull request #84 from orcvs/typed-source-paint-tokens\n\nCarry Tokens through the Render Frame and retire Glyph",
          "timestamp": "2026-09-13T03:57:06Z",
          "tree_id": "90909e00161fd10d611a77782f329ce227ad6a5a",
          "url": "https://github.com/orcvs/orcvs/commit/b3d5f8918a5c71f4cae58bbf5207db3b5b7d66cc"
        },
        "date": 1789272457581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "3bad9af387dca4deb63517a2a6fbd5c2a4377a4b",
          "message": "Merge pull request #85 from orcvs/seam-names-and-frame-diagnostics\n\nCarry paint diagnostics and retire leftover Marker names",
          "timestamp": "2026-09-13T05:04:21Z",
          "tree_id": "81d66544a90850488a7f7ba8402163a0f7885358",
          "url": "https://github.com/orcvs/orcvs/commit/3bad9af387dca4deb63517a2a6fbd5c2a4377a4b"
        },
        "date": 1789276434736,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "d309bd2f638f0a2efd7b46d3830a45780c5f4b50",
          "message": "Merge pull request #88 from orcvs/04-add-deterministic-random\n\nAdd deterministic Random",
          "timestamp": "2026-09-13T06:58:52Z",
          "tree_id": "68eb95f817ddb2a703514119b4be03e736c6914e",
          "url": "https://github.com/orcvs/orcvs/commit/d309bd2f638f0a2efd7b46d3830a45780c5f4b50"
        },
        "date": 1789283282524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "65f3608f8ded8738ce3092c6a7c31f08894d0b10",
          "message": "Merge pull request #86 from orcvs/03-add-visible-increment-and-interpolation\n\nAdd Increment and Interpolation with declared Portal inputs",
          "timestamp": "2026-09-13T17:59:12+10:00",
          "tree_id": "bfb5475911ca2c33b03a6842ae3793b5916f1571",
          "url": "https://github.com/orcvs/orcvs/commit/65f3608f8ded8738ce3092c6a7c31f08894d0b10"
        },
        "date": 1789286733103,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "f23fad22d2d073cdcd94d208eea1d8e1cc6f0330",
          "message": "Merge pull request #89 from orcvs/source-paint-09\n\nMeasure Paint derivation so viewport cost is on the series",
          "timestamp": "2026-09-13T08:13:18Z",
          "tree_id": "b2e67e4f2bac99251ff6fe5033dd2c6a23e2eca6",
          "url": "https://github.com/orcvs/orcvs/commit/f23fad22d2d073cdcd94d208eea1d8e1cc6f0330"
        },
        "date": 1789289345603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "398926b96c66f7ea444dfb044c9c0b4b7b87ae6f",
          "message": "Merge pull request #93 from orcvs/03-05-sequence-functions\n\nAdd structural and Range Sequence Functions (issues 03 & 05)",
          "timestamp": "2026-09-13T23:40:48Z",
          "tree_id": "bb7e22a675a01293b5cbe6e8edbd0c49a24c5fd3",
          "url": "https://github.com/orcvs/orcvs/commit/398926b96c66f7ea444dfb044c9c0b4b7b87ae6f"
        },
        "date": 1789343615460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "d4013e6328c6a001cf7d44078bb52ed57ecbdb35",
          "message": "Merge pull request #92 from orcvs/dependabot/cargo/rust-dependencies-06ff8b9c65\n\nBump the rust-dependencies group with 2 updates",
          "timestamp": "2026-09-13T23:40:53Z",
          "tree_id": "79fa9d04f797df6c300557e70dea34b09b21e67a",
          "url": "https://github.com/orcvs/orcvs/commit/d4013e6328c6a001cf7d44078bb52ed57ecbdb35"
        },
        "date": 1789344255980,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "3ccf1e4f62705be1c78619685c8688be0b048c13",
          "message": "Merge pull request #95 from orcvs/live-note-range-happy-path\n\nAdd live happy-path tests for Sequence Functions",
          "timestamp": "2026-09-14T00:31:28Z",
          "tree_id": "680e6dd11f803e17cc0d2dccb6cfef5d3fba117d",
          "url": "https://github.com/orcvs/orcvs/commit/3ccf1e4f62705be1c78619685c8688be0b048c13"
        },
        "date": 1789346725685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "406a4642742085f95ecd09e0661e8902f043b40e",
          "message": "Merge pull request #94 from orcvs/03-add-the-self-banging-functions\n\nRaise ADR 0009's Terminal Output Portal refusal in shipped code",
          "timestamp": "2026-09-14T00:37:23Z",
          "tree_id": "eb051146f8d24e7b02aaf05b1e2b4cc66e7e0f81",
          "url": "https://github.com/orcvs/orcvs/commit/406a4642742085f95ecd09e0661e8902f043b40e"
        },
        "date": 1789347303990,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "388f812f373704722231501eaf308b4fcbe763ee",
          "message": "Merge pull request #97 from orcvs/02-prove-product-persistence-paths\n\nProve product persistence through shipped storage paths",
          "timestamp": "2026-09-14T04:53:46Z",
          "tree_id": "8b9b2062476875933e1c8af8d5a1ce972569ec5a",
          "url": "https://github.com/orcvs/orcvs/commit/388f812f373704722231501eaf308b4fcbe763ee"
        },
        "date": 1789362535264,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "87fc79362e39c22feb42cc7d425de9dd420edaf6",
          "message": "Merge pull request #98 from orcvs/04-add-directional-jump-chains\n\nAdd directional Jump as ordinary Portal output",
          "timestamp": "2026-09-14T07:01:08Z",
          "tree_id": "32286075eec427a4c6cbd9487bea9185d7cd0cee",
          "url": "https://github.com/orcvs/orcvs/commit/87fc79362e39c22feb42cc7d425de9dd420edaf6"
        },
        "date": 1789370076032,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "45d18eb91c516e4933d07f20516d4463c136fea9",
          "message": "Merge pull request #99 from orcvs/kind-shaped-portal\n\nIntroduce Destinations silence so Terminal Output cannot be stuffed with a Portal",
          "timestamp": "2026-09-14T10:23:43Z",
          "tree_id": "67aea90da936ab1852c0e791acb001790c6e30bf",
          "url": "https://github.com/orcvs/orcvs/commit/45d18eb91c516e4933d07f20516d4463c136fea9"
        },
        "date": 1789382272651,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "6baf9572a82fd52e6778e4841493236f7129baa7",
          "message": "Merge pull request #96 from orcvs/05-add-halt-root-locking\n\nAdd Halt root locking",
          "timestamp": "2026-09-14T23:26:20Z",
          "tree_id": "47d6a0d259213f0fd53823719319075148701dd8",
          "url": "https://github.com/orcvs/orcvs/commit/6baf9572a82fd52e6778e4841493236f7129baa7"
        },
        "date": 1789429245707,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "6aac9fd69f1179fa34a2ee1c986b3f1d194fd8f4",
          "message": "Merge pull request #100 from orcvs/fix/portal-root-locks\n\nDistinguish Portal root locks from Cell writes",
          "timestamp": "2026-09-15T06:20:50Z",
          "tree_id": "d360d35d191fafbd4748eb8632244ab71e4437d1",
          "url": "https://github.com/orcvs/orcvs/commit/6aac9fd69f1179fa34a2ee1c986b3f1d194fd8f4"
        },
        "date": 1789454094841,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "1a68a192156769a511f21c9721eaaf77a6a4a196",
          "message": "Merge pull request #101 from orcvs/cursor-effects\n\nAdd animated cursor effects and theme controls",
          "timestamp": "2026-09-16T00:41:49Z",
          "tree_id": "757b00a4fcefe6c96c8b8e26626a7be6048dae3e",
          "url": "https://github.com/orcvs/orcvs/commit/1a68a192156769a511f21c9721eaaf77a6a4a196"
        },
        "date": 1789520220442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "e0e60ff2a50eee1521f05ea1785f5a39e1655b20",
          "message": "Fix PWA icon dimensions",
          "timestamp": "2026-09-16T11:02:05+10:00",
          "tree_id": "2f7f8f94c8382404df85ce87d61c8410484f6340",
          "url": "https://github.com/orcvs/orcvs/commit/e0e60ff2a50eee1521f05ea1785f5a39e1655b20"
        },
        "date": 1789528483202,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "ce8b9483014e69d21985b08c2ecdcdd63aeebdda",
          "message": "Merge pull request #103 from orcvs/midi-selection-ownership\n\nDeepen MIDI selection ownership within Playback",
          "timestamp": "2026-09-16T06:50:41Z",
          "tree_id": "6ded6a5a67183aab61737e4d41692bfa46b139fe",
          "url": "https://github.com/orcvs/orcvs/commit/ce8b9483014e69d21985b08c2ecdcdd63aeebdda"
        },
        "date": 1789545080234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "71e41c4f6fc6f61e37f15536935a4cbdc6fa4b34",
          "message": "Merge pull request #104 from orcvs/egui-agent-workflow\n\nGive the console an egui skill, opt-in inspection, and kittest coverage",
          "timestamp": "2026-09-16T08:03:48Z",
          "tree_id": "4abd37857aacef485ce65ceed1e8d06efeb945f0",
          "url": "https://github.com/orcvs/orcvs/commit/71e41c4f6fc6f61e37f15536935a4cbdc6fa4b34"
        },
        "date": 1789548297741,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "208c50aeb4fb04a1e2b76e789fa648e7d986ada9",
          "message": "Merge pull request #102 from orcvs/console-panel\n\nPut playback readouts and controls on a static console panel",
          "timestamp": "2026-09-17T00:59:13Z",
          "tree_id": "f61ffd0a5f2799d2d34c359cc34e59ff24971885",
          "url": "https://github.com/orcvs/orcvs/commit/208c50aeb4fb04a1e2b76e789fa648e7d986ada9"
        },
        "date": 1789607987057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "710ec9585d3fdc884fcbe97a509f5d41dafc56ea",
          "message": "Merge pull request #105 from orcvs/source-view\n\nPresent the Source as a bounded space at a stated Zoom",
          "timestamp": "2026-09-18T06:02:28Z",
          "tree_id": "9a6486f29f16dda5ab5c36a32142deea42f4cd44",
          "url": "https://github.com/orcvs/orcvs/commit/710ec9585d3fdc884fcbe97a509f5d41dafc56ea"
        },
        "date": 1789712191112,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "6101ca94cc77e632b58b588ebb93f432b857f0f9",
          "message": "Merge pull request #108 from orcvs/sector-navigation\n\nStep the Cursor a Sector at a time with Tab, and keep every key from the Source while a popup is open",
          "timestamp": "2026-09-19T08:28:24Z",
          "tree_id": "de0456066df350e619ab92afdc31df224a08320e",
          "url": "https://github.com/orcvs/orcvs/commit/6101ca94cc77e632b58b588ebb93f432b857f0f9"
        },
        "date": 1789807370329,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "88b75f2295eb5c88f1b465804738db1101249d32",
          "message": "Merge pull request #106 from orcvs/source-view-margin\n\nDouble the default Grid, add a two-Cell margin, and show a grab pointer while Panning",
          "timestamp": "2026-09-20T00:47:41Z",
          "tree_id": "cb9212514850714ac9e5114c304daa8141e0e369",
          "url": "https://github.com/orcvs/orcvs/commit/88b75f2295eb5c88f1b465804738db1101249d32"
        },
        "date": 1789866171706,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "77b57659adbd5b0109a7dfe33d3dd148d717776e",
          "message": "Merge pull request #110 from orcvs/untracked-cleanup\n\nCommit the untracked working files that are repository state, and ignore the ones that are not",
          "timestamp": "2026-09-20T01:23:54Z",
          "tree_id": "0949301900ab868c2641fb0319abc89e948df446",
          "url": "https://github.com/orcvs/orcvs/commit/77b57659adbd5b0109a7dfe33d3dd148d717776e"
        },
        "date": 1789868293872,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "c6b25c0ff7d1ab051c8a096fd291a15e6ff04fa2",
          "message": "Merge pull request #111 from orcvs/syntax-highlighting-07-follow-ups\n\nClose the syntax-highlighting 01-04 review follow-ups, and split theming out of the console restyle",
          "timestamp": "2026-09-20T03:21:59Z",
          "tree_id": "60c1aa23e665d682da24c204dab0daa307b38668",
          "url": "https://github.com/orcvs/orcvs/commit/c6b25c0ff7d1ab051c8a096fd291a15e6ff04fa2"
        },
        "date": 1789875366790,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "ff4844df8b435b9242a7a80f6646776a0924794d",
          "message": "Merge pull request #112 from orcvs/parser-borrow-and-tracing-cleanup\n\nAccept immutable Source text, and keep test-only tracing out of shipped graphs",
          "timestamp": "2026-09-20T04:26:04Z",
          "tree_id": "938675cd642252032d60eb73e22eab7882bc9036",
          "url": "https://github.com/orcvs/orcvs/commit/ff4844df8b435b9242a7a80f6646776a0924794d"
        },
        "date": 1789879283702,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "3650c7d8c463cf1e48fa7d37c8e84b408ad234be",
          "message": "Merge pull request #113 from orcvs/syntax-highlighting-12-fitted-portal\n\nFit Sequence Output Portal highlights to their answers",
          "timestamp": "2026-09-20T09:50:37Z",
          "tree_id": "ec1add6169d12ac42de0758f85254e57e62bec47",
          "url": "https://github.com/orcvs/orcvs/commit/3650c7d8c463cf1e48fa7d37c8e84b408ad234be"
        },
        "date": 1789898705375,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "ea9a75067476b7fcdaa2a0e8d39d45f57622639a",
          "message": "Merge pull request #116 from orcvs/dependabot/github_actions/github-actions-abd2b60a87\n\nBump benchmark-action/github-action-benchmark from 1.22.1 to 1.22.2 in the github-actions group",
          "timestamp": "2026-09-21T00:15:17Z",
          "tree_id": "5881a4bbc5ecab1b901af8bbf0c20f4ce0336951",
          "url": "https://github.com/orcvs/orcvs/commit/ea9a75067476b7fcdaa2a0e8d39d45f57622639a"
        },
        "date": 1789950569293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
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
          "id": "73b400e1ab597402d61b2bfc5115130467e5eec4",
          "message": "Merge pull request #117 from orcvs/dependabot/cargo/rust-dependencies-81b68b975f\n\nBump egui from 0.36.1 to 0.36.2 in the rust-dependencies group",
          "timestamp": "2026-09-21T01:38:13Z",
          "tree_id": "fa6caefd79d5fe238f41bdf0dd2c09038ed7e708",
          "url": "https://github.com/orcvs/orcvs/commit/73b400e1ab597402d61b2bfc5115130467e5eec4"
        },
        "date": 1789956994338,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "lang render frame re-read fixture blocks",
            "value": 24,
            "unit": "blocks"
          },
          {
            "name": "lang render frame re-read fixture bytes",
            "value": 48,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture blocks",
            "value": 11,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture bytes",
            "value": 816,
            "unit": "bytes"
          },
          {
            "name": "lang tick fixture written four times blocks",
            "value": 44,
            "unit": "blocks"
          },
          {
            "name": "lang tick fixture written four times bytes",
            "value": 3264,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 empty bytes",
            "value": 4661,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated blocks",
            "value": 111,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 16x16 populated bytes",
            "value": 46003,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 16x16 populated expressions carried",
            "value": 43,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 32x32 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 empty bytes",
            "value": 5813,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated blocks",
            "value": 326,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 32x32 populated bytes",
            "value": 169417,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 32x32 populated expressions carried",
            "value": 160,
            "unit": "expressions"
          },
          {
            "name": "orcvs write one cell 64x64 empty blocks",
            "value": 10,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 empty bytes",
            "value": 8117,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated blocks",
            "value": 1110,
            "unit": "blocks"
          },
          {
            "name": "orcvs write one cell 64x64 populated bytes",
            "value": 647744,
            "unit": "bytes"
          },
          {
            "name": "orcvs write one cell 64x64 populated expressions carried",
            "value": 621,
            "unit": "expressions"
          }
        ]
      }
    ]
  }
}