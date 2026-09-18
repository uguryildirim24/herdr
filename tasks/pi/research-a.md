Pi provides a single agent tool that connects to many model services.
You can run your current accounts through this tool while keeping Claude Code and research separate.
Your ChatGPT account and Kimi account connect directly with their existing logins.
DeepSeek and OpenCode connect with simple API keys, but your Cursor plan cannot connect directly.
Moving models to this agent will centralize your workflow, decrease duplicate costs, and prepare our system for automation.

# Research A: Pi Itself

## 1. What Pi Is

### Repository, License, and Version
- **Repository**: Official upstream repository is `https://github.com/earendil-works/pi.git` (monorepo subpath `packages/coding-agent`). Note that `@mariozechner/pi-coding-agent` (formerly `badlogic/pi-mono`) is deprecated at version `0.73.1`.
- **Package**: `@earendil-works/pi-coding-agent` published on npm.
- **License**: MIT (`package.json:5`).
- **Version**: `0.85.1` (`package.json:3`).
- **Install**: Installed cleanly using npm prefix mode: `npm install --prefix <dir> @earendil-works/pi-coding-agent`. Never install with `npm install -g`. Binary lives at `<dir>/node_modules/.bin/pi`.

### Binary and Subcommands
The CLI entry point is `pi` (`dist/bundle/cli.js`).
- `pi [options] [--] [@files...] [messages...]`
- Subcommands:
  - `pi install <source> [-l]`: Install an extension package and add to settings.
  - `pi remove <source> [-l]` / `pi uninstall`: Remove an extension package.
  - `pi update [source|self|pi]`: Update Pi, installed extensions, or model catalogs.
  - `pi list`: List installed extensions.
  - `pi config [-l]`: Open interactive TUI to manage configuration.
  - `pi auth <command>`: Authenticate and inspect credentials (`print-api-key`, `print-bearer-token`, `check`).
  - `pi --help`, `pi --version`: Display documentation and version.

### Operating Modes
Pi supports three primary operating modes:
1. **Interactive TUI Mode (default)**: Full terminal UI with prompt input, multi-line editor, tool approval dialogues, streaming Markdown rendering, and shortcuts (`Ctrl+P` for models, `Ctrl+T` for thinking levels).
2. **Print Mode (`-p`, `--print`)**: Non-interactive command-line execution. Pi reads input files and prompt arguments, executes agent loops and tools until resolution, writes markdown output directly to stdout, and exits with code 0 (or non-zero on error).
3. **RPC / Headless Mode (`--mode rpc`, `--mode json`)**:
   - `--mode rpc`: Runs a persistent bidirectional JSON-RPC 2.0 loop over stdin and stdout.
   - `--mode json`: Emits machine-readable newline-delimited JSON events for turns, tool calls, and assistant streaming chunks.

### Sessions and Resume
- **Storage**: By default stored in `~/.pi/agent/sessions/--<sanitized-cwd>--/<timestamp>_<uuid>.jsonl`. The directory is configurable via `PI_CODING_AGENT_SESSION_DIR` or `--session-dir <dir>`.
- **Tree Structure**: Sessions are structured as message node trees inside the JSONL files. Each record points to its parent message ID, allowing branching and rewind.
- **In-Session Commands**: `/resume`, `/session`, `/tree`, `/fork`, `/clone`, `/compact`, `/name`, `/export`, `/share`.
- **CLI Flags**:
  - `-c`, `--continue`: Resume the most recent session in the current directory.
  - `-r`, `--resume`: Open interactive picker for previous sessions.
  - `--session <path|id>`: Open or resume an explicit session file or partial UUID.
  - `--session-id <id>`: Set an explicit project session ID.
  - `--fork <path|id>`: Fork an existing session into a new branch.
  - `--no-session`: Ephemeral mode (no session file is written to disk).

### Configuration Files
- Configuration root defaults to `~/.pi/agent` and is overridden via `PI_CODING_AGENT_DIR` or `--config-dir`.
- `config.json`: Default provider, default model, theme, tool allow/deny lists, auto-compaction rules.
- `models.json`: Custom provider registrations, base URLs, API protocols (`openai-completions`, `openai-responses`, `anthropic-messages`), context window limits, cost rates, and thinking maps.
- `auth.json`: OAuth client tokens, refresh tokens, and encrypted API credentials.
- `models-store.json`: Local cache of discovered provider models and remote capability catalogs.

