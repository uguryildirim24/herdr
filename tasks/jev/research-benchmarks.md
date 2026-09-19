# Model Pool Coding Benchmarks and Reasoning Efforts

Research date: 2026-09-18.
Repository: herdr.
Branch: lane/jev-bench.
Source pool: tasks/jev/pi-pool.md (16 models across ChatGPT, OpenCode Go, Kimi, and Claude).

## Summary table

| Model | SWE-bench Verified | Terminal-Bench 2 | LiveCodeBench |
|---|---|---|---|
| gpt-5.6-luna | not found | 84.3% @ effort not stated | 70.4% @ effort not stated |
| gpt-5.6-sol | not found | 84.6% @ effort not stated | not found |
| gpt-5.6-terra | not found | not found | not found |
| gpt-6-astra | not found | not found | not found |
| deepseek-v4-flash | 73.7% @ effort not stated | 82.7% @ effort not stated | 87.3% @ effort not stated |
| deepseek-v4.1-flash | not found | 90.6% @ effort not stated | not found |
| glm-5.3-flash | not found | 84.3% @ effort not stated | not found |
| muse-spark-1.3-contributor | not found | not found | not found |
| qwen3.8-flash | not found | not found | not found |
| qwen3.8-max | not found | 86.6% @ effort not stated | not found |
| k3 | not found | 88.3% @ effort not stated | not found |
| k3-256k | not found | not found | not found |
| claude-fable-5-1 | 95.0% @ effort not stated | not found | 90.0% @ effort not stated |
| claude-opus-5 | 96.0% @ effort not stated | 84.6% @ effort not stated | 89.0% @ effort not stated |
| claude-sonnet-5 | 85.2% @ effort not stated | 80.4% @ effort not stated | not found |
| claude-haiku-4-5-20251001 | 73.3% @ effort not stated | not found | 51.0% @ effort not stated |

## 1. gpt-5.6-luna

Provider: openai-codex (ChatGPT login, browser).
Pi accepted efforts: off minimal low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 4.0 | 17.27% | max | max | 2026-06-26 | https://www.tbench.ai |
| Terminal-Bench 2.1 | 84.3% | effort not stated | effort not stated | 2026-07-09 | https://rdworldonline.com |
| SWE-bench Pro | 62.7% | effort not stated | effort not stated | 2026-07-09 | https://benchlm.ai |
| LiveCodeBench | 70.4% | single-pass | effort not stated | 2026-07-09 | https://benchlm.ai |
| AA Coding Agent Index | 74.6 | effort not stated | effort not stated | 2026-07-09 | https://vellum.ai |
| SWE-bench Verified | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 2. gpt-5.6-sol

Provider: openai-codex (ChatGPT login, browser).
Pi accepted efforts: off minimal low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 4.0 | 37.27% | max | max | 2026-06-26 | https://www.tbench.ai |
| Terminal-Bench 2.1 | 84.6% | effort not stated | effort not stated | 2026-06-26 | https://www.tbench.ai/news/terminal-bench-2-1 |
| SWE-bench Pro | 64.6% | max reasoning | max | 2026-07-09 | https://developers.openai.com/api/docs/models/gpt-5.6-sol |
| AA Coding Agent Index | 80 | max reasoning | max | 2026-07-09 | https://artificialanalysis.ai |
| GeneBench-Pro | 28.7% | max | max | 2026-07-09 | https://developers.openai.com/api/docs/models/gpt-5.6-sol |
| SWE-bench Verified | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 3. gpt-5.6-terra

Provider: openai-codex (ChatGPT login, browser).
Pi accepted efforts: off minimal low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 4.0 | 21.52% | max | max | 2026-06-26 | https://www.tbench.ai |
| SWE-bench Verified | not found | - | - | - | - |
| SWE-bench Pro | not found | - | - | - | - |
| Terminal-Bench 2 | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 4. gpt-6-astra

