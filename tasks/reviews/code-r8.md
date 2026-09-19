+++
verdict = "MERGE"
round = "r8"
candidate = "27453e25e79fde2adc7b48431ca334af21f8233c"
manifest_hash = "b06cd93406bad1a1d215d4746271f2b63b547db6edaf292efd601b3b62a96414"
policy_hash = "e1529634be8dfdf8688cad0bfcfd99dfa67f7f19fe2d1c3c7ab9f12ee7c0eec3"
gates = []
+++

# Round r8 review

## t-0016 — the talk tab plan

MERGE. The plan now matches SPEC-ADE D18 and the plugin and fork code it cites.

I verified both named causes in the plugin: an ask can be published first under `ask:<id>@<revision>` and then under the hook's `hook:<session>:<turn>:<position>` key, and the current prompt is redrawn only after input while idle journal updates reuse its old text. The proposed canonical ask key and event-time choice check address those causes.

The review commit corrects the parts that did not match the code or could not be accepted as written:

- conversation rows, live header facts, project data and colours now have distinct sources;
- running and waiting lane counts use the thread groups, including recorded remote state;
- the capability label uses the project's evidence-backed adapter qualification;
- recorded but not yet journaled asks are not confused with partially published asks;
- the key map reflects Rolf's current config and herdr's actual direct bindings;
- the narrow layout is defined from pane width and explicitly covers herdr's 64-column mobile switch;
- the unreachable-server drill leaves the talk pane attached, so it can be checked by eye;
- exiting the alternate screen must restore terminal modes, and restart instructions no longer claim that `ha open` restarts a stopped command in an existing talk pane.

All journal fields mapped to the screen exist in `talk.rs`, `contracts.rs` and `ask.rs`; the new answer reference is explicitly a schema addition. The three referenced pictures and their prompt files exist under `tasks/ade/talk-mockups/`. The four choices that still need Rolf remain together in section 9, each with one checked `plain:` line. They do not block merging this planning draft before its two independent checks and fold.

## Document checks

The round record lists no project gates, so the front matter keeps `gates = []`. I also ran the document checks required by the review task:

- each of the four `plain:` lines: `pass`;
- all three referenced PNG paths: present, each a 1536×1024 PNG;
- the pinned lane diff and the review-fix diff: only paths under `tasks/ade/`.

No code was built, as required for this document round.
