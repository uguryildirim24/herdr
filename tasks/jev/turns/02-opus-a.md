# SPEC-jev-picker v1, check a (turn 02)

Checker `jp-a`, Claude Opus 5 at effort high, 2026-09-18. I started from the experiment and
worked out to the design. I read the spec v1, the drafting report and brief, the experiment code
and data, the TypeSafe skill and live docs, the venator probe `240fb49`, SPEC-ADE v3 D2, D7 and
D17, decisions 3 and 19, TBC-1, the two memory notes, and the plugin at `a4cdb0a`. I re-ran Jev
live: 388 requests, 841,465 input tokens, **$0.0353**. The code, the raw answers and the summary
are on jevtest branch `experiment/lane-picker-check-a` at `6f4ca81`
(`experiments/lane-picker/check_a.py`, `check_a_analyze.py`, `results/check_a_raw.jsonl`,
`results/check_a_summary.json`). I did not touch `experiment/lane-picker`.

## 1. Plain summary for Rolf

1. The spec's "Jev matched your choice 89 times out of 97" is too good. Almost half the briefs still named the model in their title, so Jev was partly copying it.
2. With the model names hidden, and the choices you would offer today, Jev alone did slightly worse than always picking your default: 74 against 76.
3. One question was clearly right: "is this web research?" It found all six research lanes and never flagged any other lane.
4. My suggestion: keep your fixed table, and let Jev switch a lane away from the default only when a narrow yes-or-no question like that one says so.
5. Stronger models should come in when a lane stalls, not at the start, which is what your own note says. The spec needs five more things before anyone builds it: a complete launch row, a place for the per-project list, plain sentences that pass the checker, a rule for spec pairs, and one pick per lane that restarts reuse.

## 2. Findings

Legend: **blocking** (the spec's conclusion or a builder's work is wrong without it), **should**,
**nit**. "v0" is the published pass; "v1", "v2", "v3" are my re-runs (§2.1).

### 2.1 What I re-ran

| run | brief text | options offered | questions |
| --- | --- | --- | --- |
| v0 (published) | start lines stripped | 11 recipes, three of them historical | 7 |
| v1 | v0 + model and CLI names redacted | same 11 | recipe Choice only |
| v2 | redacted | the spec's 8 day-one recipes, criteria without dates | recipe Choice only |
| v2 repeat | identical request to v2, sent again | | |
| v3 | redacted | 11 | the 7 v0 questions, v0 combiner (Design B) |

Redaction (`check_a.py:33-53`) replaces model ids, vendor names, CLI names, effort words and
permission flags with `[agent]`; one `CODEX_HOME` mention survives, which is task content. Labels
for v2 map to today's equivalent (Grok high → Grok extra-high, Opus xhigh → Opus high), which is
the spec's own "family" view. Cost: v1 $0.0086, v2 $0.0077, v2 repeat $0.0077, v3 $0.0114.

| run | exact recipe | kind | non-default hit | always-default baseline (recipe / kind) |
| --- | ---: | ---: | ---: | ---: |
| v0 A (published) | 71/97 | 89/97 | 11/36 | 61 / **78** |
| v0 B (published) | 66/97 | 79/97 | 17/36 | 61 / 78 |
| v1 A, redacted | 64/97 | 82/97 | 4/36 | 61 / 78 |
| v3 B, redacted | 53/97 | 67/97 | 6/36 | 61 / 78 |
| v2 A, redacted, day-one set, no floor | 74/97 | 78/97 | 5/21 | **76** / 78 |
| v2 A, floor 0.55 | 80/97 | 83/97 | 4/21 | 76 / 78 |
| v2 A, floor chosen by leave-one-out | 79/97 | | | 76 |
| static table + one `web_research` gate (Design C, from v3 answers) | **82/97** | 84/97 | 6/21 | 76 / 78 |

### 2.2 Blocking

**B1. The kind headline is compared against the wrong baseline.** §4 (lines 119-123) and summary
line 7 put "kind 89/97" next to a baseline of 61/97. 61 is the recipe-level baseline. At kind level,
always answering `cursor` scores 61 + 15 (Grok high) + 2 (Kimi) = **78/97**
(`data/label_summary.json`). The honest kind lift is +11, not +28. Evidence:
`check_a_summary.json` `baseline_orig_labels = {recipe 61, kind 78, family 76}`.

