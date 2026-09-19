# Artificial Analysis scores for the pool, per effort row

Pulled 2026-09-19 from the Artificial Analysis data API (`/api/v2/data/llms/models`), filtered to
the 16 pool models and their effort rows. Raw rows in `aa-scores.json`. Key lives in
`~/.config/artificialanalysis/api_key`, never in the repo. Attribution: Artificial Analysis.

Columns: intel = Intelligence Index; coding = Coding Index (Terminal-Bench Hard and SciCode);
TB2.1 = Terminal-Bench 2.1 %; SciCode %; LCR = long-context reasoning %; $ = blended price per
million tokens (3:1 in:out); tok/s = median output speed; ttfa = median seconds to first answer
token. A slug with no effort suffix is the vendor default (adaptive thinking where the model
has it). Dated slugs (`-0420`, `-0803`) are older snapshots.

| pool id | AA slug | intel | coding | TB2.1 | SciCode | LCR | $ | tok/s | ttfa s |
|---|---|---|---|---|---|---|---|---|---|
| claude-fable-5-1 | claude-fable-5-1 | 53.4 | 81.6 | 91 | 63 | 85 | 20 | 69.665 | 141.332 |
| claude-fable-5-1 | claude-fable-5-1-high | 51.2 | 79.1 | 90 | 59 | 84 | 20 | 56.252 | 11.222 |
| claude-fable-5-1 | claude-fable-5-1-low | 47 | 75.2 | 85 | 57 | 82 | 20 | 50.784 | 4.683 |
| claude-fable-5-1 | claude-fable-5-1-medium | 49.1 | 77.1 | 88 | 56 | 85 | 20 | 51.265 | 6.626 |
| claude-fable-5-1 | claude-fable-5-1-xhigh | 53.2 | 80.7 | 91 | 61 | 83 | 20 | 56.847 | 30.567 |
| claude-haiku-4-5-20251001 | claude-4-5-haiku | 15.4 | - | - | - | 50 | 2 | 96.551 | 0.513 |
| claude-haiku-4-5-20251001 | claude-4-5-haiku-reasoning | 17.6 | 43.9 | 44 | 42 | 74 | 2 | 107.693 | 16.851 |
| claude-opus-5 | claude-opus-5 | 50.7 | 78 | 89 | 56 | 79 | 10 | 52.928 | 30.197 |
| claude-opus-5 | claude-opus-5-high | 48.2 | 76.5 | 88 | 55 | 79 | 10 | 50.321 | 8.606 |
| claude-opus-5 | claude-opus-5-low | 39.8 | 66.9 | 76 | 49 | 81 | 10 | 50.95 | 2.288 |
| claude-opus-5 | claude-opus-5-medium | 45.1 | 74.3 | 86 | 52 | 82 | 10 | 49.135 | 4.517 |
| claude-opus-5 | claude-opus-5-xhigh | 49.7 | 77 | 88 | 56 | 80 | 10 | 50.834 | 19.531 |
| claude-sonnet-5 | claude-sonnet-5 | 38.4 | 71.5 | 81 | 54 | 82 | 4 | 79.721 | 119.929 |
| claude-sonnet-5 | claude-sonnet-5-high | 32 | - | - | 54 | 77 | 4 | 64.6 | 2.401 |
| claude-sonnet-5 | claude-sonnet-5-low | 24.7 | - | - | 50 | 67 | 4 | 63.979 | 1.13 |
| claude-sonnet-5 | claude-sonnet-5-medium | 28.4 | - | - | 52 | 74 | 4 | 64.886 | 1.908 |
| claude-sonnet-5 | claude-sonnet-5-non-reasoning | 28.9 | 66.4 | 75 | - | 70 | 4 | 60.766 | 1.083 |
| claude-sonnet-5 | claude-sonnet-5-xhigh | 34.7 | - | - | 54 | 77 | 4 | 72 | 7.461 |
| deepseek-v4-flash | deepseek-v4-flash | 34.5 | 69.1 | 79 | 50 | 80 | 0.66 | 234.466 | 9.426 |
| deepseek-v4-flash | deepseek-v4-flash-0420 | 24.6 | 56.2 | 62 | 45 | 74 | 0.168 | 0 | 0 |
| deepseek-v4.1-flash | deepseek-v4-1-flash | 39.5 | - | - | 52 | 84 | 0.525 | 220.996 | 9.84 |
| glm-5.3-flash | glm-5-3-flash | 41.9 | 71.5 | 84 | 52 | 80 | 0.237 | 100.713 | 22.167 |
| gpt-5.6-luna | gpt-5-6-luna | 37.5 | 71.4 | 81 | 54 | 84 | 0.45 | 130.194 | 109.701 |
| gpt-5.6-luna | gpt-5-6-luna-high | 32.4 | 63.3 | 70 | 52 | 80 | 0.45 | 123.29 | 8.435 |
| gpt-5.6-luna | gpt-5-6-luna-low | 21.5 | 44.2 | 43 | 46 | 70 | 0.45 | 123.788 | 1.332 |
| gpt-5.6-luna | gpt-5-6-luna-medium | 25.5 | 50.7 | 53 | 47 | 75 | 0.45 | 113.308 | 2.564 |
| gpt-5.6-luna | gpt-5-6-luna-non-reasoning | 16.1 | 39.3 | 39 | 40 | 43 | 0.45 | 119.808 | 0.903 |
| gpt-5.6-luna | gpt-5-6-luna-xhigh | 34.8 | 68.6 | 78 | 50 | 82 | 0.45 | 136.227 | 30.31 |
| gpt-5.6-sol | gpt-5-6-sol | 47.1 | 77.4 | 88 | 57 | 84 | 8 | 60.334 | 71.18 |
| gpt-5.6-sol | gpt-5-6-sol-high | 42.5 | 77.2 | 87 | 58 | 82 | 8 | 57.189 | 9.573 |
| gpt-5.6-sol | gpt-5-6-sol-low | 33.8 | 69.7 | 77 | 56 | 78 | 8 | 56.293 | 2.719 |
| gpt-5.6-sol | gpt-5-6-sol-medium | 39.5 | 76.3 | 86 | 57 | 80 | 8 | 60.42 | 7.318 |
| gpt-5.6-sol | gpt-5-6-sol-non-reasoning | 28.3 | 65.1 | 74 | 48 | 62 | 8 | 58.246 | 1.223 |
| gpt-5.6-sol | gpt-5-6-sol-xhigh | 44.1 | 78.3 | 90 | 57 | 82 | 8 | 62.316 | 25.788 |
| gpt-5.6-terra | gpt-5-6-terra | 42.3 | 76.7 | 88 | 55 | 83 | 4.5 | 84.785 | 128.105 |
| gpt-5.6-terra | gpt-5-6-terra-high | 34.5 | 67.1 | 76 | 52 | 78 | 4.5 | 66.887 | 2.87 |
| gpt-5.6-terra | gpt-5-6-terra-low | 27.9 | 58.1 | 63 | 50 | 71 | 4.5 | 68.41 | 1.922 |
| gpt-5.6-terra | gpt-5-6-terra-medium | 30.4 | 64.7 | 72 | 50 | 74 | 4.5 | 69.46 | 2.06 |
| gpt-5.6-terra | gpt-5-6-terra-non-reasoning | 21.2 | 52.3 | 56 | 45 | 59 | 4.5 | 68.468 | 0.852 |
| gpt-5.6-terra | gpt-5-6-terra-xhigh | 38.2 | 70.6 | 80 | 52 | 79 | 4.5 | 67.043 | 6.877 |
| gpt-6-astra | gpt-6-astra | 52.8 | 76.9 | 88 | 56 | 81 | 20 | 55.975 | 214.468 |
| gpt-6-astra | gpt-6-astra-high | 51 | 77.1 | 90 | 55 | 80 | 20 | 53.026 | 28.269 |
| gpt-6-astra | gpt-6-astra-low | 46 | 75.7 | 88 | 54 | 80 | 20 | 50.219 | 1.731 |
| gpt-6-astra | gpt-6-astra-medium | 49.7 | 76.7 | 90 | 54 | 80 | 20 | 49.929 | 4.049 |
| gpt-6-astra | gpt-6-astra-xhigh | 52.5 | 75.9 | 89 | 56 | 80 | 20 | 56.399 | 70.491 |
| k3 / k3-256k | kimi-k3 | 43.8 | 76.2 | 85 | 60 | 89 | 6 | 37.412 | 57.112 |
| k3 / k3-256k | kimi-k3-low | 30.5 | 72 | 82 | 53 | 79 | 6 | 35.966 | 58.999 |
| muse-spark-1.3-contributor | muse-spark-1-3 | 48.2 | 75.8 | 84 | 59 | 83 | 2 | 234.747 | 32.548 |
| muse-spark-1.3-contributor | muse-spark-1-3-xhigh | 45.2 | 76.5 | 85 | 60 | 83 | 2 | 211.267 | 30.19 |
| qwen3.8-flash | qwen3-8-flash-next | 39.9 | 73.1 | 86 | 51 | 80 | 0.23 | 50.751 | 40.796 |
| qwen3.8-max | qwen3-8-max | 45.4 | 76.2 | 89 | 52 | 80 | 3 | 40.104 | 51.555 |
| qwen3.8-max | qwen3-8-max-0803 | 40.3 | 71.8 | 81 | 53 | 78 | 3 | 41.94 | 49.407 |

## Not in this feed

SWE-bench (any flavour) is not an Artificial Analysis evaluation. LiveCodeBench is empty for
every pool row except Haiku. Terminal-Bench Hard is filled only for Sol, Terra, Haiku and the
DeepSeek 0420 snapshot. K3 has a default and a low row only; K3-256K has no row of its own.
Muse has a default and an xhigh row. The DeepSeek V4.1 Flash row has no coding index yet.
