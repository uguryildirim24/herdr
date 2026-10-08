# herdr

## About this fork

Rolf's fork of [upstream Herdr](https://github.com/herdrdev/herdr) adds parent and child agent nesting, restart and resume changes, and detection for additional agent UIs.

Herdr is a Rust terminal multiplexer for coordinating coding agents. This fork keeps an orchestrator and its workers organized as a tree, including workers on other machines, and retains that lineage across restarts. Parent and child metadata is available in the CLI and API. Sidebar nesting is opt-in. This is not an upstream release.

The code includes [lineage handling](src/app/agent_parents.rs), [sidebar trees](src/client/shell/agent_tree.rs), and [session restoration](src/persist/restore.rs). These are software features, not research results. The screenshots and release badges below come from upstream. No fork benchmark or scientific result is claimed.

### Build this fork

Use a full source clone. Install Rust and Cargo, a native C linker (Xcode Command Line Tools on macOS), and **Zig 0.16.0** from [Zig's downloads](https://ziglang.org/download/). The vendored terminal library requires that Zig version. If it is not on `PATH`, set `ZIG` to its executable path. Rust/Cargo 1.96.1 were used for the build check; the fork does not declare a minimum Rust version.

```bash
git clone --branch agent-parent-nesting https://github.com/uguryildirim24/herdr.git
cd herdr
zig version                       # must print 0.16.0
cargo build --release --locked
./target/release/herdr --version
```

The first build needs network access for Cargo dependencies and Zig packages. The build check also succeeded with `--offline` after dependencies were cached. Agent programs and their credentials are separate prerequisites, not included in this repository.

For a new fork-only configuration, use a separate terminal and clear inherited Herdr variables before setup. Changing `XDG_CONFIG_HOME` alone is not isolation: socket overrides can still reach an installed server, and `HERDR_ENV` from an existing Herdr pane blocks a nested launch. The commands below clear socket, config, session, pane, and remote-client context, then select separate config and state directories. Update checks are disabled because release and detection downloads target upstream, not this branch.

```bash
unset HERDR_SOCKET_PATH HERDR_CLIENT_SOCKET_PATH HERDR_CONFIG_PATH HERDR_SESSION HERDR_ENV
unset HERDR_PANE_ID HERDR_TAB_ID HERDR_WORKSPACE_ID HERDR_BIN_PATH
unset HERDR_REATTACH_COMMAND HERDR_REMOTE_KEYBINDINGS
export XDG_CONFIG_HOME="$HOME/.config/herdr-fork"
export XDG_STATE_HOME="$HOME/.local/state/herdr-fork"
mkdir -p "$XDG_CONFIG_HOME/herdr"
# Refuse to overwrite an existing config. For reattachment, skip config creation.
(set -C; printf '%s\n' '[experimental]' 'agent_parent_nesting = true' \
  '' '[update]' 'version_check = false' 'manifest_check = false' \
  > "$XDG_CONFIG_HOME/herdr/config.toml") && ./target/release/herdr
```

Complete the first-run setup. `ctrl+b q` detaches without stopping the server. Run `./target/release/herdr` again in this same clean shell environment to reattach. For another terminal, repeat the environment setup shown below first. Do not run shutdown commands from an existing Herdr pane with inherited variables. Avoid `herdr update` and the upstream installers when running this fork.

### Parent and child agents

Sidebar nesting is off by default. The config above enables `[experimental] agent_parent_nesting = true`. Without that setting, parent metadata still exists but the sidebar stays flat.

For two detected or integration-reported agents in the same workspace, list their pane IDs, then link the child to its parent. Run these commands from the clone directory in a second terminal. Repeat the same cleanup and directory settings, even if `XDG_CONFIG_HOME` is already set. The IDs below are examples; replace them with IDs from `pane list`.

```bash
unset HERDR_SOCKET_PATH HERDR_CLIENT_SOCKET_PATH HERDR_CONFIG_PATH HERDR_SESSION HERDR_ENV
unset HERDR_PANE_ID HERDR_TAB_ID HERDR_WORKSPACE_ID HERDR_BIN_PATH
unset HERDR_REATTACH_COMMAND HERDR_REMOTE_KEYBINDINGS
export XDG_CONFIG_HOME="$HOME/.config/herdr-fork"
export XDG_STATE_HOME="$HOME/.local/state/herdr-fork"
./target/release/herdr pane list
./target/release/herdr agent set-parent w1:p2 w1:p1
./target/release/herdr pane get w1:p2
./target/release/herdr agent set-parent w1:p2 --clear
```

When finished, run `./target/release/herdr server stop` from this cleaned second terminal or the original cleaned launch shell. It stops the fork server and ends its pane processes. If opening a new terminal to stop it, repeat the cleanup and directory exports above first. This isolates server selection, not agent commands or filesystem access.

A plain shell is not a detected agent. Starting a worker with `agent start worker --kind pi --pane w1:p2 --parent w1:p1` requires Pi to be installed and an available shell pane. Herdr does not infer lineage from pane creation.

Fork-specific details are in the checked-in drafts: [configuration and nesting](docs/next/website/src/content/docs/configuration.mdx#agent-nesting-experimental), [CLI](docs/next/website/src/content/docs/cli-reference.mdx), and [restart and resume](docs/next/website/src/content/docs/session-state.mdx). `docs/next` is unreleased documentation. The website, installers, badges, and releases in the retained upstream sections below do not validate this fork.

### Project layout and local data

- `src/`: Rust runtime, server, CLI/API, TUI, detection, persistence, and SSH connections.
- `src/integration/assets/`: bundled agent integrations.
- `tests/`, `scripts/`, `justfile`: existing tests, validators, and maintenance commands.
- `vendor/`: required terminal and PTY sources with their license files.
- `docs/next/`: drafts for this source tree. `docs/preview/` and `docs/versions/`: upstream documentation snapshots.
- `distribution/`, `.github/`: upstream detection catalogs, installers, and release workflows. Review these before enabling fork publishing.
- `assets/`: upstream artwork, screenshots, and sounds.

There is no research dataset to fetch. On Unix, configuration and session data normally live in `~/.config/herdr/`, or `$XDG_CONFIG_HOME/herdr/` when set. Named sessions live under `sessions/<name>/` there. Logs also live there; state caches use `~/.local/state/herdr/` or `$XDG_STATE_HOME/herdr/`. Debug builds use `herdr-dev` instead of `herdr`.

Keep session files, terminal histories, logs, local planning notes, and credentials out of git. Local notes belong in ignored `.local/`; build and check output belongs in ignored `target/`. Pane screen history is off by default because terminal output can contain secrets. Supply agent credentials through each agent's documented environment variables or credential store, never tracked config. No repository secret is needed to build the fork.

### Validation and limits

The scope review passed a cached macOS ARM64 release build with Zig 0.16.0. The README's setup, parent assignment, clearing, detach, reattach, and stop commands passed in an isolated session. These checks used synthetic agent reports, not paid agent sessions.

Earlier cleanup checks restored shell panes and parent metadata after a cold stop. Live server restart kept a shell process and its parent token. Those checks were not repeated in the scope review. See [the verification record](docs/next/FORK_VALIDATION.md) for commands, results, and limits.

The full existing test recipe **did not pass during the earlier cleanup**. Its long temporary directory exceeds macOS socket path limits; a relative temporary directory permits more tests but breaks absolute-path assumptions. These failures remain unresolved. Synthetic agent reports were absent after live handoff, although parent tokens remained. Cross-machine behavior, official integration lifecycle preservation, native agent conversation resume, Windows, and Linux runtime behavior were not exercised here.

Native resume needs an installed supported agent, a current official Herdr integration, and a valid native session reference. A cold restart creates new processes; it does not preserve arbitrary running commands. Live handoff is a separate Unix path. Review the draft session-state document for prerequisites and failure cases.

### How this was built

AI coding agents did much of the fork implementation under Rolf's direction. Rolf chose the nested-agent workflow and restart/resume scope. The build, code inspection, privacy checks, and isolated runtime checks described here were run by coding agents. They are not a claim that Rolf personally checked every implementation detail. Rolf's final code and publication review is still required.

The fork keeps the upstream [Apache License 2.0](LICENSE), third-party licenses, and a [NOTICE of modifications](NOTICE).

## Upstream documentation

The original project overview and usage documentation follow.

<p align="center">
  <img src="assets/logo.png" alt="herdr" width="100" />
</p>

<p align="center">
  <a href="https://herdr.dev">herdr.dev</a> · <a href="#install">install</a> · <a href="https://herdr.dev/docs/quick-start/">quick start</a> · <a href="https://herdr.dev/docs/">docs</a>
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-666666?labelColor=333333" alt="Apache 2.0 license" /></a>
  <a href="https://github.com/herdrdev/herdr/releases"><img src="https://img.shields.io/github/downloads/herdrdev/herdr/total?labelColor=333333&color=666666" alt="total GitHub release downloads" /></a>
  <a href="https://github.com/herdrdev/herdr/stargazers"><img src="https://img.shields.io/github/stars/herdrdev/herdr?labelColor=333333&color=666666&logo=github" alt="GitHub stars" /></a>
  <a href="https://github.com/herdrdev/herdr/releases/latest"><img src="https://img.shields.io/github/v/release/herdrdev/herdr?label=release&labelColor=333333&color=666666" alt="latest stable release" /></a>
  <a href="https://formulae.brew.sh/formula/herdr"><img src="https://img.shields.io/homebrew/v/herdr?label=homebrew&labelColor=333333&color=666666" alt="Homebrew version" /></a>
  <a href="https://x.com/herdrdev"><img src="https://img.shields.io/badge/follow-%40herdrdev-000000?logo=x&logoColor=white" alt="follow @herdrdev on X" /></a>
</p>

---

https://github.com/user-attachments/assets/043ec09f-4bdd-41d5-aee0-8fda6b83e267

**the runtime your coding agents live on.**

- **detach without stopping work**: herdr keeps terminals running in a background server when you close the client or lose your SSH connection. after a server or machine restart, herdr restores the saved layout and can resume supported agent sessions; the original processes do not survive. [session state →](https://herdr.dev/docs/session-state/)
- **several machines, one window**: keep local work and saved ssh machines together, with a combined agent list and independent reconnects. [remote machines →](https://herdr.dev/docs/connecting-machines/)
- **never hunt for the stuck one**: every pane is marked working, blocked, or idle. when an agent stops and needs an answer, herdr says so.
- **agent-native**: agents drive herdr through the cli and socket api: they can spawn panes, prompt each other, and wait until another agent is genuinely blocked. [agent skill →](https://herdr.dev/docs/agent-skill/)
- **runs what you already run**: claude code, codex, cursor, opencode, grok and the rest. herdr doesn't wrap or replace them; it owns their terminals.
- **keyboard and mouse, both first-class**: tmux-style prefix keys *and* click, drag, split. pick per moment, not per tool.
- **plugins**: extend panes and workflows. [browse the marketplace →](https://herdr.dev/plugins/)
- **one rust binary, no electron**: runs in whatever terminal you already use.

---

## install

```bash
curl -fsSL https://herdr.dev/install.sh | sh
```

or `brew install herdr` · `mise use -g herdr` · windows: `powershell -ExecutionPolicy Bypass -c "irm https://herdr.dev/install.ps1 | iex"` · [endpoint-protected Windows](https://herdr.dev/docs/windows-beta/) · [binaries](https://github.com/herdrdev/herdr/releases)

then start it where the work lives:

```bash
herdr
```

run your agents, split panes, walk away. `ctrl+b q` detaches, `herdr` reattaches. [quick start →](https://herdr.dev/docs/quick-start/)

## docs

everything lives at [herdr.dev/docs](https://herdr.dev/docs/): [quick start](https://herdr.dev/docs/quick-start/) · [concepts](https://herdr.dev/docs/concepts/) · [supported agents](https://herdr.dev/docs/agents/) · [keyboard](https://herdr.dev/docs/keyboard/) · [configuration](https://herdr.dev/docs/configuration/) · [session state](https://herdr.dev/docs/session-state/) · [connecting machines](https://herdr.dev/docs/connecting-machines/) · [remote](https://herdr.dev/docs/persistence-remote/) · [integrations](https://herdr.dev/docs/integrations/) · [plugins](https://herdr.dev/docs/plugins/) · [socket api](https://herdr.dev/docs/socket-api/)

## thanks

every past sponsor and backer is listed in [SPONSORS.md](./SPONSORS.md), thank you 🐑

enterprise / partnership: hey@herdr.dev

## agent instructions

if you are an ai agent helping with this repository, read [`AGENTS.md`](./AGENTS.md) before making changes and read [`CONTRIBUTING.md`](./CONTRIBUTING.md) before opening issues or PRs.

## development

```bash
# First follow "Build this fork" above, including the Zig prerequisite.
# The existing recipes also need just, cargo-nextest, Python 3, Node.js, and Bun.
just test        # Rust tests, maintenance, integrations, and docs
just check       # also lint and Windows cross-compilation
```

## license

Herdr is licensed under the [Apache License 2.0](LICENSE). Rolf's fork modifications are described in [NOTICE](NOTICE).