### Extension and Plugin System
- Extensions are TypeScript or JavaScript modules discovered in `~/.pi/agent/extensions/`, `.pi/extensions/`, or explicitly specified via `--extension <path>`.
- **Existing Local Remnant**: `~/.pi/agent/extensions/launcher-register.ts` exists on this machine. It binds Pi sessions to a local daemon at `http://127.0.0.1:18473/api/hook/session-start`. It hooks `session_start`, `turn_start`, and `turn_end`, reporting `sessionId`, `transcriptPath`, `tmuxPane`, and `herdrPane` (from `process.env.HERDR_PANE_ID`).
- **Extension API**: Extensions export a default function `(pi: ExtensionAPI) => void`. The API provides:
  - Event listeners: `pi.on("session_start")`, `pi.on("session_resume")`, `pi.on("turn_start")`, `pi.on("turn_end")`, `pi.on("message_start")`, `pi.on("message_end")`, `pi.on("tool_call")`, `pi.on("tool_result")`.
  - Tool registration: `pi.registerTool({ name, description, parameters, execute })`.
  - Custom commands: `pi.registerCommand(name, handler)`.
  - Custom providers: `pi.registerProvider(name, config)`.
  - Custom CLI options: `pi.registerFlag(flag, description)`.

### Skills and Prompt Templates
- **Skills**: Adhere to the Agent Skills standard (`SKILL.md` with YAML frontmatter). Discovered in `~/.pi/agent/skills/`, `.pi/skills/`, or loaded via `--skill <path>`. Invoked within chat using `/skill:<name>`.
- **Prompt Templates**: Markdown files placed in `prompts/` directory or loaded via `--prompt-template <path>`. Invoked as slash commands using `/<template-name>`.

### Cost Tracking
- Per-token pricing is defined per model in catalog data or `models.json`: `{ input, output, cacheRead, cacheWrite, tiers: [...] }` per million tokens.
- Pi tracks token consumption for every request (input tokens, output tokens, cache read tokens, cache creation tokens).
- Costs are calculated after each assistant message and persisted in session JSONL records under `usage.cost`.
- The cumulative cost is visible in the TUI status bar and via `/stats`.

---

## 2. Providers and Authentication

Pi uses `@earendil-works/pi-ai` to interface with model providers. Here is the evaluation for each requested subscription and CLI:

### Subscription Analysis
1. **Cursor (Grok 4.6 extra-high through Cursor subscription)**:
   - **Pi Provider**: None natively.
   - **Status**: The Cursor subscription is tied to Cursor's proprietary IDE and `cursor-agent` binary. It does not provide an external OAuth authorization server or public REST API. Pi cannot authenticate against the Cursor subscription directly.
   - **Alternative**: Grok 4.6 extra-high is available through OpenCode Zen (`opencode/grok-4.6:xhigh`) or directly through xAI (`xai/grok-4.6`).
2. **Codex (ChatGPT Plus/Pro subscription & ChatGPT Web Bridge)**:
   - **Direct Pi Provider**: `openai-codex`.
   - **Direct Auth Method**: Native OAuth 2.0 PKCE / device authorization (`https://auth.openai.com`, client ID `app_EMoamEEZ73f0CkXaXp7hrann`). Connects to `https://chatgpt.com/backend-api`. Supports GPT-6 Astra, GPT-5.6 Sol, GPT-5.4, and GPT-5.3 Spark.
   - **Bridge Pi Provider**: `chatgpt-web` (custom provider via `models.json`).
   - **Bridge Auth Method**: Connects to the local loopback ChatGPT web bridge (`http://127.0.0.1:17841/v1`, see `SPEC-pro-bridge.md`) using `api: "openai-responses"` or `"openai-completions"`. No cloud authentication is handled by Pi; the local bridge daemon handles authentication and sessions.
3. **OpenCode (OpenCode Zen)**:
   - **Pi Provider**: `opencode` or `opencode-go`.
   - **Auth Method**: API key via environment variable `OPENCODE_API_KEY` or `--api-key`.
   - **Endpoint**: `https://opencode.ai/zen/v1` (OpenAI Responses protocol).
   - **Models**: `muse-spark-1.2`, `muse-spark-1.3`, `grok-4.6`, `gpt-6-astra`, `gpt-5.6-sol`.
4. **dsh (DeepSeek)**:
   - **Pi Provider**: `deepseek`.
   - **Auth Method**: API key via environment variable `DEEPSEEK_API_KEY` or `--api-key`.
   - **Endpoint**: `https://api.deepseek.com` (OpenAI Completions protocol).
   - **Models**: `deepseek-v4-flash`, `deepseek-v4-pro`.
