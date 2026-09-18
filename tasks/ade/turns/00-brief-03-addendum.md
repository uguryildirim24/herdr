# SPEC-ADE turn 03 addendum: item 17, the plain-language layer (Rolf, 2026-09-18)

Rolf, reading the board proposal: "it gets to a point where I don't know what
lineage-persist does or F-cap criterion is ... that's my general problem in
everything, especially software: I'm clueless no matter how plain prose I say ...
there is too much terminology." Two problems, one design answer, no clutter on the
board.

## The two problems

1. The words coordinators use with Rolf are too advanced. A message built from names
   (`wp27`, `F-cap criterion`, `lineage-persist`) carries nothing for him.
2. The board (item 14) would show those same names. A `needs you` line made of terms is
   unanswerable.

## Decision to write into v2 (D17, plus §6 item 17 for what is still open)

- **Every name gets a plain sentence at birth.** A thread (lane), a package, a round and
  a spec term are created with a required `plain:` line, written by the coordinator
  ("makes the lane tree survive a server restart", "the reviewer for this round"). The
  plugin refuses `thread start` without it. It lives in the thread record and in the
  brief header.
- **The board shows the sentence, never the name.** The workspace tokens from item 14
  carry plain text: `needs you: decide when the fly experiment counts as good enough
  and stops` instead of `needs you: F-cap criterion`. The name is one keystroke away
  (`ha context`, `ha explain <name>`), not on the card. Length stays within the
  sidebar width; the plugin truncates the sentence, not the meaning.
- **`needs you` is a question he can picture.** The coordinator publishes it as a choice
  between outcomes ("keep the experiment running another hour, or stop now?"), never a
  term. The plugin stores the question with the record so a later coordinator can
  repeat it verbatim.
- **RULES.md carries the language rule for every coordinator of every kind**, appended
  to every priming line and every brief: first use of a code name in a message to Rolf
  comes with its plain phrase; questions are picturable choices; a status message says
  what a thing does and what it means for him, not the name and the sha. Provider
  agnostic, so it holds for Claude, Cursor, Codex, agy, Pro.
- **A per-project GLOSSARY.md the plugin maintains**: every `plain:` line lands there
  automatically, one line per name, newest last. `ha explain <name>` prints that line
  plus the brief path. Coordinators add a line whenever they mint a term in a spec.
- **What stays for agents:** the names. Agents need stable ids; records, branches and
  tokens keep them. Only the layer Rolf reads changes.

Fold this into v2 as D17 (with tests: `thread start` without `plain:` refused; the
board token carries the sentence; RULES.md appended to every priming line) and list
what is still Rolf's under §6 item 17 (for example, whether the sentence goes into
the tab label too).
