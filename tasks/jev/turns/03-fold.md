# Fold: SPEC-jev-picker v1 → v2

The old "89 of 97" score counted tasks that still named their helper in the title.

Once those names are hidden, always using the usual helper, or a two-line table by job and project, already matches most of the 97.

The picker does not beat that table on this set.

The only clean signal is web research, and that is now its own job, so the tool ships off and first only records what it would have picked.

The shipped shape is yes-or-no gates over your fixed table, not an open pick of every helper.

A stronger helper waits for a stall and your say-so; it is not chosen when a lane starts.

The full launch row is stored once and reused on restart; two linked spec lanes drop a matching helper in code before the second call.

The start of the task is scrubbed and shortened before it leaves this machine, and only for a project you mark.

The call is three seconds with one retry, pinned to this version of the ranking service, through the tool's existing command runner.

A second labelled month must beat the simple table before anyone turns the picker on.

## Questions for Rolf

Each item is a picturable choice. Pick one sentence.

1. When may the picker choose for real?
   - Keep it recording only until I say so.
   - Turn it on for one project after two weeks of recording, if it beats the simple table.
   - Turn it on for every new project after a second month of labels.
   - Never: keep the table only.

2. Who may send the start of a task off this machine?
   - Only projects I mark as allowed; everything else stays off.
   - Every project may send it.
   - No project may send it; do not build the picker.

3. How long should it only record picks on a project before it may launch them?
   - Until thirty lanes or two weeks, whichever is later.
   - For one calendar month.
   - Until it beats the simple table by five matches per hundred, with at most two wrong upgrades.

4. For the second month, what counts as the helper you wanted?
   - Whatever actually ran after any correction you made.
   - Only the pins you typed, plus restarts you asked for.
   - Those, plus your chat notes that named a helper.

5. If a task names a helper in its title or body, what should happen?
   - Warn, then pick as usual from the table.
   - Treat the named helper as your pin for that lane.
   - Refuse to start until you pin it by hand.

6. How should a special helper for a whole round be set?
   - Change the table's usual helper for that job until the round ends.
   - Pin the helper on each lane by hand.
   - Let the picker guess from the brief.

7. When a lane stalls, should the tool ask you to restart it on a stronger helper?
   - Yes, in the same round that ships the recorder.
   - Yes, but later.
   - No, you will restart it yourself.

8. Should a stronger helper ever be chosen when a lane starts?
   - Never at the start; only after a stall, and only if you agree.
   - Yes, if the picker is sure and the helper is one you already allow.
   - Yes, but ask you first and start on the usual helper if you do not answer.

9. If the picker wants a costlier helper at the start, what happens?
   - Start on it when the picker is sure enough.
   - Ask you first; start on the usual helper if you do not answer.
   - Never start on a costlier helper; only offer it after a stall.

10. Besides research, which extra start-time switches do you want without being asked?
    - None yet. Research is already its own job.
    - Spec author: the careful writer versus the other writer.
    - Hard review: the careful reviewer versus the usual helper.
    - Both of those, once we have labels that separate them.

11. What should the board show?
    - A job name only.
    - The job name, and the product name only in the detail view.
    - The product name on the board.

12. May a project narrow the allowed helpers?
    - A project may name a shorter allowed list.
    - A project may only pin one helper for a job.
    - Keep one global list only.

13. What is the usual helper for the second opinion on a spec?
    - The second design helper.
    - The usual coding helper.
    - You will set it each time.

14. Should a review's helper be chosen from the review text?
    - No. The table picks it. The picker is not asked.
    - Yes. The picker may switch a review to the careful reviewer.
    - Only for projects you mark.

15. Build the picker now, or wait until the other providers sit under one shared tool?
    - Build it now against today's helpers; keep a spare column for that later move.
    - Wait until that shared tool exists, then build it.
    - Build the recorder now; wait to turn it on.
