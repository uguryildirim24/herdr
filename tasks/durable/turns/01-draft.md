# SPEC-durable turn 01 — draft notes

Author: lane `spec-durable` (this pane `w1F:p4`). Spec: `tasks/SPEC-durable.md` v1. No code.

## Design

Rolf needs the fork to survive a bounce the way an ADE should: processes stay, turns stay, the lane tree stays, cycles cannot form, agents come back with their names.

The protocol is already in tree. `HandoffManifest` plus SCM_RIGHTS batches (`d66c39d5` / #3411) already move live PTYs to a new Unix server. The operator path that uses it is `herdr update --handoff`, which downloads from herdr.dev and would overwrite this `cargo install` fork. The 0.9.1 binary also accepts `herdr server live-handoff [--import-exe]` with no download. **`herdr server restart [--exec]` is that call**, aimed at `current_exe` or a freshly installed binary, with no version pin. Fifteen panes are not a 64-fd wall; 64 is the per-message batch.

What the ADE still lacks is everything around that fd transfer. Capture never writes `metadata_tokens`, so `parent` dies on both cold restore and handoff. `set-parent` / `report-metadata` / `agent start --parent` will write a cycle; only drag-to-nest refuses one, and then by showing no drop target. Cold restore drops an `agent_name` when `managed_agent_kind` is missing — that is the coordinator (`hcoord` in live `session.json`: claude session id, name set, kind absent). `pane run` is send-text+Enter, not a launch argv, and must not be replayed after a crash. Kinds with a `plan()` (`claude --resume`, `cursor-agent --resume`, `codex resume`, `opencode --session`, `agy --conversation`, …) type that line into a fresh shell after stop/reboot. `chatgpt` and `dsh` cannot. Live handoff must not type those lines: the process is still there.

Cuts: restart must not signal children. The 2 s reader pause and 8 KiB replay (skipped on alt-screen / agent-session panes) bound herdr's parser, not the agent's turn. Attached TUIs today exit on `ServerShutdown` (`live update in progress; reconnect after handoff completes`). In-flight waits return `server_unavailable`. The spec makes the TUI reconnect and the wait CLI retry once so a watch pane does not shout GONE.

Failure: if the import never says `ready`, keep the old server and kill the child. After commit, always stop the old server and do not let it rewrite `session.json`.

Non-goals: Windows, remote/cloud panes, transferring waiters in the manifest, fixing the existing `live_handoff` SIGKILL harness.

## Observed (throwaway `--session spec-smoke`)

Binary: checkout 0.9.1 debug. Isolated XDG under `/var/tmp/spec-smoke-durable`. Never the live `herdr.sock`, never `herdr server stop` on default.

Three panes. `sleep 3600` on `w1:p2` (pid 7512). Armed `pane wait-output` on `w1:p3` (pid 7539). `report-metadata` wrote a cycle `p1↔p2` plus `lane=spec-smoke`. `agent set-parent` on the shell pane returned `agent_not_found`. `session.json` v3 had no tokens. `server live-handoff` (no `--import-exe`) succeeded in about one second (old 3914 → import 9941). Sleep 7512 still alive. Wait 7539 dead with `server_unavailable` / `server is shutting down`. Tokens gone. Public ids kept; terminal ids new. Nested TUI in the outer pane printed `herdr: server shut down: live update in progress; reconnect after handoff completes` and returned to zsh; the named server kept running. Then `session stop` / `delete` spec-smoke only, and the outer pane closed. Live default session still running.

Live default `session.json` (read-only): no parent tokens; `hcoord` named without `managed_agent_kind`; this lane `spec-durable` is cursor with a session id. API still shows `parent=w1F:p1` on this pane.

Installed CLI is 0.9.0 and does not advertise `live-handoff`. The 0.9.1 dispatcher does.

## Least sure

Cursor cold-resume flags (`--model` / `--force`) — not retyped; not measured on stop+start this turn (would kill this lane). Whether a local TUI reconnect is in scope or Rolf will re-attach. Whether wait should retry or only name the error (watch-pane GONE is the cost of the second). Whether workspace `round=` tokens should ride along with pane tokens. Whether the 2026-09-17 tree scramble was a stray **valid** drag (cycle drag is already inert). `agent_parent_notify`'s RAM `blocked_notified` bit after a handoff of a child that stayed blocked. I did not rebuild this worktree; observations used the rebase debug binary at the same commit.
