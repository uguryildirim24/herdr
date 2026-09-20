# Lean: what is in excess, what is missing

Rolf, 2026-09-20: "I just want to be lean... I want to implement all of those
and more as it comes up so it's cheaper to use and it's more comfortable to use
for you, the user and the agents."

Written by the coordinator after one night of driving the harness end to end.
Every item below is something that cost time, tokens or Rolf's patience in that
one night; the evidence is named so none of it has to be taken on faith. This
file is the list of what we cover next. Items get struck off here when they land
in a round.

## Why the weight is there in the first place

Rolf's own reading, and it is right: the inbox and several other records were
built to take control out of the model's hands for the parts that do not need a
model. A coordinator compacts, and after a compaction it starts to fumble. The
records are a hedge against that.

So the answer is not "delete the records". It is:

- **One durable copy, derived views.** The state a coordinator can fumble is
  held once, in the binary. Everything a coordinator reads is generated from it.
  A second hand-maintained copy is what goes stale, and a stale copy is worse
  than none.
- **Refuse, do not instruct.** Every rule that today lives in skill prose for a
  model to remember is a rule the binary can enforce. A fumbling coordinator
  should be unable to do damage, not merely told not to.

That is the same fix as most of the items below, which is why they are one list.

---

## E — In excess

### E1. The inbox is a second copy of the digest
176 unhandled items at the start of this session. Every one restated something
`ha context` already prints; about sixty-five were `preparation-abandoned`,
which has never once been needed. It costs tokens on every context read and
carries no information the thread records do not already hold.

**Should be:** the thread and round records are the truth; the digest is
generated from them. An inbox item exists only for something that has no other
home — a message from another machine, a scheduled routine firing. Nothing that
duplicates a thread's state.

### E2. A round's state lives in seven writable places
An event, a pin in a manifest, a brief commit on the integration branch, a
review branch, a verdict file, a merge, a checkpoint commit.

Nearly every failure this project has had is two of those drifting apart:
`head_moved`, `merge_conflict`, `admit` pinning an already-merged sha,
`verdict_commit` taking the wrong done event, `reviewer_already_bound`, a
resolve on a lane whose merge had silently failed.

**Should be:** one record owns the round. The branches and commits are outputs
of it, never inputs that can disagree with it.

### E3. Four ways to start a review, and the automatic one fails silently
`round review`, `round reviewer`, `round advance`, the herdr hook, the ticker.
`advance` failed to start its reviewer **four times in this session alone**
(r28, r30, r32, r33), each time exiting 0 and printing nothing, with
`announced = reviewer-start-failed` and `launch_attempts = 0` left on the
record. Each time a human had to notice and run `thread restart`.

**Should be:** one path. A start that fails says so on the spot, and retries
itself; t-0058's retry work does not catch this case.

### E4. The plain-word check polices writing Rolf never reads as prose
The checker is right for anything that reaches his board or his screen. It also
refuses `ha decide` lines and round sentences for naming a file. Turns were
spent rewording a decision because it contained the word `config.toml`.

**Should be:** check what Rolf reads. The internal log records what happened,
in whatever words are accurate.

---

## A — Missing, agent side

### A1. Machine-readable results from every verb
Almost every `ha` verb answers in prose, so a coordinator greps English to learn
what happened. This is not style: a piped `round merge` hiding its exit code is
what wrongly resolved two threads in this project, and the fix was a rule a
model has to remember instead of a thing that cannot happen.

**Should be:** every verb returns a structured result with an outcome field. The
human sentence is rendered from it.

### A2. Refusals in the binary, not rules in prose
The harness currently allows: admitting a sha already on the integration branch;
resolving a lane pinned in an open unmerged round; opening a round whose brief
moves the head another merge is aimed at. Each of those is a real incident in
this project's history, and each is "documented" as a sentence in the skill.

**Should be:** the state machine is in the binary and it refuses, with a reason
naming what is wrong and what to do instead.

### A3. The harness does not notice its own failures
Every defect fixed tonight — the Mac-shaped zig path on the box, the lying
`box_repo_unmapped`, the silent reviewer start, the second empty key on the box
— was spotted by a person or a coordinator and then hand-typed into a task file.
Four identical failed reviewer starts in one night is a fact the harness already
holds and does nothing with.

**Should be:** a failure ledger. Refusals, retries, silent starts, merges that
needed a second attempt, all recorded as first-class items. A coordinator turns
one into a lane with a single verb, evidence attached. A repeated failure
becomes work without anyone remembering to notice it. **This is the item that
makes the harness self-evolving rather than coordinator-evolving.**

### A4. Memory rewards appending
Left alone, `memory/state.md` grew to 62,511 bytes of chronology, inlined into
every thread brief. Nothing pruned it, nothing warned, and it was pure cost on
every lane start. It was compacted by hand to 3,858 bytes tonight.

**Should be:** memory has a shape that resists diaries — a size budget that is
enforced or warned on, and an archive path that is not inlined.

