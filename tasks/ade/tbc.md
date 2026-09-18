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
