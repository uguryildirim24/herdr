# Review ui-r1

Branch `review/ui-r1`, based on `agent-parent-nesting` (90877799).

Verdict: ready with notes.

## Merged

Each lane went in with `git merge --no-ff`. None of the merges had conflicts; git auto-merged the docs, `src/main.rs`, `tests/mod.rs`, `sidebar.rs` and `agent_sidebar.rs`.

- `lane/ui-tabs` @ 2866e202: `ui.tab_bar_padding_y` (0..1, clamped with a diagnostic) and `ui.tab_bar_padding_x` (default 2).
- `lane/ui-rows` @ a1f38e05: `ui.sidebar.spaces.row_padding` and `ui.sidebar.agents.row_padding` (default 0).
- `lane/collapsed-nesting` @ 5b4d2b8c: the collapsed sidebar nests agents (tree marks plus a roll-up cell), SPEC 3.4/3.5, Q6 and Q11.

All three lane worktrees are clean, and the stash list is empty. The stash swap the lanes reported did not leak into any branch: each lane's commit holds only its own files.

## Verified against goals

- Tabs. `layout()` gives the bar 1+2*padding_y rows and falls back to 1 row when rows <= 3. Tabs, the scroll buttons and `+` fill the full height, and labels, glyphs and status segments sit on `area.y + height/2`. The drop indicator is drawn on every row. `tab_drop_index_at` accepts any bar row. At the defaults, tab width is label+4 with a minimum of 8, the same as before. Bottom placement: `render_mode_bar` draws on the last row of the area, and composition blanks the rows above it and clears the tab hits. The config is client-only (no server layout reads it). Live reload sets both keys, and the pane surface size follows.
- Rows. Spaces: paddings, heights and gaps are computed once and shared by scroll metrics, reveal and the draw loop. The highlight goes on the outer rect, text on the inner rect, hits on the outer rect. Worktree groups are padded as one block. Endpoint sidebar: same rule, headers are never padded, the focused row gets the highlight on its outer rect. Agents: `render_agent_list` adds 2*padding to every row height, and both callers (local panel, endpoint aggregate) use `render_padded_agent_row`. Live reload clones the whole `spaces`/`agents` structs, so the new keys are covered.
- Collapsed nesting. `collapsed_agent_cells` uses `nest_agents` with the shared `collapsed_groups`. Roll-up cells carry no number, so indices match `visible_agent_pane_ids` (what `FocusAgent` uses). Clicking a roll-up goes through `agent_group_toggles`. With the flag off, output is the flat `ordered_agent_pane_ids` list as before. `last_child_flags` refactor: same semantics as the old inline loop.
- Seams. The collapsed sidebar never reads `row_padding`. Nested expanded rows with padding are covered by `sidebar_row_padding`. No wire, API or protocol files changed. No `unwrap`/`expect` in added production code. Render only writes buffer and hits (scroll-state write-back is the existing pattern).

## Fixes (52b08ca4)

- Added test `compact_sidebar_ignores_row_padding_with_nesting` (`src/client/shell/tests/agent_sidebar.rs`). It checks that the compact sidebar buffer and hits are identical at row_padding 0 and 2, with nesting on, in both expanded and collapsed group states.
- Docs: added a **Collapsed sidebar** paragraph to the Agent nesting section in `configuration.mdx` and its ja/zh-cn copies. That section is still English in all three, and the new paragraph follows suit. The nesting docs still said only "sidebar Agents panel", which no longer described the compact view.

## Gates (merged tree + fixes)

CARGO_TARGET_DIR=`.local/target`, DEVELOPER_DIR=/Library/Developer/CommandLineTools.

- `cargo fmt --all --check`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0
- `cargo nextest run --locked --no-fail-fast`: exit 100. 3162 run, 3160 passed (1 leaky), 2 failed, 2 skipped. Both failures exist on the base (below).
- Targeted run (tab_bar_padding, sidebar_row_padding, compact_sidebar, last_child_flags, live reload): 22 of 22 passed
- `just maintenance-test` unittest list: 101 tests OK
- `scripts.test_ui_hot_path_architecture`: 6 tests OK
- `scripts/config_reference_check.py`: exit 0
- `scripts/docs_translation_parity.py`: exit 0
- `just docs-contract-test`: not run, because `bun` is not installed on this machine.
- `cargo build --release --locked`: exit 0, produced `.local/target/release/herdr`

Pre-existing failures. I checked them in a throwaway detached worktree of `agent-parent-nesting` with `-E 'binary(live_handoff)'`: 20 run, 18 passed, the same 2 failed. I removed that worktree afterwards.

- `herdr::live_handoff live_handoff_preserves_pane_process_io`
- `herdr::live_handoff live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`

## Notes and open questions

- Build env: zig 0.15.2 can't build libcxx against the Xcode macOS 27 SDK. Builds need `DEVELOPER_DIR=/Library/Developer/CommandLineTools`.
- Lanes must not use `git stash`, because the stash list is shared across worktrees.
- Agent padding pads every nested row separately. Tree connectors are not drawn on padding rows, so at padding >= 1 the tree lines have gaps, the same as with `row_gap`. Spaces pack worktree groups instead. If nested agent groups should pack too, `render_agent_list` needs depth info.
- `tab_bar_padding_x` has no upper bound; the math saturates.
- The compact sidebar can expand a collapsed parent (by clicking the roll-up) but has no way to collapse one. Child cells at index >= 10 show no number.
- The multi-machine sidebar still ignores `spaces.row_gap`. That was already true before these lanes.
- Visual check of the release binary in a real terminal is still pending. Nobody ran the binary during this review.
