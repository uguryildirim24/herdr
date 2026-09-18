# SPEC-ADE turn 03: dispositions of Pro's attack, and v2 in prose

Lane spec-ade, 2026-09-18. Spec: `tasks/SPEC-ADE.md` v2 on `lane/spec-ade`. Pro's turn:
`tasks/ade/turns/02-pro.md` on `agent-parent-nesting`. Rolf's addendum (item 17, the
plain-language layer) arrived mid-turn and is folded in as D17.

## Dispositions

43 rows: 6 accepted, 37 accepted with change, 0 rejected. No rejection had a citation Pro missed.

| # | Pro's verdict | outcome | reason | changed |
|---|---|---|---|---|
| D1 | accept the rename, specify the full tuple | accepted with change | one resolved tuple (id, binary, prefix, root env, config, ticker lock, startup) printed by doctor; old ticker stopped before unlink; GitHub fork is a destination, not a fact | §1.2 D1 |
| D2 | config resolution is not a safety boundary | accepted with change | replace-not-merge, kind change needs args, unknown fields refused, `ready_timeout_ms`, launch recipe on the record, no `bootstrap` field, cooperative language | §1.2 D2, §1.3 |
| D3 | reconciliation useful, ownership claim too strong | accepted with change | parent at launch through `AgentStartParams.parent` (r2); plugin `Agent` gains `tokens`; repair only after identity verification; `lineage-mismatch` item; 15 s is not a bound; D3 is a reconciler | §0.4, §1.2 D3 |
| D4 | port every lifecycle path | accepted with change | restart, resolve, removal gate, partial-failure record, common-dir lock, no trust-avoidance claim | §1.2 D4 |
| D5 | record and outbox authoritative before sending | accepted with change | immutable event under the lock first, projections derived, submitted/acknowledged/handled, at-least-once, report-hash is `report-available`, `waiting` from dirty tree | §1.2 D5, §1.3 |
| D6 | bind review and merge to immutable inputs | accepted with change | manifest with pinned attempt/sha/hash, fail closed on unreadable records, brief commit before review branch, `verdict` field with `candidate`, CAS `update-ref` only when not checked out, two-phase merge/checkpoint, no idle-as-release | §1.2 D6, §1.3 |
| D7 | adopted Pro is not an ordinary worker | accepted with change | `--passive`, validated recipient, pinned turn, artifact event, commit verifies file, no live-build adoption command | §1.2 D7 |
| D8 | reject "no duplicates" | accepted with change | fork pushes advisory, no dedup, delivery ownership rule, half-typed text stated as unfixed | §1.2 D8 |
| D9 | temporary index is not checkout isolation | accepted with change | brief committed before the worktree; three checkout cases; `ha pickup` not `resume`; launch intent before launch; only the ticker launches; no respawn after handoff | §1.2 D9 |
| D10 | preserve the rule without claiming enforcement | accepted with change | cooperative and bounded; transitions from validated records only; long fences; argv not eval | §1.2 D10 |
| D11 | change the security claim | accepted with change | best-effort change notification; policy hash on decisions | §1.2 D11 |
| D12 | "never in git" conflicts with inlining | accepted with change | rules at runtime through `ha skill`; reserved budget; `rules_too_large`; one publishable `## Language` section is the exception (D17) | §1.2 D12, D17 |
| D13 | limit new ADE guarantees to local | accepted with change | rounds and closing verbs local; `remote_not_admissible` | §1.2 D13 |
| D14 | one write is not priming proof | accepted with change | attempt-bound receipt via `ha skill`/`ha context`; re-send only with positive evidence; O3 scoped to one Claude case | §2.2 D14 |
| D15 | qualify each adapter, no blind timed input | accepted with change | matrix marked untested per row; no 512-byte timer; agy re-send needs evidence; Pro needs pro-mcp; dsh `--env` is new code; no eighth kind named | §2.2 D15 |
| D16 | accept the small fork boundary | accepted | unchanged; the fork supplies no durable notification contract and the spec says so | §2.2 D16 |
| M1 | remove every stop/start fallback | accepted with change | no stop/start anywhere; the rebase review's fallback named as not inherited | §3 head, §3.3 rollback |
| M2 | checkpoint barrier instead of an unannounced bounce | accepted with change | steps 1 to 4: notify, four verified checkpoints, fresh inventory, no in-flight launch or prompt, acknowledgements | §3.3 |
| M3 | no live-build adoption, no premature deletion | accepted with change | nothing adopted, no adoption command, no deletion condition | §0.1, §3.3, §5 |
| M4 | one reviewed r2 target | accepted with change | single bounce 0.9.0 → `C`; one-time re-link for every live build | §3.2 P1, §3.3 |
| M5 | binary swap has a missing-path interval | accepted with change | `cp -p` backup verified, then cargo's rename or `mv` over the existing path | §3.3 step 5 |
| M6 | handoff success is not a recovered workflow | accepted with change | reconcile step before any wait or prompt; screenshot gate for chatgpt; no blind replay | §3.3 steps 7 to 9 |
| M7 | rollback exceeds the evidence | accepted with change | reverse handoff qualified at 21 panes by the r2 reviewer before it is a procedure; before-commit failure verified by socket ownership | §3.2 P2, §3.3 rollback |
| M8 | unlinking does not make records inert | accepted with change | `ha ticker stop` for the root before unlink; records preserved | §3.3 step 11 |
| Q1 | DONE recorded, line lost, context blind | accepted with change | event before anything; context shows round membership, sha, delivery state; seen is not handled | D5, D6 |
| Q2 | what two panes did not cover | accepted with change | P2's representative topology; inherited environment stated | §3.2 P2, §3.4 |
| Q3 | which first prompt is lost | accepted with change | no eighth kind; receipt gap is universal; matrix plus receipt | D14, D15 |
| §6.8 | D3 race | accepted with change | race stated; index rebuild is the fork's; late parent does not replay BLOCKED | D3, D8 |
| §6.9 | delayed DONE and round review | accepted with change | review reads the manifest and records; a later line changes nothing | D6 |
| §6.10 | 21 panes and mixed kinds | accepted with change | P2 topology; public pane id vs `terminal_id`; chatgpt not proven by O6 | §3.2, §3.4 |
| §6.11 | what plain git lacks | accepted with change | orchestration guarantees listed in the D4 gate; branch left behind is recovery material | D4 |
| §6.12 | is one typed line enough | accepted with change | transport yes, acceptance no; receipt and per-kind rows | D14, D15 |
| §6.13 | explicit import path | accepted with change | explicit absolute verified path for the first bounce; parameterless later is fine | §3.3 step 6 |
| R1 | shape Q1 to Q9 as decided | accepted | "assumed" removed from normative text | §0.1 |
| R2 | rename as decided | accepted | D1 tuple | §0.1, D1 |
| R3 | roles location as decided | accepted | D2 wording on permissiveness | §0.1, D2 |
| R4 | after-verdict as decided | accepted | no human merge hurdle; stronger automatic path | §0.1, D6 |
| R5 | reports ignored plus home copy | accepted | hash-verified copy before done and removal; rules not committed through briefs | §0.1, D4, D5, D12 |
| R6 | live builds finish on skills, draft must change | accepted with change | all four builds; no adoption; no deletion condition | §0.1, §3, §5 |
| R7 | notify after clean r2, checkpoint first | accepted with change | barrier steps; post-bounce reconcile and notification | §3.3 |
| 14 | Spaces board, (a) first | accepted with change | plugin tokens plus sidebar rows; sentences not names (D17); (b)(c)(d) deferred or not chosen | §6 item 14, D17 |
| 15 | what acknowledges a bootstrap or event | accepted with change | normative in D5 and D14 with the test list | D5, D14, §1.3 |
| 16 | which git state a MERGE authorizes | accepted with change | normative in D6 with the test list | D6, §1.3 |