**B2. The label leaks into Jev's input through the brief's own title.**
`collect_labels.py:167-175` drops only lines containing `herdr agent start` or starting with
`Start line`. 46 of 97 stripped briefs still name the labelled model, usually in the title:
`WP11.md` "(lane w11, Cursor Grok 4.6 high)", `WP13.md` "(lane lv, Cursor Kimi K3 high)",
`WP17.md` "(lane w17, Codex gpt-6-astra high)", `review-r11.md` "(Claude Opus 5, high effort)",
`WP5-contacts.md` "You are an Antigravity lane because this package is web verification". With
names redacted and the options unchanged (v1):

- Design A: 71 → 64 exact, 89 → 82 kind, non-default 11 → 4 of 36. Astra 2/2 → 0/2. Opus
  reviewers 4/9 → 0/9.
- Design B (v3): 66 → 53 exact, non-default 17 → 6 of 36. Kimi 2/2 → 0/2, Astra 2/2 → 0/2.
  Only Antigravity survives, 6/6.

On the 51 briefs that never named their label, v0 A scored 46 against a baseline of 44. The
spec's "B recovered both Kimi exceptions and both Astra engine lanes" is Jev reading the title.

**B3. In the shape the spec would ship, Design A alone is below the default.** v2 is redacted
text, the spec's own 8-row day-one table (§2), and labels in today's terms. Always Grok
extra-high scores 76/97. Design A with no floor scores **74/97**. It becomes useful only with a
floor: 0.55 gives 80/97, a leave-one-out-chosen floor gives 79/97, and tuning on one project and
testing on the other gives +2 (elicio, 40 vs 38) and +1 (flyonenomics, 30 vs 29). At 0.55 the
four correct non-default picks are three research lanes and the WP24 spec author. "Ship A"
(§4 line 132) rests on the leaky numbers.

**B4. The 0.50 floor comes from the docs example, not the data.** The confidence page
(`docs.typesafe.ai/confidence.md`) uses `if confidence < 0.5` as an illustration and says "the
correct threshold values depend on your domain". On the spec's own v0 answers a 0.50 floor
*lowers* exact agreement from 71 to 68. It drops four correct non-default picks at 0.45 to 0.49:
`review-r6` 0.46, `review-r7` 0.49, `WP19` 0.47, `WP14` 0.45. It rescues one. Confidence
"summarizes distribution concentration" (skill, "Compose and verify"), so its scale depends on
the number of options. A floor tuned over 11 options says nothing about a role with 3 allowed
rows. The floor has to be per allowed set, recorded in `policy_hash`, and re-tuned when a set
changes.

**B5. The output row is missing half of D2's row.** D2 (`SPEC-ADE.md:274-277`) defines a role
row as `kind`, `args`, `env`, `ready_timeout_ms`, and the launch record adds `policy_hash`,
`attempt`, `brief_hash` (`:281-283`). The recipe table (§2 lines 63-72) has only `kind` and
`args`. `opencode_muse` has the placeholder `<installed muse id>`, and `chatgpt_pro` has no args
at all. A builder would invent `env`, the per-kind ready timeouts, the Muse model id, and the
criteria text Jev reads. The v0 criteria live in `run_experiment.py:31-43`, not in the spec, and
they are the actual model input.

**B6. The config shape conflicts with D2 and decision 3, and the per-project list has no home.**
D2 has one inline row per role, `[roles.<name>] kind/args/env/ready_timeout_ms`; the spec adds
recipes plus `default` plus `allowed` without a migration rule. D2 also says "unknown fields are
rejected". §4 says "Put Sol/Astra in `allowed` only on projects where Rolf wants that upgrade",
but Design item 5 and the touch-point row for `src/project.rs` say "PROJECT.md still overrides
`kind`/`args` only". Decision 3 (`tasks/ade/decisions.md:11`) grants only that per-project
override. Where the per-project allowed list lives is undecided, and it is the main lever the
spec hands Rolf. Paste-ready shape in §3.2.

### 2.3 Should

**S1. The input is the `thread start` task, not a committed brief.** Today `thread start` takes
the task text as an argument or on stdin (`hp:src/cli.rs:192`). The plugin writes `brief.md`
itself after placement (`hp:src/threads.rs:153,229,274-284`). At resolution time the plugin
has: title, the `--plain` birth sentence (D17 item 6, `SPEC-ADE.md:668`), the task text,
`--role`, `--round`, the repo, and the project name and goal. The spec should list exactly those
fields. It should also make the excerpt rule normative (v0 drops code fences and cuts at 2,200
characters, `collect_labels.py:209-216`) and put it in `policy_hash`. The jaggedness page warns
about large irrelevant state, and round-table briefs are mostly setup and closing steps.

**S2. Redact names, not only start lines.** Risk 4 covers only `herdr agent start` lines. The
leak in B2 is prose. Redact model ids, vendor and CLI names, effort words and permission flags
before the call. If the task text names a model, the plugin does not follow it silently: it
prints `task names a model; pass --recipe to pin it` and resolves as usual.