Provider: openai-codex (ChatGPT login, browser).
Pi accepted efforts: minimal low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 4.0 | 58.18% | max | max | 2026-09-03 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 57.88% | xhigh | xhigh | 2026-09-03 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 57.88% | high | high | 2026-09-03 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 54.24% | medium | medium | 2026-09-03 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 50.61% | low | low | 2026-09-03 | https://www.tbench.ai |
| ExploitBench | 100% | max reasoning | max | 2026-09-03 | https://developers.openai.com/api/docs/models/gpt-6-astra |
| ARC-AGI-3 | 99.9% | max reasoning | max | 2026-09-03 | https://developers.openai.com/api/docs/models/gpt-6-astra |
| FrontierMath Tier 4 | 97.6% | max reasoning | max | 2026-09-03 | https://developers.openai.com/api/docs/models/gpt-6-astra |
| SWE-bench Verified | not found | - | - | - | - |
| SWE-bench Pro | not found | - | - | - | - |
| Terminal-Bench 2 | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 5. deepseek-v4-flash

Provider: opencode-go (API key).
Pi accepted efforts: off low high max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Verified | 73.7% | effort not stated | effort not stated | 2026-07-31 | https://github.com/deepseek-ai/DeepSeek-V4 |
| SWE-bench Verified | 79.0% | Flash-Max | max | 2026-08-15 | https://llm-stats.com |
| Terminal-Bench 2.1 | 82.7% | effort not stated | effort not stated | 2026-07-31 | https://benchlm.ai |
| LiveCodeBench | 87.3% | effort not stated | effort not stated | 2026-08-10 | https://vals.ai |
| SWE-bench Pro | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 6. deepseek-v4.1-flash

Provider: opencode-go (API key).
Pi accepted efforts: low high max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 2.1 | 90.6% | effort not stated | effort not stated | 2026-09-10 | https://benchlm.ai |
| Terminal-Bench 4.0 | 31.2% | effort not stated | effort not stated | 2026-09-10 | https://benchlm.ai |
| DeepSWE v1.1 | 74.2% | effort not stated | effort not stated | 2026-09-10 | https://github.com/deepseek-ai/DeepSeek-V4 |
| CyberGym | 88.1% | effort not stated | effort not stated | 2026-09-10 | https://github.com/deepseek-ai/DeepSeek-V4 |
| SWE-bench Verified | not found | - | - | - | - |
| SWE-bench Pro | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 7. glm-5.3-flash

Provider: opencode-go (API key).
Pi accepted efforts: low high max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 2.1 | 84.3% | effort not stated | effort not stated | 2026-08-26 | https://docs.z.ai/guides/llm/glm-5.3 |
| DeepSWE v1.1 | 63.4% | effort not stated | effort not stated | 2026-08-26 | https://docs.z.ai/guides/llm/glm-5.3 |
| Terminal-Bench 4.0 | 41.82% | max | max | 2026-08-14 | https://www.tbench.ai |
| SWE-bench Verified | not found | - | - | - | - |
| SWE-bench Pro | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

Note: Terminal-Bench 4.0 score was measured on the family model GLM-5.3 (family, not exact).

## 8. muse-spark-1.3-contributor

Provider: opencode-go (API key).
Pi accepted efforts: minimal low medium high xhigh.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| DeepSWE | 75.4% | effort not stated | effort not stated | 2026-09-02 | https://openrouter.ai/models/meta-llama/muse-spark-1.3 |
| SWE-bench Verified | not found | - | - | - | - |
| SWE-bench Pro | not found | - | - | - | - |
| Terminal-Bench 2 | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

Note: DeepSWE score is reported for the base Muse Spark 1.3 model (family, not exact).

## 9. qwen3.8-flash

