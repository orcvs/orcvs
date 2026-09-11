window.BENCHMARK_DATA = {
  "lastUpdate": 1789112210431,
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
      }
    ]
  }
}