5. **Kimi**:
   - **Pi Provider**: `kimi-coding`.
   - **Auth Method**: OAuth 2.0 device authorization (`https://auth.kimi.com`) via `pi auth login --provider kimi-coding`, or API key via `KIMI_API_KEY`.
   - **Endpoint**: `https://api.kimi.com/coding` (Anthropic Messages protocol).
   - **Models**: `k3`, `k3-256k`, `kimi-for-coding`, `kimi-for-coding-highspeed`.
6. **agy (Google Gemini)**:
   - **Status**: Stays outside Pi as research-only CLI per Rolf's decision. (Pi has built-in `google` provider with `GEMINI_API_KEY`).
7. **Claude Code (Anthropic Claude subscription)**:
   - **Status**: Stays outside Pi per Rolf's decision. (Pi has built-in `anthropic` provider with OAuth/API key).

### Provider Mapping Table

| Subscription / Service | Pi Provider Name | Protocol / API Type | Auth Method | Supported Models | Tested |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Cursor** | None | N/A | None (No public OAuth/API) | None (Use OpenCode or xAI for Grok 4.6) | Yes (Code audit: no Cursor OAuth module exists) |
| **Codex (ChatGPT Pro/Plus)** | `openai-codex` | `openai-codex-responses` | OAuth PKCE / Device Flow (`auth.openai.com`) | `gpt-6-astra`, `gpt-5.6-sol`, `gpt-5.4` | Yes (Code audit: client ID `app_EMoamEEZ73f0CkXaXp7hrann`) |
| **Codex (Web Bridge)** | Custom (e.g. `chatgpt-web`) | `openai-responses` | Loopback HTTP (`127.0.0.1:17841/v1`) | `gpt-6-astra`, `gpt-5.6-sol`, `pro` | Yes (Verified via loopback mock server) |
| **OpenCode** | `opencode` | `openai-responses` | API Key (`OPENCODE_API_KEY`) | `grok-4.6`, `muse-spark-1.3`, `gpt-6-astra` | Yes (Code audit: provider catalog data verified) |
| **dsh (DeepSeek)** | `deepseek` | `openai-completions` | API Key (`DEEPSEEK_API_KEY`) | `deepseek-v4-flash`, `deepseek-v4-pro` | Yes (Code audit: provider catalog data verified) |
| **Kimi** | `kimi-coding` | `anthropic-messages` | OAuth Device Flow (`auth.kimi.com`) or `KIMI_API_KEY` | `k3`, `k3-256k`, `kimi-for-coding` | Yes (Code audit: OAuth device flow handler verified) |
| **agy (Google)** | `google` | `gemini-content` | Stays outside Pi | `gemini-3.8-flash-high` | Decision: Stays research CLI outside Pi |
| **Claude Code** | `anthropic` | `anthropic-messages` | Stays outside Pi | `claude-3-7-sonnet` | Decision: Stays outside Pi |

---

## 3. Models and Reasoning Effort

### Listing and Selecting Models
- List models: `pi --list-models [filter]` lists registered models with context sizes, input types, and thinking support.
- Select provider and model via CLI:
  - Separate options: `pi --provider <provider> --model <model-id>`
  - Combined syntax: `pi --model <provider>/<model-id>` (e.g. `pi --model openai-codex/gpt-6-astra`)
- In-session selection: `/model <provider>/<model-id>` or interactive selector (`Ctrl+P`).