**S3. The stronger-model rule is misstated as a start-time rule.** The memory says "use a
stronger model than OpenCode Muse Spark for complex packages" (`stronger-model-for-complex-lanes.md:3,22-26`),
and "hard numerics/engine lanes may still go to sol high … if Grok stalls, with Rolf told"
(`worker-model-preference.md:85-86`). That is an escalation after a stall, relative to a weaker
model. `run_experiment.py:48` and the spec's §2 `policy` make it a start rule against Grok. That
wording caused B's 16 Sol over-fires. In v2 it also caused A's three false Sol picks (`WP24`,
`WP27`, `camber-proof`), all on flyonenomics numerics that Rolf staffed on Grok. Sol and Astra
belong on the restart path (`ha thread restart <id> --recipe codex_sol_high`, offered through an
`ha ask` after a stall), not in any start-time allowed list.

**S4. Astra is off today.** "Codex astra is out of weekly usage on 2026-09-16; use it again only
when Rolf says so" (`worker-model-preference.md:11`). Clamp 4 has no data source because the
plugin cannot see Codex usage. So the day-one table must have `codex_astra_high` in no allowed
list, and it comes back only when Rolf adds it.

**S5. Spec pairs are resolved together, before either launch, not fixed afterwards.** D7 starts
both sides in one verb, `ha dialogue start <topic> --drafter <role> --critic <role|pro>`
(`SPEC-ADE.md:458-460`). Resolve the drafter first. Then remove every recipe with the drafter's
model from the critic's allowed set before the critic call. Clamp 3's fixed swap to
`claude_fable_high` fails when the drafter is Fable: SPEC-ADE itself was drafted on
`claude-fable-5-1` (`SPEC-ADE.md:3`). Nor can the data settle the critic pick: the only critic
label is Opus with a Grok author (`durable/turns/00-brief-02.md`), and Jev chose Fable (0.84 in
v0, 0.44 in v2). When the critic is `pro`, the resolver does not run, because Pro is started by
`pro-mcp start` and adopted passively (`SPEC-ADE.md:460-463`).

**S6. `pro` and `coordinator` never go through Jev.** Pro is adopted, never launched by
`thread start` (D7). The coordinator is launched by `ha open` before any task exists
(`SPEC-ADE.md:149-152`). `chatgpt_pro` should leave the Choice options. It is also not in the
spec's own six installed kinds (§2 line 41), so the §5 inventory intersection would drop it.
v0 offered it on all 97 cases, and it was never chosen.

**S7. Clamps 1 and 2 are table validation, not edits after the pick.** If code rewrites `args`
after Jev picks, `recipe_id` no longer describes the argv that ran, and a replay of
`policy_hash` gives a different launch. Check at config load and in `ha doctor`, and refuse to
start. A Claude recipe whose `args` contain `--effort xhigh` or `--effort max` is refused
(`recipe_effort_forbidden`). A `claude`, `cursor` or `agy` recipe without its permission flag is
refused (`recipe_permission_missing`), which extends D2's warning at `SPEC-ADE.md:287-288`. That
leaves one pick-time rule: the spec-pair exclusion in S5.

**S8. One pick per thread; restarts reuse it.** The identical request repeated gave the same
choice 93 times out of 97. Confidence moved by up to 0.20 (`stability_v2_repeat`). All four
flips were below 0.55 (`WP17` 0.44→0.53, `WP24` 0.35→0.34, `WP3-renders` 0.50→0.46,
`WP5b-drawings` 0.38→0.33), so with a 0.55 floor the launch was the same in both runs. The spec
should say that `ha thread restart` (`hp:src/threads.rs:385-423`), the ticker's relaunch
(`hp:src/ticker.rs:453-456`) and a re-run of a `thread start` that failed after resolution all
reuse `launch` and never call Jev again. Only an explicit `--recipe` changes it, and that change
is recorded as a new `attempt`.

**S9. Risk 6 and the `which` inventory rest on the wrong process.** The ticker is spawned with
the caller's environment minus five `HERDR_*` keys (`hp:src/ticker.rs:121-137`), so it does
inherit `TYPESAFE_API_KEY` when the caller had it. Under the spec's Design item 2 the ticker
never calls Jev anyway. The real gap is the inventory. `codex` and `opencode` resolve under a
per-shell fnm path, `~/.local/state/fnm_multishells/<pid>_<ts>/bin/`, so `which` in the plugin
process says nothing about the pane shell herdr types into. Use `herdr agent start --help`'s
kind list plus an `ha doctor` probe. Treat `agent_start` failing to detect the agent as the
runtime signal, not `which` at `thread start`.