## The design in prose

The plugin is still the answer, and the fork still changes for nothing beyond round r2. What
changed between v1 and v2 is that five contracts I had folded into one are now separate: process
continuity (the handoff keeps pids and ids), metadata restoration (the r2 fork keeps tokens across
durable handoffs; the first bounce from 0.9.0 is repaired once by a reviewed script), prompt
submission (the PTY accepted the bytes), task completion (a validated event with a sha and a report
hash, written before anything is sent), and permission to merge (an exact verdict for a pinned
candidate). Each has its own record and its own test list.

The lane's closing verbs write an immutable event under the project lock first and derive tokens,
inbox item and the typed line from it. The ticker resends an unacknowledged event with the same id
when the coordinator is ready; the coordinator's next `ha context` is the acknowledgement. Rounds
pin lanes by attempt, sha and report hash; the review brief is committed before the review branch
exists; `ha round merge` is a compare-and-swap on the integration head followed by a two-phase
checkpoint. Priming stays one typed line per kind, but a launch attempt is acknowledged only by the
first plugin call from that pane, and a re-send needs positive evidence from one screen read.

The migration is one bounce from the installed 0.9.0 to the r2 candidate, after the r2 reviewer
has qualified that exact binary on a 21-pane throwaway with two ChatGPT panes, an unnamed
coordinator and a plain watcher. Before it, four coordinators checkpoint and acknowledge; the
inventory is captured fresh; the swap keeps the installed path present at every instant; after it,
every live build is re-linked by script and each coordinator reconciles against its checkpoint
before re-arming a wait. Rollback is a live reverse handoff or nothing. No build is adopted, no
skill is deleted by this spec.

Rolf's addendum adds the layer he reads: every thread, round and term is born with a plain
sentence; the board shows sentences and picturable questions, never names; the language rule is
the one publishable section of his rules file and rides every priming line; the plugin keeps a
glossary and answers `ha explain`.

## What I am least sure of

- Whether the acknowledgement through `ha context` is strong enough: it proves the coordinator's
  turn ran the digest, not that it read the line. Pro's item 15 asked for "an explicit matching
  protocol receipt"; the digest call carrying the pane id is the cheapest such receipt without a
  new command family.
- The representative-topology qualification asks the r2 reviewer to start two `chatgpt` panes on
  a throwaway. If `pro-mcp start` cannot run there, the fallback (`terminal-browser` with
  `HERDR_AGENT=chatgpt`) tests survival, not the composer.
- D17's `needs-you` check is a question mark and a glossary lookup. That is cooperative; the
  clarity of the sentence is still the coordinator's.

## Left for Pro or Rolf

§6 items 17 to 23.
