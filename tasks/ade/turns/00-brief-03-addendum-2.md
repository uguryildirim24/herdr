# SPEC-ADE turn 03 addendum 2: item 17 is enforced by the binary, not by rules

Rolf, 2026-09-18, after the first addendum: "your rules don't work as a memory or as
claude.md, I have tried, believe me ... there are two separate planes here: what I,
the user, see, and what you or any coordinator does with other lanes. That can be
another language or filled with terminology as far as I care. It's only what is
communicated to me I have to understand, and I want this in the binary somehow."

This replaces the RULES.md line of the first addendum. A rule that lives in a model's
context is not a mechanism. The mechanism is a check that a message cannot pass until
it is plain, run by the plugin (Rolf's binary), for every coordinator kind.

## The two planes

- **Agent plane**: coordinator to lanes, lanes to coordinator, briefs, reports, records,
  specs, tokens for agents. Any language, any terminology. Untouched.
- **Rolf's plane**: everything that reaches his eyes. The coordinator's replies in its
  own pane (he types there and reads there; that stays his interface), the board
  (item 14), notifications and toasts, `needs you`, anything a coordinator "asks" him.
  Only this plane is gated.

## D17 as a mechanism (write this into v2)

1. **A deterministic plain check** in the plugin, `ha plain check` (also a library
   function used by every publish path). No model involved. It fails a text that:
   - contains a name from the project registry (thread names, package ids, branch
     names, round ids, error codes, config keys, file paths, 7+ hex digit ids)
     without that name's `plain:` sentence in the same message;
   - contains an identifier-shaped token (snake_case, CamelCase, kebab-case with a
     digit, ALL-CAPS acronyms not in the glossary) that is not in GLOSSARY.md;
   - has a sentence over the configured length, or a question that is not phrased as
     a choice (`ha ask` below);
   - is a status line without a "what this means for you" clause when it ends a turn
     that carries a decision for Rolf.
   The word lists are files the plugin ships and the project extends (GLOSSARY.md);
   the check prints exactly what to change, so the model can fix it without guessing.

2. **Gate A, the coordinator's replies, per kind, installed by the plugin.** The plugin
   already provisions per-kind hook assets (v1 D14 counted seventeen). For every kind
   with an end-of-turn hook (Claude: `Stop` hook returning `decision: block` with the
   check's reason; Cursor and Codex: their equivalent stop/after-response hooks; the
   roles table says which kinds have one), the coordinator role's turn cannot end until
   `ha plain check` passes on its final message. Lane roles are exempt (agent plane).
   The hook is installed by `ha` when the coordinator thread starts, in the project's
   hook config, and removed when it ends. Rolf never edits a rule; the model never
   remembers one; the turn just does not end. Kinds without a hook (agy, chatgpt) fall
   to gate B only, and the roles table marks them "not gated in chat".

3. **Gate B, the binary-owned surfaces.** Every publish path in the plugin (board
   tokens, `needs you`, notifications, `ha ask`, `ha say`) runs the same check and
   refuses text that fails. So the board and the toasts are plain by construction,
   whatever the coordinator's kind.

4. **`ha ask`**: the one way a coordinator asks Rolf something. It takes a question
   and two to four choices, each a sentence he can picture ("keep the experiment
   running another hour" / "stop it now"), never a term. It publishes to the board's
   `needs you`, sends a notification, and records the question so a later coordinator
   repeats it verbatim. The check applies to the question and every choice.

5. **Translator fallback, provider agnostic**: after three failed checks in one turn,
   the hook may call the configured plain model (`[plain] model = "<any CLI in
   print mode>"`) to rewrite the failing sentences, then re-check. Off by default;
   Rolf turns it on. It never replaces gate A; it helps the model pass it.

6. **Names at birth** stays as in addendum 1: `thread start` without `plain:` is
   refused; the sentence goes to the record, the brief header and GLOSSARY.md.

## Tests to name in v2

- A Claude coordinator's final message containing `lineage-persist` without its
  sentence is blocked; with the sentence it passes; a lane's message is not gated.
- `ha ask` refuses a term as a choice; accepts picturable sentences; the board token
  shows the question.
- Board token text that fails the check is not published and the old value stays.
- The hook is present in the project's hook config while the coordinator thread runs
  and absent after `ha done` on it.
- A kind without a hook is marked in `ha context` as "chat not gated".

## Still Rolf's (§6 item 17)

- Sentence length limit and the shipped plain-word list: defaults to propose.
- Whether the plain sentence also becomes the tab label.
- Whether the translator fallback is on from day one.
