# Spec turn 03 — SPEC-durable v2 (lane spec-durable)

The critic's turn is on `lane/spec-critic` at 5881ca31: `tasks/durable/turns/02-critic.md`
(340 lines: 16 objections ranked B/C/X with observations O1 to O9 from a throwaway
0.9.0 to 0.9.1 handoff, a replacement §6 drill, answers to every §8 item, and a
three-lane split). `git merge lane/spec-critic` into `lane/spec-durable` first
(docs only, no conflicts expected).

Write **v2** of `tasks/SPEC-durable.md`. For every objection 1 to 16: accept it and
change the spec, or rebut it with evidence of the same grade (`file:line`, sha, or an
observation you make). Record each outcome in a new §9 "Turn 03 dispositions" table:
objection, accepted or rebutted, where the spec changed. Where you and the critic
still disagree after evidence, put it in §8 as a question for Rolf; do not settle it
by assertion.

Take these as decided, from the coordinator:

- Objection 5 stands: v2 gets a **§1.4 First install from the running 0.9.0 server**,
  the critic's seven-step runbook, verified by you on a throwaway session the way O1
  was. Rolf installs with `cargo install --path . --locked --root ~/.local`; find out
  whether that replaces `~/.local/bin/herdr` by rename or by in-place write (test
  with a throwaway `--root` and a running copy), and write the runbook so the
  running server's file is only ever swapped by rename.
- Objection 15 stands: TUI reconnect is deferred to a later round. §1.2.1 shrinks
  to "the operator re-runs `herdr`" plus the limits the critic lists.
- Objection 12 stands: all 22 `live_handoff` tests pass on this tree on macOS. Drop
  the stale Q8 language; the drill goes into `tests/live_handoff.rs` in the critic's
  definite shape (rows 1 to 13, 16 fake panes).
- The critic's lane split (A restart-core, B lineage-persist, C cycle-gate, one
  reviewer) becomes §10 of the spec, with owned files and the two seams, so the
  coordinator briefs lanes straight from it. Agree the `AgentStartParams.parent`
  field name in the spec.

Answers to the critic's questions, from the coordinator:

1. The "Claude keeps flags in the transcript" claim is from the save-state skill
   (`~/.claude/skills/save-state/SKILL.md`, the restart table): two Claude panes
   resumed by herdr with `claude --resume <id>` kept their names, session ids and
   bypass mode on 2026-09-14. Model and effort were not measured. So replay args
   for every kind per objection 1; for claude it is harmless.
3. `cargo install --path . --locked --root ~/.local`. Verify the rename question
   yourself (above).
4. Reverse handoff 0.9.1 to 0.9.0 as rollback: state it as untested and not
   required this round.
5. `w19` is a lane: a Claude Opus pane named `w19` (pane `w16:p2V`) in the
   flyonenomics build. On 2026-09-17 at 19:45 the other lanes and the coordinator
   ended up parented under it; the mechanism was not recorded. Treat the stray-drag
   theory as unproven and keep the immediate drop, per the critic's answer to Q1.
6. `restart` through `herdr --remote`: refuse with a clear error; remote stays out
   of scope.

Commit as `docs(spec): SPEC-durable v2 (turn 03)` on `lane/spec-durable`, with
`tasks/durable/turns/03-draft.md` (one page: what changed, what you rebutted, what
is left for Rolf). Report `.reports/spec-durable-03-report.md` (git-ignored, not
committed). Same env and rules as turn 01: throwaway sessions only, never the live
socket, never `herdr server stop` on the default session, never `cargo install`
into `~/.local`, never `git stash`.

## Closing steps (verbatim, every turn ends with one of these, also on failure)

```
herdr pane report-metadata $HERDR_PANE_ID --source lane --token lane=spec-durable-03 --token done=1
herdr notification show "spec-durable-03 done" --body "lane spec-durable" --sound done
herdr agent prompt hcoord "DONE spec-durable-03 .reports/spec-durable-03-report.md <final commit sha>" || herdr agent prompt hcoord "DONE spec-durable-03 .reports/spec-durable-03-report.md <final commit sha>"
```

If you must stop for something outside the lane:

```
herdr pane report-metadata $HERDR_PANE_ID --source lane --token lane=spec-durable-03 --token waiting="<what>"
herdr agent prompt hcoord "WAITING spec-durable-03 <what>"
```

Push even when you failed.