**S10. The spec's own reason sentences fail D17.** "This is web research, so it uses the Gemini
lookup CLI." fails R3, because `CLI` is ALL-CAPS of two letters (`SPEC-ADE.md:561-563`).
`Gemini` fails R4 unless it is in the word list or the glossary (`:565-568`). The fallback
sentence "…because Jev did not answer" fails R4 on `Jev`. Templates are a finite set, so check
all of them at config load with `ha plain check`. A template that fails refuses the config. Any
model word a template keeps gets a glossary entry through `ha term add … --plain`.
Paste-ready templates are in §3.4.

**S11. Recommend Design C, gates over a fixed table, instead of an open Choice.** On redacted
briefs the `web_research` Noul split the set cleanly. The six research lanes scored 0.80 to
0.98, and every other brief scored 0.26 or less. With the fixed table plus "lane → Antigravity
when `web_research ≥ 0.75`", agreement is 82/97 against a baseline of 76, with 0 wrong switches
and no floor to tune (thresholds 0.5 to 0.75 give the same result). The venator probe reached
the same shape: "Use it only as a high-precision kill, never as a three-way decision"
(`240fb49` `jev-experiment.md:290`), and there hand rules beat the fitted combiner at zero false
positives (`:509`). Caveat: all six research labels come from one project and one lane name
(elicio `w5`), and some package paths contain "research". Other gates, such as `hard_review`
for reviewers or `spec_author` for drafters, stay telemetry until labels show they separate.
`hard_review` did not: Opus-labelled reviewers scored 0.32 to 0.60, Grok-labelled ones 0.28 to
0.61.

**S12. Most non-default labels encode a per-round instruction, which no brief contains.** The
start lines were written by coordinators applying the rule of the day or Rolf's word for that
round. Examples: "use codex astra high" for WP17 and WP19 (`worker-model-preference.md:88`), the
Opus reviewer fallback when Codex ran out (`:61`), Grok high from 2026-09-14 (`:74-86`). Label
dates confirm the drift. Every 2026-09-14 and 2026-09-15 label is Grok high, Kimi, Opus or
Astra; from 2026-09-16 on, 61 of 75 are Grok extra-high (git add dates of the brief files). In
the new world those picks are `--recipe` overrides or a project-level default. The evaluation
should tag them and score Jev only on labels a brief could explain.

**S13. The timeout needs a single attempt.** Across 388 calls, p95 was 0.43 s and the maximum
2.59 s (one call over 2 s). A 2 s deadline is right, but the spec also says "retry 429/529 with
backoff, as the SDKs do". Backoff starting at 1 s (`run_experiment.py:220-231`) breaks a 2 s
budget. Make it one attempt with a 2 s total deadline. A 429, 529,
timeout or transport error goes straight to `fallback = "timeout"|"error"`, with no retry.

**S14. `policy_hash` must cover the resolver.** It hashes: `resolver`, the recipe rows, each
role's `default` and `allowed`, the gate questions (instructions and criteria, verbatim), the
thresholds, the redaction and excerpt rules, and `jev_model`. Otherwise two launches with the
same hash can differ.

### 2.4 Nits

- **N1.** §4 line 126 says 84/97; the table says family 85/97. Both are right: 84 = 71 + 13 Grok
  date-drift misses; 85 also counts `claude_opus_xhigh → claude_opus_high`. Say which one is the
  headline.
- **N2.** Line 128, "Low confidence on all four non-default kind misses except two Opus
  reviewers", is hard to parse. The four Opus reviewer misses were at 0.45, 0.47, 0.66 and 0.77.
- **N3.** The drafting lane's own brief `herdr:tasks/jev/tbc-jev-picker.md` is in the label set.
  It states the rule verbatim, and Jev gave it 0.94. Exclude the picker's own briefs.
- **N4.** The set is two projects: elicio 47 and flyonenomics 39. herdr has 10 and venator 1.
  Give per-project numbers next to the pooled ones.
- **N5.** `opencode models` lists `opencode/muse-spark-1.2` and `opencode-go/muse-spark-1.3-contributor`.
  Name one id in the table. herdr also has a `muse` kind; say why the recipe uses `opencode`.
- **N6.** Codex recipes carry only `-c model=… -c model_reasoning_effort=high`. They inherit
  `approval_policy = "never"` and `sandbox_mode = "danger-full-access"` from `~/.codex/config.toml`.
  Say so, or put the flags in `args` so a changed global file cannot silently sandbox a lane.