Provider: opencode-go (API key).
Pi accepted efforts: off minimal low medium high.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Pro | 62.5% | effort not stated | effort not stated | 2026-08-26 | https://qwenlm.github.io/blog/qwen3.8/ |
| SWE-bench Multilingual | 81.0% | effort not stated | effort not stated | 2026-08-26 | https://qwenlm.github.io/blog/qwen3.8/ |
| DeepSWE 1.1 | 58.7% | effort not stated | effort not stated | 2026-08-26 | https://qwenlm.github.io/blog/qwen3.8/ |
| SWE-bench Verified | not found | - | - | - | - |
| Terminal-Bench 2 | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

Note: Scores were published for preview architecture Qwen 3.8-Flash-Next (family, not exact).

## 10. qwen3.8-max

Provider: opencode-go (API key).
Pi accepted efforts: low medium xhigh.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Pro | 67.7% | effort not stated | effort not stated | 2026-08-26 | https://qwenlm.github.io/blog/qwen3.8/ |
| Terminal-Bench 2.1 | 86.6% | effort not stated | effort not stated | 2026-08-26 | https://qwenlm.github.io/blog/qwen3.8/ |
| PaperBench | 93.0% | effort not stated | effort not stated | 2026-08-26 | https://qwenlm.github.io/blog/qwen3.8/ |
| SWE-bench Verified | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 11. k3

Provider: kimi-coding (Kimi login, device code).
Pi accepted efforts: low high max (adaptive).

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 2.1 | 88.3% | effort not stated | effort not stated | 2026-07-15 | https://moonshot.ai/blog/k3 |
| DeepSWE | 67.5% | effort not stated | effort not stated | 2026-07-15 | https://moonshot.ai/blog/k3 |
| SciCode | 58.7% | effort not stated | effort not stated | 2026-07-15 | https://moonshot.ai/blog/k3 |
| BrowseComp | 91.2% | effort not stated | effort not stated | 2026-07-15 | https://moonshot.ai/blog/k3 |
| SWE-bench Verified | not found | - | - | - | - |
| SWE-bench Pro | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 12. k3-256k

Provider: kimi-coding (Kimi login, device code).
Pi accepted efforts: low high max (adaptive).

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| Terminal-Bench 2.1 | 88.3% | effort not stated | effort not stated | 2026-07-15 | https://moonshot.ai/blog/k3 |
| DeepSWE | 67.5% | effort not stated | effort not stated | 2026-07-15 | https://moonshot.ai/blog/k3 |
| SWE-bench Verified | not found | - | - | - | - |
| Terminal-Bench 2 | not found | - | - | - | - |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

Note: No benchmark scores are published under the exact ID k3-256k. Reported numbers are from Kimi K3 (family, not exact).

## 13. claude-fable-5-1

Provider: Claude (native kind, Claude Code CLI).
Pi accepted efforts: low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Pro | 81.2% | effort not stated | effort not stated | 2026-09-01 | https://www.anthropic.com/news/claude-fable-5-mythos-5 |
| SWE-bench Verified | 95.0% | effort not stated | effort not stated | 2026-09-01 | https://benchlm.ai |
| Terminal-Bench 4.0 | 57.88% | max | max | 2026-09-01 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 57.88% | xhigh | xhigh | 2026-09-01 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 54.55% | high | high | 2026-09-01 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 53.94% | medium | medium | 2026-09-01 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 43.33% | low | low | 2026-09-01 | https://www.tbench.ai |
| LiveCodeBench | 90.0% | effort not stated | effort not stated | 2026-09-01 | https://benchlm.ai |
| Terminal-Bench 2 | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 14. claude-opus-5

Provider: Claude (native kind, Claude Code CLI).
Pi accepted efforts: low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Verified | 96.0% | adaptive thinking | effort not stated | 2026-07-24 | https://www.anthropic.com/news/claude-opus-5 |
| SWE-bench Pro | 79.2% | effort not stated | effort not stated | 2026-07-24 | https://www.anthropic.com/news/claude-opus-5 |
| Terminal-Bench 2.1 | 84.6% | effort not stated | effort not stated | 2026-07-24 | https://vals.ai |
| Terminal-Bench 4.0 | 53.94% | xhigh | xhigh | 2026-07-24 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 51.82% | max | max | 2026-07-24 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 50.30% | high | high | 2026-07-24 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 44.85% | medium | medium | 2026-07-24 | https://www.tbench.ai |
| Terminal-Bench 4.0 | 34.85% | low | low | 2026-07-24 | https://www.tbench.ai |
| LiveCodeBench | 89.0% | effort not stated | effort not stated | 2026-07-24 | https://vals.ai |
| Aider polyglot | not found | - | - | - | - |

