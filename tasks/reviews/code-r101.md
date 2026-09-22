+++
verdict = "MERGE"
round = "r101"
candidate = "0acfb063b6d3f81ff2e844008c2ec6103b473da5"
manifest_hash = "d2c84328c37ee82c0027301b5f7f94aafd007f6282c507eeceda5534df05823a"
policy_hash = "d9b7ce2a8a3a9102c2ed533f14bca3988e9f54acbf667b744058b12959db943a"
gates = []
+++

# Round r101 verdict

## t-0248 — MERGE

The agy rules now require affirmative evidence instead of treating every unmatched frame as a completed turn. A braille spinner with an activity label and the queued-message hint are working evidence. The bare composer plus shortcut footer is the only idle evidence, and the catch-all unknown rule prevents the known-agent idle fallback from marking an intermediate frame done.

Unknown does not satisfy Herdr's default agent wait, so an intermediate frame deliberately keeps a wait open. The ordinary completed agy frame in the captured evidence matches `empty_prompt_idle`, transitions to idle, and releases start and prompt waits. If that evidence never appears, callers wait or reach their configured timeout rather than reporting unfinished research as complete.

The bundled and published files are byte-identical (SHA-256 `0319c44625e93a6305a816bf508ed6f9551f275cd4bfbdcab00d9382c6fd9fc4`), and the published catalog already maps `agy` to `antigravity.toml`.

A hosted published manifest changes a running server after its automatic or manual manifest update fetches it: the updater reloads the affected cache entry and resets matching detection runtimes. No server restart is needed. Committing the published file alone does not alter a server until that fetch occurs; the bundled copy requires a binary carrying it.

The focused detector test and manifest publication validator pass. The requested full `cargo test` remains red on both candidate and base because the existing parallel suite has unrelated global-state failures. The candidate additionally reproduced the reported SIGPIPE twice in four runs; eight base runs did not reproduce that exact symptom. The diff does not touch CLI output or signal handling, and the focused changed test passes, so this is a branch-sensitive exposure of the existing full-suite instability rather than a defect in the detection rules.