- **N7.** `herdr agent start --help` now defaults `--timeout` to 30000; the plugin still passes
  20000 (`hp:src/herdr.rs:341`). D2's `ready_timeout_ms` covers it, so give per-kind values in
  the table.

### 2.5 Confirmed

- §1: endpoint, request and answer shapes (`api.md`); price $0.042/Mtok, output free, 64k/32k
  context, rate limits (`models.md`); alias `jev-latest` → `jev-1.13.0` and the advice to pin a
  version when thresholds are tuned (`models.md`, "Aliases"); `DEFAULT_TIMEOUT = 10.0`
  (`sdk/python/api/constants.md`); Choice confidence derived from probabilities, no confidence on
  Noul (`confidence.md`).
- §4: v0 numbers reproduce from `results/records.jsonl`: A 71/89/85 and B 66/79/78; the 8 A kind
  misses with their confidences; B's 16 Sol over-fires (12 Grok extra-high + 4 Grok high).
- §5 and touch points: plugin deps (`hp:Cargo.toml`: anyhow, clap, jiff, serde, serde_json, sha2,
  toml); `hp:src/project.rs:129` `thread_agent`; `:217` `thread_agent_args`;
  `hp:src/cli.rs:187-189` `--agent`; `hp:src/threads.rs:138`; `hp:src/herdr.rs:340-352`;
  `hp:src/ticker.rs:456` launches with live `safety.thread_agent_args` (the spec is right that it
  must stop); the detached spawn at `hp:src/ticker.rs:121-146`.
- SPEC-ADE refs: D2 `:274-293`, launch record `:281-283`, D17 `:539`, TBC-1 `:1349-1350`.
- CLIs: `cursor-grok-4.6-xhigh` in `cursor-agent --list-models`; `claude --effort` and
  `--dangerously-skip-permissions`; `agy models` lists `gemini-3.8-flash-high` and `agy` takes
  `--dangerously-skip-permissions`; `~/.codex/config.toml` has `gpt-5.6-sol` at high; herdr lists
  25 kinds including `chatgpt` and `dsh`.
- Venator: LOO AUC 0.952 (`jev-experiment.md:491`).

## 3. Proposed text, paste-ready

### 3.1 Summary (replaces lines 3 to 11)

```markdown
Jev is a small judgment service. It reads a lane's task and answers narrow yes-or-no questions with a probability.

Your table still picks the lane's agent and model. Jev can move a lane to another allowed row only when one of those questions is clearly yes.

On 97 old tasks with model names hidden, one question found all six research lanes and flagged no other lane. An open pick among all rows did worse than your default.

The pick is made once, before the lane exists, and saved. If Jev is slow or down, your default row is used and the board says so.

Stronger models come in when a lane stalls, with you asked first, not at the start.
```

### 3.2 §2 Input, output and config (replaces lines 35 to 74)