## 15. claude-sonnet-5

Provider: Claude (native kind, Claude Code CLI).
Pi accepted efforts: low medium high xhigh max.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Verified | 85.2% | effort not stated | effort not stated | 2026-06-30 | https://www.anthropic.com/news/claude-sonnet-5 |
| SWE-bench Pro | 63.2% | effort not stated | effort not stated | 2026-06-30 | https://www.anthropic.com/news/claude-sonnet-5 |
| Terminal-Bench 2.1 | 80.4% | effort not stated | effort not stated | 2026-06-30 | https://www.anthropic.com/news/claude-sonnet-5 |
| Terminal-Bench 4.0 | 12.42% | max | max | 2026-06-30 | https://www.tbench.ai |
| LiveCodeBench | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## 16. claude-haiku-4-5-20251001

Provider: Claude (native kind, Claude Code CLI).
Pi accepted efforts: flag accepted; honoured: not verified.

| Benchmark | Score | Effort as stated | Pool effort | Date | Source |
|---|---|---|---|---|---|
| SWE-bench Verified | 73.3% | extended thinking | effort not stated | 2025-10-15 | https://docs.anthropic.com/en/docs/about-claude/models/all-models |
| LiveCodeBench | 51.0% | thinking | effort not stated | 2025-10-15 | https://benchlm.ai |
| SWE-bench Pro | not found | - | - | - | - |
| Terminal-Bench 2 | not found | - | - | - | - |
| Aider polyglot | not found | - | - | - | - |

## Name matching

1. `gpt-5.6-luna`: Family, not exact. Pi accesses the model through ChatGPT login. Vendor documentation and benchmarks test the general API model `GPT-5.6 Luna`.
2. `gpt-5.6-sol`: Family, not exact. Pi accesses the model through ChatGPT login. Vendor documentation and benchmark cards evaluate API model `GPT-5.6 Sol`.
3. `gpt-5.6-terra`: Family, not exact. Pi accesses the model through ChatGPT login. Benchmark entries refer to `GPT-5.6 Terra`.
4. `gpt-6-astra`: Family, not exact. Pi accesses the model through ChatGPT login. Benchmark leaderboards evaluate API model `GPT-6 Astra`.
5. `deepseek-v4-flash`: Exact match for technical reports referencing `DeepSeek-V4-Flash`. Leaderboard runs on `DeepSeek-V4-Flash-0731` are marked family, not exact.
6. `deepseek-v4.1-flash`: Exact match for `DeepSeek-V4.1-Flash`.
7. `glm-5.3-flash`: Exact match for vendor documentation on `GLM-5.3-Flash`. Leaderboard rows on `GLM-5.3` are marked family, not exact.
8. `muse-spark-1.3-contributor`: Family, not exact. The pool specifies the contributor pricing tier. All benchmark scores belong to the general model `Muse Spark 1.3`.
9. `qwen3.8-flash`: Family, not exact. Published scores belong to architecture preview `Qwen 3.8-Flash-Next`.
10. `qwen3.8-max`: Exact match for `Qwen 3.8-Max`.
11. `k3`: Exact match for `Kimi K3`.
12. `k3-256k`: Family, not exact. The model is a 256k context-window variant. All benchmark scores come from `Kimi K3`.
13. `claude-fable-5-1`: Exact match for `Claude Fable 5.1`.
14. `claude-opus-5`: Exact match for `Claude Opus 5`.
15. `claude-sonnet-5`: Exact match for `Claude Sonnet 5`.
16. `claude-haiku-4-5-20251001`: Exact match for `Claude Haiku 4.5` (snapshot 20251001).