---

## U — Missing, Rolf's side

### U1. What it costs
There is no number anywhere: not tokens, not money, not minutes. Rolf is asked
to choose between "the cheap helper" and "the strong paid helper" and neither he
nor the coordinator can state the ratio.

**Should be:** cost per lane, per round and per day, on the screen. If one
number goes on that screen, it is this one.

### U2. A verb for overturning a decision
"Decided for you" shows him the choices made without asking. Reversing one is a
sentence he types into chat, hoping the coordinator acts on it. The record
exists; the action does not.

### U3. Questions he cannot dismiss
A duplicate ask was put on his board tonight and could not be withdrawn, because
only the newest open ask may be re-asked. He is looking at a card both of us
know is wrong.

**Should be:** an ask can be withdrawn, and a duplicate is refused at write time.

### U4. Staleness is invisible
This was most of his frustration tonight, and none of it was broken code — it
was correct code that was not running anywhere he could see:

Everything that can be older than the program, found by hitting each one:

1. **A talk shell** (`ha talk <slug>`), one per project — stale after every
   plugin install. Four of them were nine hours and ~25 installs old.
2. **His client window** (the `herdr` process) — stale after every fork install.
   It was running the pre-r27 binary while the fix sat on disk.
3. **The Mac's herdr server** — stale after every fork install; only a live
   handoff moves it, and that is Rolf's call. Still on its 13:07 image.
4. **The box's herdr server** — the same, on the box.
5. **A coordinator's loaded skill text** — a merge that changes
   `skill/COORDINATOR.md` does not reach a coordinator that is already running.
   r25 changed it tonight; this session still holds the copy it read at start.
6. **A running lane's brief and skill** — the same problem, per lane.
7. **pi's provider config** — `herdr-pro serve` picks a fresh port on every
   start, so `herdr-pi setup` must run again or pi points at a dead port.

The harness knows every one of those versions.

He hit all four and had to ask why a finished fix was not there. The harness
knows every one of those versions.

**Should be:** the screen says *this window is older than the program* and names
what to restart. Nobody should discover staleness by being confused.

### U5. The task list is not on the screen
`TASKS.md` is the list of what Rolf wants done. Today it exists only inside the
coordinator's digest, so the one place he looks does not show it. Rolf,
2026-09-20: "I want to see the task md on the talk page."

**Should be:** the screen shows the open tasks, grouped by list, each with its
owner and — for a delegated one — the state of the thread doing it. Adding,
finishing and cancelling stay his words in the chat; the screen is where he sees
what is on the list without asking.

---

## M — Model choice, from Rolf, 2026-09-20

This reverses the decision of 2026-09-19 13:40 ("no Jev picker, we don't need
the classifier"). Rolf, tonight:

> "Whenever you're dispatching something we should make Jev pick the model...
> I don't think there should be roles, just for the simple fact that it's a very
> hard problem — sometimes you cannot classify anything. One of the agents
> failed and I wanted to use a stronger model, but it was 'oh, I cannot edit the
> config'. I don't want that. When it sees that it failed, it just calls for
> another model which is better."

> "[Review] is part of the root flow, I understand, but it should not be a role,
> if that makes sense."

### M1. No roles
The roles table fixes a model per named role, chosen up front by a coordinator
guessing what kind of work this is. That classification is the hard problem and
it is being solved at the wrong moment, by the wrong thing.

**Should be:** the work itself decides. At dispatch, Jev reads the task and the
state and returns the model. There is no role name to pick and no table to
maintain.

### M2. Failure escalates on its own
A lane that fails should reach for a stronger model, not stop and wait for
someone to edit a settings file. No config edit, no permission dance, no
`escalate` list that a coordinator is forbidden to touch.

**Should be:** failure is an input to the same picker. It picks up.

### M3. Review stays in the flow, "reviewer" stops being a role
Checking work before it lands is part of the root flow and stays. What goes is
the fixed `[roles.reviewer]` row that decides the model before anyone has seen
the work. The review's model comes from the same picker, reading what is
actually being reviewed.

---

## D — Known defects carried in

- **D1.** `round advance` leaves `reviewer-start-failed` with `launch_attempts = 0`
  and prints nothing; a human runs `thread restart`. Four times this session.
- **D2.** Lanes push their branch to origin although the brief says not to
  (t-0058, t-0071). Two lanes ignoring the same line means the line is not where
  they look.
- **D3.** The box's `herdr-pi doctor` probes with zsh's `whence` through a bash
  login shell, so one row fails on the box; one test scripts only the zsh probe.

---

## Order

A3 first: the failure ledger is what turns everything after it into work the
harness raises by itself rather than work someone has to remember. Then A2 and
A1, which are what make a fumbling or freshly-compacted coordinator harmless.
Then E1 and E2, which are only safe to thin once the binary refuses. U4 and U1
are cheap and are what Rolf feels every day. M is its own design turn.