```markdown
**Input** (state built by the plugin at `thread start`, before any tab or worktree):

- `task`: the `thread start` task text after redaction (model ids, vendor and CLI names, effort
  words, permission flags → `[agent]`; lines containing `herdr agent start` dropped; fenced code
  dropped; cut at 2,200 characters on a word boundary).
- `title`, `sentence` (the `--plain` birth sentence), `role`, `round` (if any), `project`
  (name and goal from PROJECT.md), `repo` (basename only).
- `policy`: the role's policy lines from the config, verbatim.
- Nothing else. No installed-kinds list (the options already are the installed set) and no
  model lists.

**Resolver shape (Design C).** The role's `default` row is the answer unless a gate fires. A gate
is one Noul question tied to one recipe in the role's `allowed` list. A gate fires when its
Noul is at least its `threshold`. If two gates fire, the higher Noul wins; a tie keeps the
default. All gates for a role go in one request. A role with no gates does not call Jev.

**Output**: the D2 row of the chosen recipe, copied into the thread's `launch`, plus
`recipe_id`, `gate` (the gate that fired, or none), `gate_p`, `reason` (the recipe's template,
or the fallback template), `fallback`, `jev_model`, `jev_input_tokens`.

**Config** (`~/.config/herdr-ade/config.toml`, the safety file):

    [roles]
    resolver = "off"            # "off" | "jev"
    jev_model = "jev-1.13.0"
    jev_timeout_ms = 2000

    [recipes.cursor_grok_xhigh]
    kind = "cursor"
    args = ["--model", "cursor-grok-4.6-xhigh", "--force"]
    env = []
    ready_timeout_ms = 30000
    reason = "This is ordinary build work, so it uses your standing default."

    [recipes.agy_gemini_flash]
    kind = "agy"
    args = ["--model", "gemini-3.8-flash-high", "--dangerously-skip-permissions"]
    env = []
    ready_timeout_ms = 60000
    reason = "This task reads public web pages and cites them, so it uses the research helper."

    [recipes.claude_opus_high]
    kind = "claude"
    args = ["--model", "claude-opus-5", "--effort", "high", "--dangerously-skip-permissions"]
    env = []
    ready_timeout_ms = 90000
    reason = "This is design or a hard review, so it uses the careful reviewer."

    [recipes.claude_fable_high]
    kind = "claude"
    args = ["--model", "claude-fable-5-1", "--effort", "high", "--dangerously-skip-permissions"]
    env = []
    ready_timeout_ms = 90000
    reason = "This is the other side of a spec talk, so it uses a different model."

    [roles.lane]
    default = "cursor_grok_xhigh"
    allowed = ["cursor_grok_xhigh", "agy_gemini_flash"]
    [[roles.lane.gates]]
    recipe = "agy_gemini_flash"
    threshold = 0.75
    instructions = "Is the main job of `task` to read public web pages, vendor pages or datasheets and cite them, rather than to write or change the product?"
    criteria = { true = "Web research with citations.", false = "Implementation, review, or spec writing." }

    [roles.reviewer]
    default = "cursor_grok_xhigh"
    allowed = ["cursor_grok_xhigh", "claude_opus_high"]   # no gate yet: Opus only by --recipe

    [roles.critic]
    default = "claude_fable_high"
    allowed = ["claude_fable_high", "claude_opus_high", "cursor_grok_xhigh"]

    [roles.drafter]
    default = "claude_opus_high"
    allowed = ["claude_opus_high", "claude_fable_high", "cursor_grok_xhigh"]

A D2-era role written inline (`[roles.lane] kind = … args = …`) still loads: it is read as a
recipe named `<role>_inline` with `default` and `allowed` both that recipe. A config with both
forms for one role is refused (`role_form_mixed`).

**Per-project lists.** PROJECT.md front matter may give, per role, either `kind`/`args` (decision
3: a pin, Jev skipped) or `allowed = [...]`, a subset of the config's recipe ids that replaces
the global list for this project (arrays replace, never merge, as D2). It may not define new
recipes. An id that is not a global recipe is refused (`recipe_unknown`).

**Validation at load and in `ha doctor`, before any tab or worktree:**

- each recipe's `kind` is one of `herdr agent start --kind` values, else `recipe_kind_unknown`;
- a `claude` recipe with `--effort xhigh` or `--effort max` is refused (`recipe_effort_forbidden`);
- a `claude`, `cursor` or `agy` recipe without its permission flag is refused
  (`recipe_permission_missing`);
- `default` is in `allowed`, every gate's `recipe` is in `allowed`, every id exists;
- every `reason` template and the fallback template pass `ha plain check`
  (`recipe_reason_not_plain`);
- `codex_*` recipes are absent from start-time `allowed` lists unless Rolf added them by hand,
  and `ha doctor` prints a one-line note that Codex usage is not visible to the plugin.

`pro` and `coordinator` have no `allowed` list and never call the resolver: Pro is adopted (D7),
the coordinator is launched by `ha open` before any task exists.
```

### 3.3 §3 Order (replaces lines 80 to 85)

```markdown
**Order at `thread start` and `dialogue start`, before any tab or worktree exists:**

1. `--recipe <id>` → that recipe (must be in the role's allowed list for this project, else
   `recipe_not_allowed`); `--agent <kind>` without `--recipe` is `role_args_missing` (decision 13).
   `fallback = "override"`.
2. PROJECT.md `kind`/`args` pin for the role → that row; `fallback = "project"`.
3. `resolver = "off"` or the role has no gates → the role's default; `fallback = "resolver_off"`.
4. Else one request, one attempt, deadline `jev_timeout_ms` including connect. Answer received:
   a gate fires → its recipe, `fallback = ""`; no gate fires → default, `fallback = "no_gate"`.
   Timeout, transport error, 401, 422, 429, 529 → default, `fallback = "timeout"` or `"error"`,
   no retry. An answer missing a gate id → `fallback = "error"`.
5. Store the row on `launch` with `attempt = 1`. From here on, `thread restart`, the ticker's
   relaunch and a re-run after a failed placement reuse `launch` and never call Jev again. Only
   `ha thread restart <id> --recipe <id>` changes it, as a new `attempt`.

**Spec pairs.** `ha dialogue start --drafter <role> --critic <role>` resolves the drafter first,
then removes every recipe with the drafter's `--model` value from the critic's allowed set, then
resolves the critic. If that leaves the critic's set empty, the verb refuses with
`dialogue_same_model` before creating anything. `--critic pro` skips the resolver.

**Stall escalation.** A stronger model is never picked at the start. When a lane's thread is
stalled (the ticker's existing stall signal), the plugin may open an `ha ask`: "This lane has
stopped making progress. Restart it on the stronger coding model?" with the recipes Rolf listed
under `[roles.lane] escalate = [...]`. A yes runs `thread restart --recipe`.
```