## Effort sensitivity

1. `gpt-6-astra` on Terminal-Bench 4.0:
   - low: 50.61%
   - medium: 54.24%
   - high: 57.88%
   - xhigh: 57.88%
   - max: 58.18%
2. `claude-fable-5-1` on Terminal-Bench 4.0:
   - low: 43.33%
   - medium: 53.94%
   - high: 54.55%
   - xhigh: 57.88%
   - max: 57.88%
3. `claude-opus-5` on Terminal-Bench 4.0:
   - low: 34.85%
   - medium: 44.85%
   - high: 50.30%
   - max: 51.82%
   - xhigh: 53.94%
4. `deepseek-v4-flash` on SWE-bench Verified:
   - effort not stated: 73.7%
   - Flash-Max (max): 79.0%
5. Other models: Only single-point scores were published without effort curves.

## Gaps

1. Models with no published coding score under their exact ID:
   - `k3-256k`: Moonshot AI publishes benchmark scores for `Kimi K3`. No separate numbers exist for the 256k context-limited catalog entry.
   - `muse-spark-1.3-contributor`: Benchmark scores exist for `Muse Spark 1.3`. No separate numbers exist for the contributor discount row.
   - `qwen3.8-flash`: Benchmark scores are published for `Qwen 3.8-Flash-Next`. No separate numbers exist for the base flash row.
   - `gpt-5.6-terra`: Has no scores on SWE-bench Verified, SWE-bench Pro, Terminal-Bench 2, LiveCodeBench, or Aider. Its only published score is Terminal-Bench 4.0 (21.52% at effort max).
2. Models whose only scores are at an effort pi does not accept:
   - `muse-spark-1.3-contributor`: Pi accepts `minimal low medium high xhigh`, but excludes `max`. Any evaluation using max effort cannot run in pi.
   - `qwen3.8-flash`: Pi accepts `off minimal low medium high`, but excludes `xhigh` and `max`.
   - `qwen3.8-max`: Pi accepts `low medium xhigh`, but excludes `off`, `minimal`, `high`, and `max`.
   - `gpt-6-astra`: Pi accepts `minimal low medium high xhigh max`, but excludes `off`.
3. Ambiguity where effort was not stated:
   - `deepseek-v4.1-flash`, `glm-5.3-flash`, `qwen3.8-max`, and `k3` have vendor-reported scores on Terminal-Bench 2.1 and DeepSWE without published reasoning effort levels.
4. Harness sensitivity for Claude Haiku 4.5:
   - `claude-haiku-4-5-20251001` accepts the `--effort` CLI flag, but the CLI does not verify whether Haiku honors the flag.

## Sources

- https://aireleasetracker.com
- https://artificialanalysis.ai
- https://benchlm.ai
- https://developers.openai.com/api/docs/models/gpt-5.6-luna
- https://developers.openai.com/api/docs/models/gpt-5.6-sol
- https://developers.openai.com/api/docs/models/gpt-5.6-terra
- https://developers.openai.com/api/docs/models/gpt-6-astra
- https://docs.anthropic.com/en/docs/about-claude/models/all-models
- https://docs.z.ai/guides/llm/glm-5.3
- https://github.com/deepseek-ai/DeepSeek-V4
- https://llm-stats.com
- https://moonshot.ai/blog/k3
- https://openrouter.ai/models/meta-llama/muse-spark-1.3
- https://qwenlm.github.io/blog/qwen3.8/
- https://rdworldonline.com
- https://vals.ai
- https://vellum.ai
- https://www.anthropic.com/news/claude-fable-5-mythos-5
- https://www.anthropic.com/news/claude-opus-5
- https://www.anthropic.com/news/claude-sonnet-5
- https://www.tbench.ai
- https://www.tbench.ai/news/terminal-bench-2-1
