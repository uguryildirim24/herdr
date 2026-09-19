# The model pool (Jev picker redo, step 1)

Read 2026-09-19 from the pinned pi 0.85.1 under `~/.herdr-ade/pi`, read-only. Source: the
pi.dev overlay `~/.herdr-ade/pi/agent/models-store.json`, refreshed 2026-09-18 21:46 when the
logins ran. It replaces the package's built-in table per provider at runtime. It carries
`deepseek-v4.1-flash`, which settles SPEC-ADE §6 item 101 (the plugin's DeepSeek row is
valid at runtime). Rolf cut the list down to the rows below (2026-09-19, twice).

Prices are pi's catalogue numbers, $ per million tokens in / out. ChatGPT and Kimi bill by
subscription, so their numbers are nominal; OpenCode Go bills the key. Context and max
output in tokens. "efforts" = the thinking levels pi accepts for the row, computed by pi's
own rule (`getSupportedThinkingLevels`: a level mapped to null is out; `xhigh` and `max`
need an explicit map entry). "used" = a plugin recipe row already names it.

## openai-codex (ChatGPT login, browser)

| id | ctx | out | in/out $ | images | efforts | used |
|---|---|---|---|---|---|---|
| gpt-5.6-luna | 272k | 128k | 0.2 / 1.2 | yes | off minimal low medium high xhigh max | |
| gpt-5.6-sol | 272k | 128k | 5 / 30 | yes | off minimal low medium high xhigh max | pi_codex_sol_high |
| gpt-5.6-terra | 272k | 128k | 2 / 12 | yes | off minimal low medium high xhigh max | |
| gpt-6-astra | 272k | 128k | 10 / 50 | yes | minimal low medium high xhigh max | pi_codex_astra_xhigh |

## opencode-go (API key)

| id | ctx | out | in/out $ | images | efforts | used |
|---|---|---|---|---|---|---|
| deepseek-v4-flash | 1M | 384k | 0.15 / 0.6 | no | off low high max | |
| deepseek-v4.1-flash | 1M | 384k | 0.15 / 0.6 | yes | low high max | pi_opencode_deepseek |
| glm-5.3-flash | 1M | 131k | 0.15 / 0.5 | yes | low high max | |
| muse-spark-1.3-contributor | 1M | 131k | 0.1 / 0.2 | yes | minimal low medium high xhigh | pi_opencode_muse |
| qwen3.8-flash | 1M | 131k | 0.15 / 0.47 | yes | off minimal low medium high | |
| qwen3.8-max | 1M | 131k | 2 / 6 | yes | low medium xhigh | |

## kimi-coding (Kimi login, device code)

| id | ctx | out | in/out $ | images | efforts | used |
|---|---|---|---|---|---|---|
| k3 | 1M | 131k | 3 / 15 | yes | low high max (adaptive) | pi_kimi_k3 |
| k3-256k | 262k | 131k | listed 0 | yes | low high max (adaptive) | |

## Claude (raw models; harness is a separate Claude alias, later)

Rolf 2026-09-19: this table is about raw models and their efforts, not the harness. Claude
lanes will run through a separate Claude Code alias trimmed to the tools a worker needs, so
it behaves like pi. That alias is SPEC-ADE §6 item 115, for later. No Anthropic login in pi.
Rolf's standing rules: Opus lanes run high, never xhigh; throwaway panes run Haiku. Context,
output and price below are pi's catalogue numbers for the same ids.

| id | ctx | out | in/out $ | efforts | used |
|---|---|---|---|---|---|
| claude-fable-5-1 | 1M | 128k | 10 / 50 | minimal low medium high xhigh max | the coordinator pane |
| claude-opus-5 | 1M | 128k | 5 / 25 | minimal low medium high xhigh max | lanes at high |
| claude-sonnet-5 | 1M | 128k | 2 / 10 | off minimal low medium high xhigh max | workflows |
| claude-haiku-4-5-20251001 | 200k | 64k | 1 / 5 | off minimal low medium high | throwaway panes |

## Dropped by Rolf (2026-09-19)

ChatGPT: `gpt-5.3-codex-spark`, `gpt-5.5` (legacy). Go: `gpt-5.6-luna` (GPT rows come from
ChatGPT only), `grok-4.6` (no Grok outside Cursor), `deepseek-v4-pro`,
`deepseek-v4-flash-vision-exp`, all GLM but 5.3-flash, Hy3, Hy4, the three Kimi rows,
LongCat, both MiMo, both MiniMax, Muse 1.2, Qwen 3.6 and 3.7. Kimi: `kimi-for-coding`,
`kimi-for-coding-highspeed`. Go lists no Claude row.

## Counts

16 models: 12 through pi (4 ChatGPT, 6 Go, 2 Kimi) and 4 Claude through the alias; 5 named
by plugin recipes today. Native Codex (Pro on the Mac) and agy (research) stay outside.