### 3.4 Reason templates (replaces line 89 and Design item 6)

```markdown
Reason sentences come from the recipe's `reason` field or the fixed fallback line, never from
Jev. All are checked by `ha plain check` at config load. Day-one lines:

- "This is ordinary build work, so it uses your standing default."
- "This task reads public web pages and cites them, so it uses the research helper."
- "This is design or a hard review, so it uses the careful reviewer."
- "This is the other side of a spec talk, so it uses a different model."
- Fallback: "The standing default is used because the picker did not answer in time."
- Pinned: "You chose this helper for this lane."

The helper names ("research helper", "careful reviewer") get one `ha term add` each with a birth
sentence that names the product once, for example `ha term add "research helper" --plain "The
research helper is the Google model that reads the web for us."`.
```

### 3.5 §4 results (replaces lines 119 to 132)

```markdown
**Baselines.** Always Grok extra-high: 61/97 exact, 78/97 kind (Grok high and Kimi also run on
Cursor). In today's terms (Grok high counted as extra-high, Opus xhigh as Opus high): 76/97.

**Leak.** 46 of 97 briefs name their model in the title or prose after the start line is
stripped. The first pass (71 exact, 89 kind) read those names. With names redacted, the same
request gives 64 exact and 82 kind, and the only non-default lanes it still finds are research
lanes.

**Shipped shape, names redacted, today's 8 rows, today's labels (check a):**

| resolver | agrees | wrong switches away from the default |
| --- | ---: | ---: |
| always default | 76/97 | 0 |
| open Choice, no floor | 74/97 | 7 |
| open Choice, floor 0.55 | 80/97 | 0 |
| default + `web_research` gate ≥ 0.75 | 82/97 | 0 |

The gate recovers all six research lanes; research Noul 0.80–0.98, every other brief ≤ 0.26.
Same request twice: 93/97 identical picks, confidence moving up to 0.20; all flips below 0.55.
Cost at one question: about 1,880 input tokens, $0.00008 per start. Latency p95 0.43 s, max 2.59 s.

Ship Design C: the fixed table plus narrow gates. The open Choice stays an offline experiment
until a clean second month beats the default by leave-one-out.
```

### 3.6 Evaluation protocol for the second labelled month (new §4.1)

```markdown
**Labels.** From `ha` launch records, not brief text: for each `thread start` record the
redacted Jev input, `launch.recipe_id`, `fallback`, and whether the pick was Rolf's word for
that round (`--recipe` given, or PROJECT.md pin) or the table's. A thread Rolf restarts on a
different recipe within its first hour is labelled with the restart recipe. Target: at least 150
table-decided threads across at least three projects, with at least 15 per non-default recipe
that has a gate.

**Scoring.** Score only table-decided threads. Report exact recipe agreement, wrong switches away
from the default, and recovered non-default lanes, each per project and pooled, against the
always-default baseline for the same threads. Choose each gate's threshold by leave-one-out over
threads and confirm it by training on the older projects and testing on the newest one. A gate
ships only when it has zero wrong switches at its threshold on held-out projects and recovers
more than half of its recipe's lanes. Re-run with the identical request once to report
pick stability. Budget: under $0.05 per full re-run at one to three gates per request.
```

### 3.7 Design (normative), replaces lines 177 to 183

```markdown
1. Config grows `[roles] resolver`, `jev_model`, `jev_timeout_ms`; `[recipes.<id>]` carrying the
   full D2 row plus `reason`; per role `default`, `allowed`, optional `gates` and `escalate`.
   Inline D2 roles still load.
2. Resolution runs in `thread start` and `dialogue start` before any tab or worktree, is stored on
   `launch`, and is never repeated for that thread; the ticker only executes `launch`.
3. Jev answers one Noul per gate in one request; it never picks from an open list. A gate can only
   move a lane to a recipe already in that role's allowed list.
4. Failure, timeout, a missing answer, or no gate over its threshold → the default row and a
   plain fallback sentence. One attempt, no retry.
5. `--recipe` and PROJECT.md pins skip Jev; `--role` selects the table; PROJECT.md may narrow a
   role's `allowed` list.
