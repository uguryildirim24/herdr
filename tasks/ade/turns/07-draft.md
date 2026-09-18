# Turn 07: SPEC-ADE v3.1, the fold of Pro's last look (turn 06)

Lane spec-ade, pane w1F:p8, 2026-09-18. Input: `tasks/ade/turns/06-pro.md` (`converged: no`; T1,
R1, R2, R3 blocking outside §3; M1, M2 migration no-go; S1 to S3 should; §3 change list; §4
readings for items 27 to 31; §5 items 32 to 35). Output: `tasks/SPEC-ADE.md` v3.1 in place (1580
lines, was 1363). Nothing rejected; no fork feature added; no Jev, cloud or streaming work pulled
in. Nothing ran against the live server.

## What changed from v3 to v3.1

1. §3.3 step 6: the `cp` plus `noclobber` backup is gone; `cp` ignores `noclobber` (ZH, CP). The
   swap is one tested state-dependent script with an exclusive-create backup branch (absent:
   create from verified bytes; present with the source hash: reuse; anything else: stop), a fresh
   `mktemp` staging file, and a read-the-state-first rule so the retry Pro named writes nothing.
2. §3.1, §3.3: one maintenance epoch in `go.toml`; `WAITING` pushed in the window is a fact class
   with a named recovery source (the lane's own conversation, read by its coordinator through
   one `STATUS` control message); step 9 reconnects connections only; a reversal happens inside
   the held epoch or a fresh one; M1 and M2 are written as no-go conditions until tested.
3. D5: the op carries the complete requested payload, helper pid, revision and a fixed event id
   (the op id); seal is create-if-absent with byte equality; new crash point X2b; a dead helper
   is `preparation-abandoned`, a retry resumes or supersedes (item 32).
4. D5, D6: the admission manifest with its revision and frozen policy is authoritative and fails
   closed; only completion links are a projection; a later admission bumps the revision and
   makes the review stale (item 33).
5. D6: head, checkout and ancestry re-validated under the repository lock at the effect
   boundary; a checkpoint intent bound to `V` with the HANDOFF payload hash precedes `H`;
   recovery finishes `V` to `H` or repairs a verified `H`; only `checkpointed` is a no-op (item 34).
6. D17 item 3, D18 item 3: one typed publisher (`Say`, `Ask { id, revision }`, `Notice`); the hook
   forwards only `ade-say` and `ade-ask` envelope blocks, prose stays private, a prose question
   can never be the question a number answers (item 35, T1).
7. D18 items 2, 4, 6: talk input has a request id and `queued`, `submitted`, `uncertain`,
   `accepted` states, never auto-retried; the ask binding is frozen while a number is typed;
   `!native`/`!back`; fixed notices for dialogs, session changes and hook failures; one journal
   append owner with fsync and tail handling; switching only at `ha open`/`ha close` (S1).
8. D8: the single writer is the plugin's own writers only; the fork's `BLOCKED`/`GONE` and a
   person in the native tab are outside it (S1).
9. §1.1, D17 item 2: Claude's documented `MessageDisplay` hook recorded as a fail-open display
   transform, unqualified, not used this round (S3); scrollback does not survive a handoff
   (`tasks/SPEC-durable.md:79,360`), so it is not a recovery source.
10. §4.2, §4.3, §1.3: A0 owns the new contract types, A1 the swap script and its test; row 4(b)
    asserts no submission while blocked; row 6 observes `V` at an instrumented stop and `H` at
    completion; row 10 reads `talk` with `pane read` under a hook barrier; row 8 records
    versions and credential availability; the item 32 to 35 fixtures are tests (S2).

## For Rolf

What still needs you, in plain words:

- **Item 24**: does the talk tab (the plugin's own conversation window, where only checked
  replies appear and where you type) ship in the first plugin round? My recommendation: yes, on
  by default for a Claude coordinator, off for the other kinds until their hooks are verified.
- **Item 27**: when the talk tab is on, you still go to the coordinator's own pane for
  permission and trust dialogs and for slash commands, and for nothing else. Confirm.
- **Item 28**: the limits on how often the plugin makes the coordinator rewrite a reply (three
  tries, one optional translator run, ten minutes). Confirm.
- **Item 29**: the word list the plain check uses: a checked-in everyday-English list of about
  five thousand words plus the plugin's own words, grown only through the project glossary.
  Confirm, or name a list you prefer.
- **Item 30**: who does what during the migration: you type the install and handoff commands,
  `hcoord` runs the barrier, the capture, the final check and the three control messages, each
  build's coordinator checkpoints and reconciles its own lanes. Confirm.
- **Item 31**: after an emergency reversal to the old server, the new command-line binary stays
  installed and is swapped back only if it is itself at fault. Confirm.

Items 32 to 35 are build questions with their readings and tests written in; they are not
yours. Two things are not questions but conditions: the migration does not go until the binary
swap script has passed its retry and failure tests on your Mac (M1) and until every kind of
message a worker might send during the bounce has a named way to get it back (M2).
