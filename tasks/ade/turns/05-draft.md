# Turn 05: Fable answers Pro's turn 04 and writes SPEC-ADE v3

Lane spec-ade, pane w1F:p8, 2026-09-18. Input: `tasks/ade/turns/04-pro.md` (`converged: no`, B1 to
B6, S1 to S6, N1, open questions 24 to 26), `tasks/ade/turns/00-brief-05.md` (coordinator readings),
`tasks/ade/decisions.md`, `tasks/ade/tbc.md`. Output: `tasks/SPEC-ADE.md` v3 in place. Pro's hook
documentation citations (H1 to H3) are taken as facts, as the brief says; nothing here contradicts
them. Fork claims re-verified this turn on the worktree: the 80-character token value cut
(`src/app/api_helpers.rs:208,244-275`), the read structs without expiry
(`src/api/schema/panes.rs:527-556`, `agents.rs:206-210`, `workspaces.rs:62-75`), `ttl_ms` only on the
report requests (`panes.rs:505`, `workspaces.rs:58`), `PaneProcessInfo` (`panes.rs:570-594`),
`agent set-parent` with `ttl_ms: None` (`src/cli/agent.rs:873-925`), the skills' tokens written
without TTL (`round-table/SKILL.md:32,186,195`), the pane env exports (`src/integration/env.rs:8-10,28-33`),
the plugin's `coordinator::open` path (`hp:src/coordinator.rs:77,174-201`, `hp:src/project.rs:195-209`),
`hp:src/actions.rs:18,108-109`, `hp:src/main.rs:1-21`, `hp:herdr-plugin.toml:8-90`, and the 0.9.1 CLI
verbs `tab create --env/--no-focus`, `pane run`, `pane get`, `agent send-keys esc`.

## Dispositions