6. Stronger models come only through stall escalation with an `ha ask`, never at start.
7. Board publishes `launch.reason`; every template passes `ha plain check` at config load.
8. `policy_hash` covers the resolver fields, recipes, lists, gate text, thresholds, redaction and
   excerpt rules, and `jev_model`.
9. No herdr-core change. No new socket method. No Python helper.
```

## 4. Readings on open questions 1 to 12

1. **Turn the resolver on.** Only with Design C and only for `lane` with the research gate,
   after the config lands with `resolver = "off"`. Turn it on per project after a week of shadow
   mode, where the plugin calls Jev, logs what the gate would have done, and launches the
   default. No project gets `jev` as its default until the second-month protocol (§3.6) passes.
2. **Confidence floor.** No single number. With Design C there is no Choice floor. Each gate has
   its own threshold, chosen by leave-one-out. 0.75 for the research gate is supported: 0.50 to
   0.75 gave the same result on 97. A 0.50 Choice floor is contradicted by the spec's own data
   (B4).
3. **Sol auto-upgrade.** Not at start. Rolf's own note makes it a stall escalation, "if Grok
   stalls, with Rolf told". Put Sol under `escalate`, not `allowed`. The data agrees: every
   start-time Sol pick, in both designs, was on a lane Rolf ran on Grok.
4. **Astra quota.** Rolf, by editing the list. The plugin cannot see Codex usage. `ha doctor`
   prints a one-line reminder whenever a `codex_*` recipe is in any list. Day one: Astra is in
   no list (S4).
5. **Grok high vs extra-high.** Extra-high only. Grok high exists only in labels from 2026-09-14
   and 2026-09-15.
6. **Kimi.** Historical. Two lanes, both in titles only, 0/2 once redacted. If Rolf wants it, it
   is a `--recipe` pin, not a Jev option.
7. **Spec-dialogue pairing.** Neither. Exclude the drafter's model from the critic's allowed set
   before the critic resolves, in `dialogue start` (§3.3). Passing the sibling into Jev's state
   is multi-hop indirection, which the jaggedness page lists as a weakness. A clamp after the
   pick breaks when the drafter is Fable.
8. **Timeout.** 2 s total, one attempt, no retry (S13). The observed maximum was 2.59 s in 388
   calls, and the rare miss costs only a default launch.
9. **Pin or follow latest.** Pin `jev-1.13.0`, as the models page advises when thresholds are
   tuned. Re-run the §3.6 scoring before moving the pin. The model id is in `policy_hash`.
10. **Reason templates.** Neither as written, since the spec's two samples fail R3 and R4. Put
    them in config as `reason` fields, check them at load, and add the helper names with
    `ha term add` (§3.4).
11. **Installed-model crawl.** A hand-maintained recipe table, validated against
    `herdr agent start --help` kinds at load. An `ha doctor --models` probe may list model ids
    for Rolf to copy, and never writes the table. `which` at `thread start` is the wrong process
    (S9).
12. **Cost cap.** Not needed for money: one gate costs about $0.00008 per start. Keep a
    structural cap instead of a daily dollar cap: at most one Jev call per thread (S8) and one
    per dialogue side.

## 5. New open questions

13. **What leaves the machine.** The redacted task text goes to TypeSafe. Zero retention is
    enterprise-only (§1). Which projects may send task text at all? Proposed: a per-project
    `jev = true` in PROJECT.md, default off.
14. **A task that names its model.** When a coordinator writes "Claude Opus, high effort" in the
    task, should the plugin treat that as a pin (`--recipe`), or only warn and resolve as usual
    (S2)?
15. **Round-level picks.** Rolf's per-round words ("opus high will do" for this round's
    reviewer) are most of the non-default labels. Should they become a project-level, dated
    change to `[roles.reviewer] default` that expires when the round closes, rather than a
    `--recipe` on every thread?
16. **Stall escalation as a feature.** Is an `ha ask` to restart a stalled lane on a stronger
    recipe wanted in the plugin round that ships the resolver, or later? It replaces the
    stronger-model rule that Jev cannot apply at start.
17. **Second gate candidates.** Which non-default picks does Rolf want the table to make without
    being asked? Candidates: `spec_author` for a drafter on Opus versus Fable, and "hard review"
    for a reviewer on Opus. On today's data neither separates, so each needs its own labels
    first.
18. **Shadow mode length.** How long does the resolver run in log-only mode on a project before
    it may launch? Proposed: until 30 threads or two weeks, whichever comes later.
19. **Does Rolf want the model named on the board at all?** The D17 path is a helper name with a
    glossary sentence. The alternative is no model word on the board, and the model only in
    `ha thread show`.
