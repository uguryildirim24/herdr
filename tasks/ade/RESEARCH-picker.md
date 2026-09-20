# Dispatch Classification Policy: Jev System One Routing Specification

This document defines the classification policy for the herdr model picker. The picker runs at task dispatch. It uses Jev, TypeSafe's System One model, to classify tasks and select the execution model.

---

## 1. One Choice, or Several Judgments Composed in Code?

### Recommendation
Use **composite scoring composed in code**.

Do not use a single `Choice` question over the model roster. Instead, send three atomic judgment questions (`difficulty`, `ambiguity`, `blast_radius`) in parallel in one TypeSafe request. Compute a numeric routing score in code. Map the score to a model tier using configurable thresholds.

### Why Composite Scoring Wins
1. **Decoupling from the model roster:**
   The classification prompt evaluates the properties of the software task. It contains no model names, provider labels, or pricing. When you add a new model, remove a deprecated model, or change providers, the prompt does not change.
2. **Zero-cost policy retuning:**
   You can change dimension weights, threshold cutoffs, and cost policies in a configuration file. You can test thousands of threshold combinations against historical data in milliseconds without sending a single API request.
3. **Auditability and ledger clarity:**
   Every dispatch decision writes its raw dimension scores and confidence to the failure ledger. You can inspect why a task escalated:
   `routed=gpt-6-astra-high: difficulty=2.88, ambiguity=1.40, blast_radius=1.80, composite=2.22 (threshold >= 1.80)`.
   A single `Choice` returns only an opaque selection and relative probabilities.
4. **Alignment with System One design:**
   TypeSafe documentation recommends decomposing broad judgments into narrow, typed questions. Broad questions hide multiple judgments behind one answer. Atomic questions expose those judgments for inspection and composition in code.

### What the Alternatives Cost

| Attribute | Single `Choice` Over Models | Composite Scoring in Code |
| :--- | :--- | :--- |
| **Model Roster Change** | High cost. Must rewrite prompt criteria and instructions for every model change. | Zero cost. Prompts evaluate task complexity, not models. |
| **Policy Retuning** | Impossible without paying for new model inference calls. | Instant. Change weights and thresholds in a local TOML file. |
| **Token Cost** | ~820 input tokens, ~30 output tokens. | ~820 input tokens, ~140 output tokens (output is free on TypeSafe). |
| **Latency** | ~100–300 ms. | ~100–300 ms (questions execute in parallel). |
| **Separation Quality** | Fragile. Tends to collapse to cheap default on short context. | Robust. Separates mechanical work from architectural risk. |

---

## 2. Which Dimensions Separate Hard Work from Easy Work?

Software tasks fail on cheap models (such as DeepSeek V4.1 Flash) for three distinct reasons:
1. Cognitive and state complexity (subtle ordering, concurrency, distributed state, or hidden invariants).
2. Architectural ambiguity (unspecified contracts where the worker must invent the design).
3. Blast radius and coupling (changes touching core lifecycle, contracts, or cross-process protocols).

### The Three Proposed Questions (Ready to Paste)

All three questions evaluate `state.brief` and `state.context` in one parallel request.

```json
{
  "difficulty": {
    "type": "score",
    "instructions": "Rate the technical and state complexity of the implementation work required by `state.brief`.",
    "criteria": [
      "Level 0 (Mechanical): A localized edit, straightforward bug fix, or boilerplate addition. The implementation pattern is obvious or fully described in the brief.",
      "Level 1 (Multi-file Coordination): Standard implementation across several files. The patterns and interfaces already exist in the codebase, but multiple call sites must agree.",
      "Level 2 (Design & New Subsystem): A new subsystem, new abstraction, or new data contract. The worker must understand unfamiliar code and make architectural decisions.",
      "Level 3 (Subtle State & Concurrency): High invariant risk. Involves process lifecycles, race conditions, async coordination, state recovery, or ordering rules that are easy to get subtly wrong."
    ]
  },
  "ambiguity": {
    "type": "score",
    "instructions": "How much architectural design and requirement interpretation does `state.brief` leave to the worker?",
    "criteria": [
      "Level 0 (Prescribed): The brief specifies the exact files, functions, types, and logic. Little to no design discretion is left to the worker.",
      "Level 1 (Bounded Discretion): The goal, interfaces, and constraints are clear, but internal implementation details are left to the worker.",
      "Level 2 (Open-ended Goal): The brief states an outcome or symptom, but does not specify the architectural design, contracts, or sequence of changes."
    ]
  },
  "blast_radius": {
    "type": "score",
    "instructions": "What is the system coupling and regression risk of the files and subsystems described in `state.brief`?",
    "criteria": [
      "Level 0 (Leaf / Isolated): Changes are limited to standalone CLI flags, documentation, isolated utilities, or single test fixtures.",
      "Level 1 (Feature Module): Changes touch a functional module or tool, but do not alter core schemas, coordinator bindings, or harness lifecycles.",
      "Level 2 (Core Backbone): Changes modify persistent state schemas, IPC protocols, process supervision, lock boundaries, or round-verification contracts."
    ]
  }
}
```