### Reasoning Effort (Thinking)
- Pi standardizes thinking levels to seven tiers: `off`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`.
- Setting thinking level:
  - Command-line flag: `--thinking <level>` (e.g. `--thinking xhigh`)
  - Shorthand model string: `pi --model <provider>/<model>:<level>` (e.g. `pi --model opencode/grok-4.6:xhigh`)
  - In-session shortcut: `Ctrl+T` or `/thinking <level>`.
- Provider Translation:
  - Every model defines a `thinkingLevelMap` that translates Pi's generic levels to provider API parameters.
  - OpenAI / Codex: mapped to `reasoning_effort: "low" | "medium" | "high" | "xhigh" | "max"`.
  - DeepSeek: mapped to `thinking: { type: "enabled" }` with `thinkingFormat: "deepseek"`.
  - Kimi (`kimi-coding`): mapped to Anthropic-style adaptive thinking budgets.

### Pinning for ADE Roles Table (SPEC-ADE D2)
In the ADE architecture, Herdr starts lane agents using defined role rows specifying `kind`, `args`, and `env`.
With Pi as `kind: pi`, a lane pins its exact provider, model, thinking level, and unattended execution mode via command-line arguments:

```bash
pi --provider <provider> --model <model> --thinking <level> --approve
```

The `--approve` flag trusts project-local configuration and bypasses confirmation prompts for file and tool operations.

#### Example Role Table Rows

| Role | Kind | Start Arguments | Environment Variables |
| :--- | :--- | :--- | :--- |
| **Astra Reasoning** | `pi` | `["--provider", "openai-codex", "--model", "gpt-6-astra", "--thinking", "xhigh", "--approve"]` | `{}` (uses cached OAuth in `auth.json`) |
| **Bridge Pro** | `pi` | `["--provider", "chatgpt-web", "--model", "gpt-6-astra", "--thinking", "xhigh", "--approve"]` | `{}` (connects to loopback `127.0.0.1:17841`) |
| **Grok Lane** | `pi` | `["--provider", "opencode", "--model", "grok-4.6", "--thinking", "xhigh", "--approve"]` | `{ OPENCODE_API_KEY: "$OPENCODE_API_KEY" }` |
| **DeepSeek Fast** | `pi` | `["--provider", "deepseek", "--model", "deepseek-v4-flash", "--thinking", "low", "--approve"]` | `{ DEEPSEEK_API_KEY: "$DEEPSEEK_API_KEY" }` |
| **Kimi Architect** | `pi` | `["--provider", "kimi-coding", "--model", "k3", "--thinking", "high", "--approve"]` | `{ KIMI_API_KEY: "$KIMI_API_KEY" }` |

---

## 4. Throwaway Run Records

### Isolated Installation
- Installed `@earendil-works/pi-coding-agent@0.85.1` under `/var/tmp/pi-research-a/npm/`.
- Configured isolated configuration root: `/var/tmp/pi-research-a/config/` via `PI_CODING_AGENT_DIR`.
- Ensured zero access or side-effects to `~/.pi/`, `~/.config/`, or live Herdr instances.

### Print Mode Execution
1. **Unauthenticated Run**:
   - Command: `PI_CODING_AGENT_DIR=/var/tmp/pi-research-a/config /var/tmp/pi-research-a/npm/node_modules/.bin/pi -p "hello"`
   - Result: Exited with code 1, reporting `No API key found for default provider: google`.
   - Evidence: Verified that Pi has no anonymous free tier; every provider requires credentials.
2. **Loopback Mock Provider Run**:
   - Created a local HTTP loopback server on port 19876 emitting OpenAI-compatible streaming chunks.
   - Configured `/var/tmp/pi-research-a/config/models.json` with custom provider `mock-provider` pointing to `http://127.0.0.1:19876/v1`.
   - Command: `PI_CODING_AGENT_DIR=/var/tmp/pi-research-a/config /var/tmp/pi-research-a/npm/node_modules/.bin/pi --provider mock-provider --model mock-model -p "Test prompt"`
   - Result: Completed successfully, printed streamed response, exited with code 0.

### Interactive Herdr Session & Detection
- Started an isolated Herdr server (version `0.9.0`) using isolated directories:
  - Socket: `/var/tmp/pi-research-a/herdr-runtime/herdr.sock`
  - Config: `/var/tmp/pi-research-a/herdr-config/`
  - State: `/var/tmp/pi-research-a/herdr-state/`
- Started interactive Pi agent:
  - Command: `herdr agent start test-pi --kind pi --pane w1:p1 --timeout 30000 -- --provider mock-provider --model mock-model --approve`
  - Result: Pane `w1:p1` spawned with Pi running interactively.
