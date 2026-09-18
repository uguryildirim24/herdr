# SPEC-ADE turn 01: the draft in prose

Lane spec-ade, 2026-09-18. Spec: `tasks/SPEC-ADE.md` v1 on `lane/spec-ade`.

## The design

The coordination layer moves out of my skills and notes into a plugin forked from
`eliasstravik/herdr-projects`, renamed `herdr-ade`, running on Rolf's fork. The fork changes for
nothing in this round: its two contributions, the `parent` token with BLOCKED/GONE pushes and the
durability work, are what the plugin leans on, and the plugin leans on them from the outside.

Three things change in the plugin. First, a roles table replaces the single `thread_agent`: six
roles, each a kind plus argv plus tab env, kept in the safety file agents are told not to write, so a
model pick is one edit and no rebuild. Second, threads become what lanes are today: tabs in the
coordinator's workspace on plain git worktrees under the repo, started with `--parent`, and the
ticker re-issues `set-parent` whenever the token is missing. That last sentence is the load-bearing
one: I watched a 0.9.1 live handoff drop every token, including a parent I had just set, and watched
the plugin's own tokens come back within one tick because the ticker owns them. Parents get the same
owner, so they survive any handoff before the durability spec lands and after it. Third, the lane's
closing lines become two verbs, `ha done` and `ha waiting`, that gate on a clean worktree and an
existing report, write the record and the tokens, and then type the exact `DONE`/`WAITING` line
into the coordinator as today, with a queue for the case where the line cannot land.

On top sit rounds (`round open`, `round review`, `round merge`, the last gated on a MERGE verdict
file, which is how the coordinator gets to merge without breaking upstream's "never merge on your
own" rule), the two-thread spec dialogue with Pro adopted rather than started, a `RULES.md` for
Rolf's standing rules appended to every skill print and brief, and checkpoint/resume ported from
`state.py`. Priming stays the plugin's typed line for every CLI. I looked for a reason to inject text
through the fork's hooks and found none: seventeen hook assets to bump, and the two hookless kinds
Rolf actually uses would still need the typed path.

The migration is a rename install and one live handoff with an explicit `--import-exe`, preceded by
an inventory that becomes a re-link script, and followed by Rolf re-running his TUI. The plugin is
linked only after the server is 0.9.1, first into a throwaway config, then for real. The three live
builds finish their current rounds on the skills; adoption of a live build needs a plugin change
because upstream's `adopt-workspace` would start a second coordinator. Skills and notes go last.

## What I observed

Everything is in `tasks/SPEC-ADE.md` §0.3 as O1 to O8, raw material in
`.reports/spec-ade-01/`. The short version: the plugin builds and its 150 tests pass; it links on
0.9.1 and refuses 0.9.0 through `HERDR_BIN_PATH`, which every herdr pane exports; it starts agents
with no flags at all, so Claude's trust dialog appears and the ticker primes once the dialog is
answered; a tab thread ran end to end with tokens and an inbox item; a 0.9.1 to 0.9.1 handoff took
0.125 s and lost all tokens; a 0.9.1 to 0.9.0 handoff to the installed binary also completed, and so
did the way back. The live server has 21 panes, 14 parent links, two unnamed coordinators, one
plain process that must survive (the flyonenomics watcher), and no armed waits at the time I looked.
The rebase review has three fix commits and no verdict yet.

## What I am least sure of

- Whether re-stamping `parent` from the ticker interacts badly with the server's own link index
  (`src/app/agent_parents.rs`) right after a handoff, when the server has no links and the agents
  have not reported status yet. I could not observe a GONE or BLOCKED in the throwaway.
- The reverse handoff at 21 panes with two `chatgpt` panes. Two panes proved the path, not the load.
- Where the roles table belongs. The safety file keeps agents out of it by rule, not by permission,
  and Rolf may prefer seeing roles in `PROJECT.md` every turn.
- Whether committing briefs through a temporary index from inside the plugin is acceptable to Rolf
  as "the plugin does git", given his rule that the coordinator never codes.

## Three questions for Pro

1. D3 and D5 together make the plugin the owner of both the tree and the DONE record. Is there a
   sequence (handoff during a `DONE` push, or a coordinator blocked at a dialog) where the record
   says done, the line never lands, and `ha context` does not make that visible on the next turn?
2. §3.3(a) hands off 21 panes with an explicit `--import-exe` after a rename install. What does
   the two-pane test not cover: fd batching, unnamed coordinators, `chatgpt` re-detection, or the
   old server's environment being inherited by the new one?
3. D14 keeps priming as one typed line for all seven kinds. Which kind, on its first prompt after
   `agent start` reports ready, can drop or garble that line in a way the plugin's ready-gate does
   not catch?
