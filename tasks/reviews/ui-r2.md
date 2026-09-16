# Review ui-r2

Branch `lane/ui-r2`, based on `agent-parent-nesting` (76e328fe). Fixes the open rough edges listed in ui-r1 "Notes and open questions".

Verdict: ready. Not merged into `agent-parent-nesting`.

## Fixes

### 1. Tree lines with `row_padding` (and `row_gap`)

- New `agent_tree::tree_lines_below(rows) -> Vec<u8>`. For each visible row it returns a bit mask over the three clamped indentation columns. A column's bit is set when its line runs on below the row: to a later sibling, to a later sibling of an ancestor, or (depth + 1) to the row's own first visible child. It is one backward pass. Depths past 3 fold into column 3, the same way indentation clamps.
- `AgentRow` has two new fields, `tree_lines_above` (the previous row's mask) and `tree_lines_below`. Flat rows (flag off, and the aggregate multi-machine panel) keep 0, so their output doesn't change.
- `render_padded_agent_row` draws `│` on the top padding rows using the above mask and on the bottom padding rows using the below mask. The glyph and colour match the existing connectors (`overlay0` foreground). The highlight background stays.
  - Padding below a `├` row continues the line.
  - Padding below a `└` row doesn't.
  - Padding above a child connects to its parent.
  - Padding below a parent drops a line to its first child.
- `row_gap`: same code path and cheap, so it's in. The local panel's row closure draws the below mask on the gap rows, clipped to the agent body. The last row never has lines below it.
- Deviation, needed for continuity: content rows and continuation rows at depth >= 2 now draw ancestor lines in their leading columns (`│ ` instead of two blanks). Before this, lines broke at every grandchild even with padding 0. Depth-1 output doesn't change.

### 2. Compact sidebar collapse affordance (design)

- In nesting mode, the compact cell's first column is the tree column. A parent whose children are visible draws an accent `▾` there, like `▾1●`. That one column is pushed to `hits.agent_group_toggles` with the parent's `agent:<pane_id>` key. It uses the same `collapsed_groups` state and the same mouse path as the expanded chevron and the `▸N●` roll-up. So `▾` collapses and `▸` expands, both in column 0.
- The rest of the cell stays in `hits.agents`. Toggles are hit-tested first, so clicking columns 1–2 still focuses the agent.
- Cell count and order don't change, so `visible_agent_pane_ids` and `FocusAgent` still line up.
- A nested parent shows `▾` in place of its own `├`/`└`. A top-level parent's index moves from column 0 to column 1.
- Why this design: it's the mirror of the roll-up, it costs no extra rows (compact height is scarce), and it's the only free column in a 3-column cell. A separate `▾` row per group would cost vertical space.
- A parent at index >= 10 keeps `▾` and shows no number, so its only compact collapse affordance never disappears. `FocusAgent` only binds 1–9 anyway, and right-click menus are off in the compact sidebar.
- Documented in SPEC §3.5 and Q6, and in the "Collapsed sidebar" paragraph of en/ja/zh-cn `configuration.mdx`. That paragraph is English in all three, as in ui-r1.

### 3. Child numbers >= 10

The flat compact list prints `{index:<2}` in columns 0–1 (`10●`). Nested children at index >= 10 now do the same: the two-digit index replaces the tree mark, and the cell is byte-identical to the flat cell. Expanded parents at >= 10 keep `▾` (see 2).

### 4. `ui.tab_bar_padding_x` upper bound

- Added `MAX_TAB_BAR_PADDING_X = 8` and `UiConfig::tab_bar_padding_x()` (clamped).
- `tab_bar_padding_diagnostics` now returns `Vec<String>` and reports x and y separately. Callers already used `chain`/`extend`.
- Client startup and live reload read the clamped getter.
- Range and clamp are stated in `config-reference.json`, the `--default-config` template in `src/main.rs`, and the en/ja/zh-cn `configuration.mdx` sentence.

### 5. Multi-machine sidebar `row_gap`

`endpoint_sidebar::render_expanded` now builds real gaps from `spaces.row_gap`, following the local rule:
- a gap before every top-level row
- none inside a worktree group (next row indented)
- none right after an endpoint header, so a header stays attached to its first workspace
- a gap before the next header

The same `gaps` vector feeds `list_scroll_metrics` and both `y` advances in the draw loop. Hits stay on the row rects, and gap rows aren't hit targets, as in the local panel. That sidebar has no reveal logic, so there was nothing to change there.

## Tests

New:

- `agent_tree::tests::tree_lines_below_follow_open_branches`
- `src/client/shell/tests/sidebar_row_padding.rs`:
  - `padded_nested_agent_rows_keep_tree_lines_continuous`
  - `row_gap_rows_between_nested_agents_carry_tree_lines`
  - `unpadded_flat_agent_rows_draw_no_tree_lines`
  - `row_gap_applies_to_the_multi_machine_sidebar`
- `src/client/shell/tests/agent_sidebar.rs`:
  - `compact_sidebar_collapse_marker_toggles_the_shared_group_state` (collapse via `▾`, expand via roll-up, shared state with the expanded panel, clicking the rest of the cell focuses)
  - `compact_sidebar_numbers_children_past_nine_like_flat_cells`
- `src/client/shell/tests/tab_bar_padding.rs`: `oversized_horizontal_padding_is_clamped_with_diagnostic`
- `config::model::tests::tab_bar_padding_clamps_horizontal_padding`

Updated for the new design (paths are under `src/client/shell/tests/`):

- `compact_sidebar_nests_children_after_their_parent_with_tree_marks` (`▾1●`, one toggle hit)
- `compact_sidebar_marks_deeper_levels_by_their_own_siblings` (nested parent shows `▾`)
- `mouse.rs` `compact_sidebar_shares_collapse_state_with_the_agents_panel_and_focuses_children` now expects the collapse toggle after expanding. This file is outside the three named test files, but the test pinned the old "no collapse affordance" behaviour.
- `tab_bar_padding_parses_and_clamps_vertical_padding` uses the `Vec` API.

## Gates

CARGO_TARGET_DIR=`.local/target`, DEVELOPER_DIR=/Library/Developer/CommandLineTools.

- `cargo fmt --all --check`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0
- `cargo nextest run --locked --no-fail-fast`: exit 100. 3171 run, 3169 passed (2 leaky), 2 failed, 2 skipped. The failures are the known pre-existing `live_handoff_preserves_pane_process_io` and `live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session`.
- `scripts/config_reference_check.py`: exit 0
- `scripts/docs_translation_parity.py`: exit 0
- `cargo build --release --locked`: exit 0, produced `.local/target/release/herdr`

## Visual check

Session `rh-uir2` ran the debug build, with its config at the driver's `/var/tmp/rh-uir2/config.toml`, so `~/.config/herdr-dev` was not touched. Config:
- `agent_parent_nesting = true`
- `tab_bar_padding_y = 1`
- `ui.sidebar.agents.row_padding = 1`

Agents: 12 fake agents (`pane report-agent` plus `agent set-parent`):
- p1 has children p2, p3 and p5, and p3 has child p4
- p9 has children pA, pB and pC

Clicks were sent as SGR mouse sequences through tmux. The session was taken down, and `/var/tmp/rh-uir2` was removed.

Shots:
- `.local/run-herdr/shots/rh-uir2-expanded-padding.txt` (and `.ansi`): the expanded Agents panel with padding. The tree lines are continuous through the padding rows, including the depth-2 ancestor line.
- `.local/run-herdr/shots/rh-uir2-compact-expanded.txt` (and `.ansi`): the compact sidebar with groups expanded (`▾1●`, `├2●`, `▾3●`, `└4●`, …, `10●`, `11●`, `12●`).
- `.local/run-herdr/shots/rh-uir2-compact-collapsed.txt` (and `.ansi`): after clicking `▾` on p1: `1 ●`, `▸4●`, and the indices renumbered.
- `.local/run-herdr/shots/rh-uir2-numbering-ge10.txt` (and `.ansi`): re-expanded via the roll-up, then clicked the status column of `▾9●`. That focused `w1:p9` without toggling, and the >= 10 numbering is visible.

The worktree's `.claude/skills/run-herdr/driver.sh` is an untracked copy of the main checkout's driver. It has one added line: `RH_EXTRA_CONFIG` appended to the session config. It is not committed.

## Notes

- There is no CHANGELOG line. The ui-r1 lanes didn't add one either.
- The multi-machine agents panel and the multi-machine compact sidebar stay flat (Q11), so they draw no tree lines and no `▾`.