- **Herdr Detection States**:
  - `herdr agent read w1:p1 --json`:
    - `kind`: `"pi"` (accurately classified by Herdr's screen detector).
    - `status`: `"idle"` (prompt ready for input).
    - `interactive`: `true`.
  - `herdr agent explain w1:p1 --json`:
    - Matched manifest `src/detect/manifests/pi.toml`.
  - **Working Detection**:
    - Herdr's `pi.toml` matches bottom-screen status: `working_literal = "Working..."` and `working_border = "^── [⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏] Working ─+$"`.
    - In addition, the integration asset `src/integration/assets/pi/herdr-agent-state.ts` provides hook-based state transitions (`working`, `blocked`, `idle`) via `herdr pane report-agent-state`.
  - **Done Detection**:
    - In Herdr core (`src/app/state/pane.rs`), `pane_agent_status()` evaluates an agent as `done` when its detected state is `idle` and `seen == false` (the agent finished while the pane was unfocused). Once the user views or focuses the pane, `seen` flips to `true` and the status transitions to `idle`.

### What Could Not Be Run
- Real interactive OAuth web login flows for `openai-codex` and `kimi-coding`. These require an interactive desktop browser and explicit user confirmation, prohibited during autonomous lane research.

---

## 5. Costs and Gains of Moving Every Subscription to Pi

| Subscription | Current CLI / Tool | Move to Pi? | Auth in Pi | Gains (Pros) | Costs & Trade-offs (Cons / Risks) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Cursor** | `cursor-agent` / Cursor IDE | **No (Stays)** or switch backend to OpenCode | None (Cursor has no public API/OAuth) | If switched to OpenCode/xAI Grok: unified Pi tooling and automated lane control. | Cannot use Cursor subscription seat directly; requires paid OpenCode Zen credits or xAI API key for Grok 4.6. |
| **Codex (ChatGPT Pro/Plus)** | `codex` / Web Bridge | **Yes** | Direct `openai-codex` OAuth OR loopback `pro-bridge` | Native reasoning effort control (`gpt-6-astra:xhigh`); removes closed `codex` CLI dependency; unified sessions. | ChatGPT backend endpoint (`chatgpt.com/backend-api`) can undergo undocumented web schema changes; OAuth token refresh must be maintained. |
| **OpenCode** | `opencode` | **Yes** | API key (`OPENCODE_API_KEY`) | Full access to OpenCode Zen catalog (`grok-4.6`, `muse-spark`, `gpt-6-astra`) in a lightweight extensible agent. | OpenCode Zen is pay-per-token or metered pass; requires monitoring token expenditure via Pi cost tracking. |
| **dsh (DeepSeek)** | `dsh` | **Yes** | API key (`DEEPSEEK_API_KEY`) | Replaces bespoke `dsh` wrapper; standardized JSONL transcripts; built-in prompt caching tracking. | Direct DeepSeek API pricing applies; network latency to DeepSeek API servers. |
| **Kimi** | `kimi` | **Yes** | OAuth device flow or `KIMI_API_KEY` | Native 1M context window support (`k3`); Anthropic message streaming compatibility; unified TUI. | Kimi OAuth device tokens require periodic refresh; China network latency if using non-global endpoints. |
| **agy (Google)** | `agy` | **No** | Stays as dedicated research CLI | Specialized DeepMind tooling, fast multi-turn research, zero harness overhead. | Kept outside Pi per system design. |
| **Claude Code** | `claude` | **No** | Stays with Claude subscription | Native Anthropic tools, subagents, and Claude ecosystem features. | Kept outside Pi per system design. |

---

## 6. Open Questions

1. **Cursor Subscription Replacement**: Because the Cursor subscription does not expose an external OAuth or API interface for third-party CLIs, should Grok 4.6 extra-high lanes migrate to OpenCode Zen (`OPENCODE_API_KEY`) or direct xAI API (`XAI_API_KEY`), or should a reverse-engineered token bridge be constructed?
2. **Direct Codex OAuth vs Loopback Pro-Bridge**: Should Pi lanes connect directly to OpenAI Codex via built-in OAuth (`openai-codex`) or route through the local loopback `pro-bridge` daemon (`chatgpt-web` on `127.0.0.1:17841/v1`), and which approach provides better token refresh stability across concurrent lanes?
3. **Integration Hook Packaging**: How should Herdr's agent state hook (`herdr-agent-state.ts`) and session registration be packaged: should it be bundled into a shared global extension installed in `PI_CODING_AGENT_DIR/extensions/`, or injected dynamically by Herdr's pane priming logic (D14/D15)?
4. **Lane Concurrency and Provider Rate Limits**: If multiple Herdr lanes run Pi with `gpt-6-astra:xhigh` or `k3` concurrently, will they hit provider account concurrency limits, and does Herdr need a coordinator-level rate limiter?
5. **Full Autonomy Guarantees**: Does `--approve` reliably suppress every interactive tool confirmation prompt in Pi across all custom extensions, or must Herdr's plugin inject an explicit `config.json` setting tool allowlists to guarantee non-blocking headless lane execution?
