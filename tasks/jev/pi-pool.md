# The pool pi exposes per provider (Jev picker redo, step 1)

Read 2026-09-19 from the pinned pi 0.85.1 under `~/.herdr-ade/pi`, read-only. Source: the
pi.dev overlay `~/.herdr-ade/pi/agent/models-store.json`, refreshed 2026-09-18 21:46 when the
logins ran. It replaces the package's built-in table per provider at runtime. The built-in
table (`pi-ai/dist/providers/data/*.json`) differs only by three rows the overlay dropped
(`gpt-5.4`, `gpt-5.4-mini`, `omen-alpha`) and one it added (`deepseek-v4.1-flash`, which
settles SPEC-ADE §6 item 101: the plugin's DeepSeek row is valid at runtime).

Prices are pi's catalogue numbers, $ per million tokens in / out. Two providers bill by
subscription (ChatGPT, Kimi), so their numbers are nominal; OpenCode Go bills the key.
Context and max output in tokens. "used" = a plugin recipe row already names it.

## openai-codex (ChatGPT login, browser)

| id | name | ctx | out | in/out $ | images | used |
|---|---|---|---|---|---|---|
| gpt-5.6-luna | GPT-5.6 Luna | 272k | 128k | 0.2 / 1.2 | yes | |
| gpt-5.6-sol | GPT-5.6 Sol | 272k | 128k | 5 / 30 | yes | pi_codex_sol_high |
| gpt-5.6-terra | GPT-5.6 Terra | 272k | 128k | 2 / 12 | yes | |
| gpt-6-astra | GPT-6 Astra | 272k | 128k | 10 / 50 | yes | pi_codex_astra_xhigh |

## opencode-go (API key)

| id | name | ctx | out | in/out $ | images | used |
|---|---|---|---|---|---|---|
| deepseek-v4-flash | DeepSeek V4 Flash | 1M | 384k | 0.15 / 0.6 | no | |
| deepseek-v4-flash-vision-exp | DeepSeek V4 Flash Vision Exp | 1M | 384k | 0.15 / 0.6 | yes | |
| deepseek-v4-pro | DeepSeek V4 Pro | 1M | 384k | 0.66 / 1.98 | no | |
| deepseek-v4.1-flash | DeepSeek V4.1 Flash | 1M | 384k | 0.15 / 0.6 | yes | pi_opencode_deepseek |
| glm-5.1 | GLM-5.1 | 203k | 33k | 1.4 / 4.4 | no | |
| glm-5.2 | GLM-5.2 | 1M | 131k | 1.4 / 4.4 | no | |
| glm-5.3 | GLM-5.3 | 1M | 131k | 1.4 / 4.4 | no | |
| glm-5.3-flash | GLM-5.3-Flash | 1M | 131k | 0.15 / 0.5 | yes | |
| hy3 | Hy3 | 256k | 128k | 0.14 / 0.58 | no | |
| hy4-preview | Hy4 preview | 1M | 64k | 0.83 / 2.5 | no | |
| kimi-k2.6 | Kimi K2.6 | 262k | 66k | 0.95 / 4 | yes | |
| kimi-k2.7-code | Kimi K2.7 Code | 262k | 262k | 0.95 / 4 | yes | |
| kimi-k3 | Kimi K3 | 1M | 131k | 3 / 15 | yes | |
| longcat-2.0 | LongCat-2.0 | 1M | 131k | 0.3 / 1.2 | no | |
| mimo-v2.5 | MiMo V2.5 | 1M | 128k | 0.14 / 0.28 | yes | |
| mimo-v2.5-pro | MiMo V2.5 Pro | 1M | 128k | 0.44 / 0.87 | no | |
| minimax-m2.7 | MiniMax-M2.7 | 205k | 131k | 0.3 / 1.2 | no | |
| minimax-m3 | MiniMax-M3 | 1M | 131k | 0.3 / 1.2 | yes | |
| muse-spark-1.2-contributor | Muse Spark 1.2 Contributor | 1M | 131k | 0.1 / 0.2 | yes | |
| muse-spark-1.3-contributor | Muse Spark 1.3 Contributor | 1M | 131k | 0.1 / 0.2 | yes | pi_opencode_muse |
| qwen3.6-plus | Qwen3.6 Plus | 1M | 66k | 0.5 / 3 | yes | |
| qwen3.7-max | Qwen3.7 Max | 1M | 66k | 2.5 / 7.5 | no | |
| qwen3.7-plus | Qwen3.7 Plus | 1M | 66k | 0.4 / 1.6 | yes | |
| qwen3.8-flash | Qwen3.8 Flash | 1M | 131k | 0.15 / 0.47 | yes | |
| qwen3.8-max | Qwen3.8 Max | 1M | 131k | 2 / 6 | yes | |

## kimi-coding (Kimi login, device code)

| id | name | ctx | out | in/out $ | images | used |
|---|---|---|---|---|---|---|
| k3 | Kimi K3 | 1M | 131k | 3 / 15 | yes | pi_kimi_k3 |
| k3-256k | Kimi K3-256K | 262k | 131k | 0 / 0 (listed) | yes | |
| kimi-for-coding | kimi-for-coding | 1M | 33k | 0.95 / 4 | yes | |
| kimi-for-coding-highspeed | Kimi For Coding HighSpeed | 262k | 33k | 1.9 / 8 | yes | |

## Dropped by Rolf (2026-09-19)

Legacy on ChatGPT: `gpt-5.3-codex-spark`, `gpt-5.5`. On OpenCode Go: `gpt-5.6-luna` (GPT
rows come from ChatGPT only), `grok-4.6` (no Grok outside Cursor). Go lists no Claude row.

## Counts

33 rows in the pool: 4 ChatGPT, 25 Go, 4 Kimi; 5 named by plugin recipes today. Every row reports reasoning support. Thinking levels come from each row's
`thinkingLevelMap` (not listed here); the redo's cards read them from the same file.
