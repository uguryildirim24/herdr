# Code review + fix — <PACKAGE> on branch <BRANCH> (<REVIEWER MODEL>)

You are the reviewer for this repo. A worker finished package <PACKAGE> on
branch `<BRANCH>` (worktree `<DIR>`). Your job is to inspect it against
`<SPEC>` and **fix what is wrong yourself**, then commit your fixes on the
same branch.

1. `cd <DIR>`. Read the package brief `<BRIEF>` and the spec sections it
   cites. <MERGE OR DIFF LINE>
2. Run the gates: <GATES>. Use <BUILD DIR VAR>=<DIR>/../../.target/rev-<pkg>.
3. Adversarial review: contract conformance (names, types, error variants,
   exit codes, schemas), correctness under the cases the brief lists,
   security (credential handling, redaction, containment, nothing sensitive
   written to disk), the minimum supported toolchain, and dependency pins.
   Missing tests the brief requires count as defects. <ATTACK POINTS>
4. Fix every defect you find directly in the code. Keep the worker's
   structure unless it is wrong. Commit each logical fix separately with a
   message starting `review(<PACKAGE>):`. Do not push. Do not merge.
5. If something cannot be fixed without a spec change, do not guess: leave it
   and list it under "Needs a decision".

Output: `docs/reviews/code-<PACKAGE>.md` with: a 3-line verdict (MERGE /
MERGE-AFTER-DECISION / REJECT), the gate results, a table of defects found
(severity, file:line, what was wrong, what you changed, commit hash), and
"Needs a decision" items. Then reply exactly: `DONE docs/reviews/code-<PACKAGE>.md`

The worker's final report, for reference:

<WORKER REPORT IN A FENCED BLOCK>