### Why These Dimensions Avoid the Surface-Feature Trap

Tonight, a question asking "does this need the open web" scored 0.96 on a brief that merely cited a documentation URL. Jev-1.13 is a literal reader. When instructions ask a general question without explicit boundaries, Jev matches on surface keywords (for example: `http://` -> `web = true`).

These proposed dimensions do not fire on surface features for three reasons:
1. **Action-oriented criteria instead of keyword criteria:**
   The criteria describe what the worker must do, not which words appear in the brief. A brief citing `src/contracts.rs` to fix a typo scores Level 0 on `blast_radius` because the change does not "modify persistent state schemas or system-wide contracts."
2. **Contrastive boundary definitions:**
   Each level defines its operational boundary. Level 0 on `difficulty` explicitly covers "localized edit, straightforward bug fix, or boilerplate addition." If a brief discusses a complex distributed bug but specifies a one-line shell fix, it matches Level 0.
3. **Separation of ambiguity from complexity:**
   A brief can be complex but completely prescribed (Level 3 difficulty, Level 0 ambiguity). Another brief can be simple but vague (Level 0 difficulty, Level 2 ambiguity). Evaluating them separately prevents high vocabulary complexity from masquerading as design difficulty.

---

## 3. What State Should the Question See?

### State Structure
Pass a structured JSON object to Jev. Do not pass a raw string.

```json
{
  "state": {
    "brief": "<scrubbed task brief text, capped at 4000 characters>",
    "title": "<one-line task title>",
    "context": {
      "named_paths": ["src/lane.rs", "src/ops.rs"],
      "existing_paths_count": 2,
      "new_paths_count": 0,
      "touches_core": true,
      "is_review": false,
      "diffstat": null,
      "attempt": 1,
      "prior_failure": null
    }
  }
}
```

### Facts Worth Including (Cheap Code Computation)
Code computes these facts deterministically before dispatch:
1. **Extracted file paths (`named_paths`):**
   A fast regex scans the brief for paths matching `src/*`, `plain/*`, `tasks/*`, etc.
2. **Path existence count (`existing_paths_count`, `new_paths_count`):**
   Fast filesystem check (`Path::is_file`). If `new_paths_count > 0`, it indicates new subsystem work.
3. **Core subsystem flag (`touches_core`):**
   Boolean check against core files (`src/contracts.rs`, `src/ops.rs`, `src/lane.rs`, `src/ticker.rs`, `src/project.rs`).
4. **Review diffstat (for review rounds only):**
   When `ha round review` calls the picker, code supplies `{"files_changed": N, "insertions": N, "deletions": N}`.
5. **Prior failure context (for escalations only):**
   If `attempt > 1`, code includes the last failure outcome (for example: `test_failed`, `reviewer_rejected`, `build_error`).

### Facts to Exclude
- **Full file contents:** Do not inline source files. Inlining files wastes tokens, causes context rot, and distracts Jev.
- **Git logs and commit histories:** Irrelevant background noise.
- **Model rosters or provider descriptions:** Leaks model bias into task evaluation.

### Why Code-Computed Facts Beat Asking the Model
Jev-1.13 cannot count reliably and struggles with arithmetic. Asking Jev "how many files does this touch?" produces hallucinated numbers. Computing path counts and core flags in code takes under 1 ms and costs 0 tokens. Jev uses these facts as semantic context to judge architectural scope.

---

## 4. Where Does Confidence Enter the Policy?

### The Asymmetric Cost of Routing Errors
The penalty for misclassification is strongly asymmetric:
- **Under-provisioning (hard task sent to cheap model):**
  The lane runs for 15–30 minutes, produces invalid code, fails verification, stalls review, and requires a restart.
  **Estimated loss:** $10.00 equivalent (human attention, review cycles, delayed round).
- **Over-provisioning (easy task sent to strong model):**
  The task completes on attempt 1. The lane succeeds.
  **Estimated loss:** $0.25–$0.50 (the token price difference).

**Core Principle:** When uncertain, upgrade to the stronger model. Never block dispatch. Never prompt the user interactively. Record the low-confidence pick in the failure ledger.

### Concrete Policy Rule

