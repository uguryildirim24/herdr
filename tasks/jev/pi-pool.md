# The model pool (Jev picker redo, step 1)

Raw models only. Read 2026-09-19 from pi 0.85.1's catalogue (`~/.herdr-ade/pi/agent/
models-store.json`, refreshed 2026-09-18 21:46; Claude rows from the package's built-in
table). Prices $ per million tokens in / out. Context and max output in tokens. Efforts are
the thinking levels the catalogue accepts for the row. Rolf cut the list to these 16.

## OpenAI

| id | ctx | out | in/out $ | efforts |
|---|---|---|---|---|
| gpt-5.6-luna | 272k | 128k | 0.2 / 1.2 | off minimal low medium high xhigh max |
| gpt-5.6-sol | 272k | 128k | 5 / 30 | off minimal low medium high xhigh max |
| gpt-5.6-terra | 272k | 128k | 2 / 12 | off minimal low medium high xhigh max |
| gpt-6-astra | 272k | 128k | 10 / 50 | minimal low medium high xhigh max |

## DeepSeek, Zhipu, Meta, Alibaba

| id | ctx | out | in/out $ | efforts |
|---|---|---|---|---|
| deepseek-v4-flash | 1M | 384k | 0.15 / 0.6 | off low high max |
| deepseek-v4.1-flash | 1M | 384k | 0.15 / 0.6 | low high max |
| glm-5.3-flash | 1M | 131k | 0.15 / 0.5 | low high max |
| muse-spark-1.3-contributor | 1M | 131k | 0.1 / 0.2 | minimal low medium high xhigh |
| qwen3.8-flash | 1M | 131k | 0.15 / 0.47 | off minimal low medium high |
| qwen3.8-max | 1M | 131k | 2 / 6 | low medium xhigh |

## Moonshot

| id | ctx | out | in/out $ | efforts |
|---|---|---|---|---|
| k3 | 1M | 131k | 3 / 15 | low high max |
| k3-256k | 262k | 131k | listed 0 | low high max |

## Anthropic

| id | ctx | out | in/out $ | efforts |
|---|---|---|---|---|
| claude-fable-5-1 | 1M | 128k | 10 / 50 | minimal low medium high xhigh max |
| claude-opus-5 | 1M | 128k | 5 / 25 | minimal low medium high xhigh max |
| claude-sonnet-5 | 1M | 128k | 2 / 10 | off minimal low medium high xhigh max |
| claude-haiku-4-5-20251001 | 200k | 64k | 1 / 5 | off minimal low medium high |

## Dropped by Rolf (2026-09-19)

`gpt-5.3-codex-spark`, `gpt-5.5`, `grok-4.6`, `deepseek-v4-pro`, `deepseek-v4-flash-vision-exp`,
GLM 5.1 / 5.2 / 5.3, Hy3, Hy4, Kimi K2.6 / K2.7 Code, LongCat 2.0, MiMo V2.5 / Pro, MiniMax
M2.7 / M3, Muse Spark 1.2, Qwen 3.6 / 3.7, `kimi-for-coding`, `kimi-for-coding-highspeed`.
