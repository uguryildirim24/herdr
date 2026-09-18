# SPEC-durable turn 03 — draft notes

Author: lane `spec-durable` (`w1F:p4`). Spec: `tasks/SPEC-durable.md` v2. No code. Merged critic `5881ca31` (docs only, fast-forward).

## What changed

All 16 critic objections were accepted. Coordinator already locked 5, 12, and 15.

`herdr server restart` is still sugar over live-handoff, but the default after install is an upgrade. Spawn uses `launch_executable`. `--exec` is validated in the CLI. Waits retry only on `server_handed_off`. TUI reconnect is out; the operator re-runs `herdr`. First install from 0.9.0 is a seven-step runbook that uses `cargo install --path . --locked --root ~/.local` and `herdr server live-handoff` with no `--expected-*`.

Lineage: handoff keeps every non-TTL token; cold restore keeps `parent`/`lane`/`round`/`branch`. Names restore on cold start only when a native plan is typed. `managed_agent_args` are replayed per kind, including claude. `AgentStartParams.parent` is validated before launch. The drill is `live_restart_keeps_lane_tree` in `tests/live_handoff.rs`, rows 1–13, 16 fake panes.

§10 is the three-lane split (A restart-core, B lineage-persist, C cycle-gate). Field name: `parent`.

## What I rebutted

Nothing. Every objection has a spec change. Cursor/agy `sessionStart` after `--resume` is still unmeasured; that is a documented limit, not a rebuttal.

## Observed this turn

**O10.** `cargo install --root` of a probe crate onto a running copy: dest inode changed; the process kept the old inode and stayed alive. Cargo 1.96.1 stages under dest then `rename`. Not an in-place write. Never wrote `~/.local`.

**O11.** Throwaway `--session spec-durable-03-o1`, isolated XDG, 0.9.0 copy → rename to 0.9.1 release → `live-handoff` with the 0.9.0 CLI, no `--import-exe`. Exit 0. Pids and names kept. Tokens gone. `set-parent` restored `parent`. Live default session stayed running.

## Left for Rolf

None that block the build. Reverse 0.9.1 → 0.9.0 is untested and not required. The drafter and the critic do not disagree after evidence.