1. **Calculate the Composite Score ($S$):**
   $$S = (0.50 \times \text{difficulty}) + (0.30 \times \text{ambiguity}) + (0.20 \times \text{blast\_radius})$$
   Scale: $S \in [0.0, 2.7]$.

2. **Evaluate Routing Tiers:**
   - **Strong Tier (`gpt-6-astra-high`):**
     If $S \ge 1.60$, route to Strong Tier.
   - **Cheap Tier (`deepseek-v4.1-flash`):**
     If $S < 1.00$ AND $\text{confidence}_{\text{difficulty}} \ge 0.65$, route to Cheap Tier.
   - **Uncertain / Borderline:**
     If $1.00 \le S < 1.60$, OR $\text{confidence}_{\text{difficulty}} < 0.65$:
     **Route to Strong Tier (conservative upgrade).**
     Write a record to `.state/routing-ledger.jsonl`:
     `{"event": "low_confidence_upgrade", "score": S, "confidence": conf, "task": title}`.

3. **Specialized Review Routing:**
   When `is_review == true`:
   - If $S \ge 1.40$, route to careful verification model (`gpt-5.6-sol-high`).
   - If $S < 1.40$, route to `deepseek-v4.1-flash`.

### How to Choose the Threshold on Real Data

1. **Collect Validation Data ($N \ge 50$ historical tasks):**
   Extract historical task briefs from `adeherdr` threads (`t-0001` to `t-0088`). Label each task with the tier that succeeded on attempt 1.
2. **Define the Empirical Loss Function:**
   $$\text{TotalLoss}(T_{\text{cheap}}, C_{\text{floor}}) = \sum_{i=1}^{N} \text{Cost}(y_i, \hat{y}_i)$$
   Where:
   - $\text{Cost}(\text{hard}, \text{cheap}) = 10.0$ (critical failure)
   - $\text{Cost}(\text{easy}, \text{strong}) = 0.35$ (small token waste)
   - $\text{Cost}(\text{correct}) = 0.0$
3. **Grid Sweep:**
   Sweep $T_{\text{cheap}} \in [0.8, 1.4]$ in steps of 0.1, and $C_{\text{floor}} \in [0.50, 0.85]$ in steps of 0.05. Select parameters that minimize $\text{TotalLoss}$ while keeping the False Cheap rate below 4%.

---

## 5. How Do We Know It Is Working? Evaluation Design

### What a Labelled Case Looks Like
Store evaluation fixtures in `tests/fixtures/routing-eval.jsonl`. Each record contains:

```json
{
  "id": "t-0071",
  "title": "The cloud box builds the terminal program too",
  "brief": "Full brief text from t-0071.task.md...",
  "named_paths": ["scripts/build-box.sh", "Cargo.toml"],
  "touches_core": true,
  "ground_truth_tier": "strong",
  "ground_truth_reason": "Cross-compilation environment debugging on remote Linux container; required multi-step toolchain diagnosis.",
  "historical_attempts": [
    {"attempt": 1, "model": "deepseek-v4.1-flash", "outcome": "failed_build"},
    {"attempt": 2, "model": "gpt-6-astra-high", "outcome": "merged"}
  ]
}
```

### Sample Size Requirements
- **15–20 cases:** Statistical noise. Standard error is too wide ($\pm 15\%$).
- **40–50 cases:** Minimum viable evaluation. Can detect a 10% regression in classification quality.
- **80–100 cases:** High confidence benchmark. Standard error drops below $\pm 4\%$.
`adeherdr` already has 88 completed threads (`t-0001` through `t-0088`). We can construct a gold-standard dataset of 80 real tasks immediately.

### What to Measure (Metrics)
Do not optimize for raw accuracy alone. Measure:
1. **False Cheap Rate (Under-provisioning):**
   $$\frac{\text{Hard tasks sent to Cheap}}{\text{Total Hard tasks}} \quad (\text{Target: } < 4\%)$$
   This is the primary failure mode.
2. **False Strong Rate (Over-provisioning):**
   $$\frac{\text{Easy tasks sent to Strong}}{\text{Total Easy tasks}} \quad (\text{Target: } < 25\%)$$
   Acceptable buffer to ensure safety.
3. **Total Cost vs Baselines:**
   Compare the picker's total operational cost against:
   - Baseline 1: "Always Cheap" (calculating token costs + $10 penalty per failed lane).
   - Baseline 2: "Always Strong" (calculating maximum token cost with zero under-provisioning).
   - Baseline 3: "Legacy Roles Table".
4. **Calibration (Expected Calibration Error):**
   Verify that Jev's reported confidence matches actual empirical accuracy.

### Proposed Evaluation Command
Implement an evaluation harness verb:

