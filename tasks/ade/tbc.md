# SPEC-ADE: to be continued (Rolf, 2026-09-18)

Two items from Rolf's Peekaboo backlog that belong to this project but not to the
current round. They are recorded here so no coordinator loses them; they become work
only when Rolf moves them out of "tbc". SPEC-ADE §6 references them as TBC-1 and TBC-2.

## TBC-1: Jev picks the model and CLI for each lane

Peekaboo `3B29B295-C066-443A-B405-36F99579D7E1`: "Jev picks the model + CLI for each herdr
lane from the brief (all installed kinds/models), instead of Rolf naming it."

Plain: today Rolf names the agent and model for every lane by a standing rule (Cursor
grok xhigh for lanes and reviewers, Claude at effort high, and so on). Jev, his ranking
model from the venator project, would read the brief and choose the kind and model from
what is installed on the machine. In the ADE this is a roles-table resolver (D2): a
`[roles] resolver = "jev"` option that runs before `thread start`, gets the brief and the
installed kinds and models, and returns the role row to use. The safety file keeps the
allowed set; Jev chooses within it. Not in the first plugin round.

## TBC-2: pro-mcp streaming through the codex-chatgpt-web infrastructure

Peekaboo `184D5AC7-C116-470A-A813-DD040999B6CC`: "replace terminal-browser streaming with
the codex-chatgpt-web streaming infra (clone github.com/miuuyy/codex-chatgpt-web again);
two Pro sessions at once make it unusable; after ADE is done, no troubleshooting before."

Plain: GPT-6 Pro runs as a real ChatGPT page inside a herdr tab, streamed by
terminal-browser. With two Pro panes open (one per build) the streaming is unreliable.
The Codex bridge Rolf already runs (`127.0.0.1:17841`, from miuuyy/codex-chatgpt-web) has
its own streaming path to ChatGPT; pro-mcp should use that instead of the browser stream.
This touches pro-mcp and the `chatgpt` agent kind's state detection in the fork, not the
plugin. Not before the ADE round; no troubleshooting of the current setup meanwhile.

## TBC-3: cloud sessions for the CLIs that have them

Rolf, 2026-09-18: "we also have to solve cloud session integration for CLIs that
support it, because when I'm running 4 spaces running their own lanes" (the message was
cut off there; the meaning: four builds with their lanes on one Mac is too much load).

Plain: some agent CLIs can run a session on their vendor's cloud instead of on this
machine (Claude Code cloud sessions today; others as they appear). A lane that runs in
the cloud costs this Mac nothing. What is known (memory, 2026-09-18, Claude Code
2.1.269): a cloud session cannot be attached as an interactive terminal; only
print-mode follow-ups and a teleport exist; cloud workers are not visible in herdr, so
builds stayed local. In the ADE this becomes a role kind `claude-cloud` (and siblings)
in the roles table: `thread start` launches the cloud session, the ticker polls its
state and mirrors it into a pane's tokens so the board and the lane tree show it, the
DONE push arrives through the plugin's inbox instead of a typed line, and the worktree
stays local (the cloud session works on a branch it pushes). Needs a per-CLI adapter and
a re-check of what each CLI's cloud mode allows at the time of building. Not in the
first plugin round.