| # | finding | disposition | reason | changed |
|---|---|---|---|---|
| B1 | a stop hook does not gate what Rolf has already seen | accepted | true for all three documented hooks (H1 to H3); no native pane gets a `gated` label; gate A is now a correction hook after display with a per-kind table; the pre-display guarantee exists only on plugin-owned surfaces; D18 designs the plugin-owned conversation surface Pro named as the candidate and item 24 asks Rolf whether it ships | D17 lead, D17 item 2, D18 new, D16, §6 item 24, §5 |
| B2 | the check can certify opaque explanations | accepted with change | the check is now stated as a form check with rules R1 to R7 and an explicit "does not certify understanding" clause; binary-owned surfaces get Pro's typed envelope and templates; no glossary exemption (gloss form only); Pro's adversarial fixtures are tests. Change: in free prose the recorded birth sentence appears in the gloss form `<sentence> (<name>)` per the coordinator's reading, instead of banning names from prose; on the board and in notifications names never appear (S6) | D17 items 1, 3, 6; §1.3 |
| S1 | bound correction attempts; bind hooks to the coordinator lifetime | accepted | persisted budget per native turn (three requests, one translator run, ten minutes, 64 KiB, 60 s), continuation never resets it, exhaustion publishes a fixed notice and copies nothing; hook installed by `ha open` before launch, removed by `ha close`/rebind, never by `ha done`; scoped by pane id and session id; tests listed | D17 items 2, 5; §1.3 |
| S2 | `ha ask` needs a durable record and a complete display path | accepted | record first at project scope with revision, publication derived; compact board line at most 60 characters checked separately; full question with numbered choices on the surface or in `ha context`; answer binds id and revision, stale refused, nothing else clears an ask; the "only channel" claim is withdrawn ("structured way"); the fork limit is not widened | D17 item 4, §6 item 14, §1.3 |
| B3 | D5 does not define a recoverable validated completion | accepted | op (reserved, staged, sealed), immutable sealed event with tagged `done`/`waiting` payload, append-only delivery journal, content-addressed artifact staged before seal, git outside the project lock with a repository lock and a stated lock order, crash points X1 to X5 with recovery, `context` acknowledges only for the bound recipient and never on `--peek`, manifest rebuilt from events | D5, D4, D9, D6 admission; §1.3 |
| B4 | the verdict must carry its own hash; merge lacks a pre-effect phase | accepted | commits `B` (brief), `C` (candidate), `V` (verdict-only child of `C`), `H` (checkpoint); merge verifies `V^ = C`, `C..V` is the verdict file only, `manifest_hash`/`policy_hash`, ancestry of `B` and every lane sha in `C`, head still `B`; merge intent written before any git effect; recovery from the actual ref; ref writers serialized on the repository lock; acceptance asserts `main = V` then `V` ancestor of `H` | D6, D4, D9; §1.3; §4.3 rows 5 to 7 |
| S3 | restore the brief-hash and observable-binding receipt | accepted | receipt input is project, thread, attempt, brief hash and recipient binding, passed through `HERDR_ADE_LAUNCH` at tab creation; second call returns `already accepted` and reauthorizes nothing; a blank composer is ambiguous and keeps the send pending; D3 binds pid, argv0 and session id, rebinds only across a verified handoff; D8 states the composer limitation and counts D18 as the only ownership mechanism | D14, D3, D4, D8; §1.3 |
| B5 | the barrier has no complete send ownership and no fenced go point | accepted | owner table (r2 reviewer, `hcoord`, build coordinators, workers, Rolf per decision 15) recorded in `go.toml`; three explicit control messages `CHECKPOINT`, `RECONCILE`, `RESUME`; worker pushes declared possibly missed and reconciled from files; a final fenced check by `hcoord` after the install and immediately before the handoff; any drift is no go without stopping anything; `RECONCILED` receipt before `RESUME` | §3.1, §3.2 P3, §3.3 steps 1 to 5 and 10, §3.4 |
| B6 | the inventory cannot derive expiry; `agent list` cannot verify every pane | accepted | raw capture preserved; `tokens.toml` classifies each value by its known producer with the citation (`set-parent` `ttl_ms: None`, the skills' lines without `--ttl-ms`); unknown keys unresolved and never restored; a required unresolved value blocks go; `relink.sh` checks the pane's foreground pid per line; step 8 verifies through `pane get` and `workspace list` per id | §3.1, §3.3 steps 3 and 8, §3.4 |
| S4 | retry-safe install and rollback | accepted | create-only backup named by source hash and hash-verified; staged `cargo install` into a stage root, hash equals the manifest's target before the rename; idempotent lines; rollback in named states (before commit with restoration failure covered, after commit conditional on a responsive server and Rolf's go, no automatic recovery when control is unavailable); both binaries kept; installed CLI path stated | §0.2, §3.3 step 6 and rollback, §3.4, §6 item 31 |
| S5 | the lane split needs shared contracts and executable tests first | accepted with change | prerequisite A0 `ade-contracts` (shared types in `src/contracts.rs`, the real pure checker with word lists and fixtures, scenario wiring, acceptance skeleton, `Cargo.lock`), pinned; A1 owns `main.rs`, `ticker.rs` whole and the git module; A2 owns `cli.rs` registration; A3 owns the assembled acceptance script; no permissive stub. Change: the rename tuple including `actions.rs` sits in A0 rather than A1, so every lane branches from the renamed crate. Acceptance rows corrected per Pro's table: required set declared, NOT-RUN never passes a required row, deterministic fault rows separated from live rows, independent fixtures, the blocker released before delivery is expected, `main = V` then `H` | §4.2, §4.3, D15 |
| S6 | plugin-only board without raw names, no legacy publisher | accepted | `ade_stage` and `ade_lanes` are templated sentences without registry names; absent or expired rows mean unavailable, not clear; open asks persist in the record; no `ade_*` stamping by the four builds (decision 17); short checked lines plus detail views, width unknown to the plugin | §6 item 14, D17 item 3, §5 |
| N1 | narrow the r2 atomicity wording; do not sum lane tests | accepted | "parent applied before launch; a failed typed send can leave the token to reconcile" with the lineage report lines; the r2 reviewer supplies the combined result including the Linux-only crate; P1 requires it | §0.4, D3, §3.2 P1 |
| 24 | can the coordinator pane be a plugin-owned conversation surface? | accepted with change | designed as D18 (rendering, input, how a checked reply reaches it per kind, what Rolf loses, records, tests, per-project `talk` switch) rather than left as an investigation; whether it ships in the first round is Rolf's, with a recommendation to ship it for the `claude` coordinator kind | D18, §6 item 24, §4.2 A3, §4.3 row 10 |
| 25 | minimum complete question view | accepted | compact line on the board, the full question with numbered choices on D18 when on and in `ha context` when off, the answer bound to id and revision; written in as the coordinator's reading, Rolf overrules | D17 item 4, §6 item 25 |
| 26 | which adapters are mandatory | accepted with change | the required set is declared now in D15 per the coordinator's reading (Claude coordinator and lane, Cursor lane and reviewer, `chatgpt` as adopted Pro for attack turns), installed versions recorded, NOT-RUN never passes a required row, other kinds labelled `capability: unqualified`. Change: declared in the spec rather than left for the reviewer to declare before the run | D15, §4.3, §6 item 26 |