```bash
ha picker eval --cases tests/fixtures/routing-eval.jsonl --config ~/.config/herdr-ade/picker.toml
```

Output report:
```text
Cases evaluated:        84
Accuracy:               88.1% (74/84)
False Cheap (critical): 1 / 38 (2.6%)  [PASS: target < 4%]
False Strong (waste):   9 / 46 (19.5%) [PASS: target < 25%]
Mean Latency:           240 ms
Estimated Net Savings:  $142.50 vs Always-Strong
```

---

## 6. What Should Be Data and What Should Be Code?

Rolf wants to tune the policy without recompiling the `herdr-ade` binary.

### Editable Configuration (`~/.config/herdr-ade/picker.toml`)

This file contains all tuneable policy parameters, weights, prompts, and recipe bindings:

```toml
# ~/.config/herdr-ade/picker.toml

[engine]
jev_model = "jev-1.13.0"
timeout_ms = 3000
max_retries = 1

[tiers]
cheap = "deepseek_v4_flash"
strong = "codex_astra_high"
verifier = "codex_sol_high"

[weights]
difficulty = 0.50
ambiguity = 0.30
blast_radius = 0.20

[thresholds]
cheap_ceiling = 1.00
strong_floor = 1.60
confidence_floor = 0.65
review_strong_floor = 1.40

[questions.difficulty]
type = "score"
instructions = "Rate the technical and state complexity of the implementation work required by `state.brief`."
criteria = [
  "Level 0 (Mechanical): Localized edit, straightforward bug fix, or boilerplate addition.",
  "Level 1 (Multi-file Coordination): Standard implementation across several existing files.",
  "Level 2 (Design & New Subsystem): New subsystem, new abstraction, or new data contract.",
  "Level 3 (Subtle State & Concurrency): High invariant risk. Process lifecycles, race conditions, async coordination, or recovery."
]

[questions.ambiguity]
type = "score"
instructions = "How much architectural design and requirement interpretation does `state.brief` leave to the worker?"
criteria = [
  "Level 0 (Prescribed): Exact files, functions, types, and logic are specified.",
  "Level 1 (Bounded Discretion): Clear goals and constraints; internal details left open.",
  "Level 2 (Open-ended Goal): Outcome or symptom stated without architecture or sequence."
]

[questions.blast_radius]
type = "score"
instructions = "What is the system coupling and regression risk of the files and subsystems described in `state.brief`?"
criteria = [
  "Level 0 (Leaf / Isolated): Standalone flags, docs, leaf utils, or test fixtures.",
  "Level 1 (Feature Module): Functional module or tool without modifying core contracts.",
  "Level 2 (Core Backbone): Modifies persistent state schemas, IPC protocols, process supervision, or lifecycle contracts."
]
```

### What Belongs in Compiled Code (`src/launch.rs` and `src/picker.rs`)

1. **Deterministic Exclusions (Strict Rules):**
   - Web research -> always `agy_gemini_flash`.
   - Claude coordinator / Claude binary -> always `claude`.
   - Spec and plan drafting -> always `fable_xhigh` or Pro relay.
   - User command-line pin (`--recipe <id>`) -> always the pinned recipe.
2. **Pre-computation and State Scrubbing:**
   - Path regex extraction, filesystem checks, git diffstat extraction.
   - Scrubbing agent names, API tokens, and shell wrappers from the brief.
3. **HTTP Client & Resilience:**
   - `curl` / HTTP POST to `https://api.typesafe.ai/v1/systemone`.
   - Timeout handling, 1 retry on 5xx/network error, fallback on complete failure.
4. **Mathematical Composition & Ledger Logging:**
   - Computing the weighted score $S$.
   - Evaluating confidence against `confidence_floor`.
   - Appending dispatch results to `.state/routing-ledger.jsonl`.
5. **Deterministic Escalation Ladder:**
   - If `attempt > 1`, code bumps the model tier automatically without re-invoking Jev.

---

## 7. Assumptions and Guesses

To maintain complete intellectual honesty, here are the explicit guesses in this design:
1. **Cost of failure ($10.00 equivalent):**
   This is an engineering judgment based on Rolf's time, review delays, and lane rebuilds. It is not an accounting measurement.
2. **Threshold values ($1.00, 1.60, 0.65):**
   These are initial priors derived from the score distribution in `scratch/jev-on-tasks.json`. They must be calibrated against the 80 historical cases before locking in.
3. **Difficulty weight (0.50):**
   We assume algorithmic and state complexity causes more agent failures than blast radius. If empirical data shows multi-file regressions dominate, `blast_radius` weight should be increased.
4. **Output token cost:**
   Assumed to remain $0.00 based on current TypeSafe pricing documentation.