Totals: 16 rows, 12 accepted as written, 4 accepted with change, 0 rejected.

Pro's audit of the 43 turn-03 rows (`04-pro.md:246-272`) is not a new finding set; the rows it
marks "not closed" or "partial" (D5/Q1/§6.9/item 15, D6/item 16, D8, D14/D15/Q3/§6.12, M2/R7,
M5, M6, item 14) are closed through B3, B4, S3, B5, S4, B6 and S6 above; the rows it retains are
unchanged in v3. R1 to R6 are not reopened.

## What v3 changes, in Pro's implementation order

1. Publication ownership (B1, item 24): D17's lead paragraph now says which surfaces the plugin
   owns and that a native pane is checked after display at best; the capability table in D17
   item 2 labels every kind; D18 is the plugin-owned surface, fully designed, switchable per
   project, shipping subject to item 24.
2. Human-message and ask contracts (B2, S1, S2, S6): R1 to R7, the typed envelope for `ha say`
   and `ha ask`, the budget, durable asks with revisions, the compact-plus-full display path, and
   board templates without names.
3. Sealed events and bound receipts (B3, S3): D5's three records and five crash points; D14's
   receipt input through `HERDR_ADE_LAUNCH`; D3's process binding.
4. Git state machine (B4): `B`, `C`, `V`, `H`, merge intent, ancestry, repository lock.
5. Runbook (B5, B6, S4): owners, `go.toml`, classification, create-only backup, staged bytes,
   final fenced check, `RECONCILE`/`RESUME`, rollback states.
6. Lane graph and acceptance (S5, N1): A0 prerequisite, glue owners, required rows, PASS/FAIL/NOT-RUN.

Decisions folded from `tasks/ade/decisions.md`: §0.1 now lists 17 decisions; §6 items 17 to 23
are marked decided with no recommendation wording; item 21 (Rolf types the commands) is the
owner table's Rolf row; item 23 (plugin projects only) closes S6's legacy-publisher question.
TBC-1 to TBC-3 are referenced in §6 as later work and listed in §5.

## Where I am least sure

- **D18's reply path relies on the hook input carrying the finished reply** (`last_assistant_message`
  for Claude per H1; `afterAgentResponse` for Cursor per H2). If an installed version lacks that
  field, the surface falls back to `ha say`/`ha ask` only for that kind; A2's first test on the
  installed versions decides, and D18 item 5's labels already allow that outcome.
- **Hook children inherit `HERDR_PANE_ID`.** The scoping in D17 item 2 assumes the CLIs pass the
  pane environment to hook processes. Standard for child processes, unverified per CLI; A2's
  first test covers it, and a CLI that scrubs it gets `chat: not checked` until another scoping
  key (the session id alone) is verified.
- **R4's word list** is the rule most likely to make a coordinator loop; item 29 is Rolf's for
  that reason, and the budget (S1) bounds the loop whatever the list.
- **The owner table** is a proposal (item 30); Pro accepted any allocation recorded before go.
- **Whether `ha ask`'s compact line at 60 characters is short enough** for Rolf's actual sidebar
  width is unknown to the plugin by design (S6); the full text is always one tab or one `ha
  context` away.
