use super::*;
use ratatui::{
    text::Line,
    widgets::{Paragraph, Widget},
};

pub(in crate::client::shell) fn collapsed_sidebar_sections(
    area: Rect,
) -> (Rect, Option<u16>, Rect) {
    let content = Rect::new(area.x, area.y, area.width.saturating_sub(1), area.height);
    if content.is_empty() {
        return (Rect::default(), None, Rect::default());
    }
    if content.height < 7 {
        return (content, None, Rect::default());
    }
    let workspace_height = content.height.div_ceil(2);
    let divider_y = content.y + workspace_height;
    let detail_height = content.height.saturating_sub(workspace_height + 1);
    (
        Rect::new(content.x, content.y, content.width, workspace_height),
        Some(divider_y),
        Rect::new(content.x, divider_y + 1, content.width, detail_height),
    )
}

pub(crate) fn render_collapsed_sidebar(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed_groups: &HashSet<String>,
    selected_workspace_id: Option<&str>,
    hits: &mut ShellHitMap,
) {
    let palette = &config.palette;
    render_sidebar_background(buffer, area, palette);
    let (workspace_area, divider_y, detail_area) = collapsed_sidebar_sections(area);
    for (index, workspace) in snapshot
        .workspaces
        .iter()
        .take(workspace_area.height as usize)
        .enumerate()
    {
        let rect = Rect::new(
            workspace_area.x,
            workspace_area.y + index as u16,
            workspace_area.width,
            1,
        );
        let selected = selected_workspace_id == Some(workspace.workspace_id.as_str());
        let selection_background =
            if workspace.focused && palette.selection_bg == ratatui::style::Color::Reset {
                palette.active_row_bg
            } else {
                palette.selection_bg
            };
        if selected {
            buffer.set_style(rect, Style::default().bg(selection_background));
        } else if workspace.focused {
            buffer.set_style(rect, Style::default().bg(palette.active_row_bg));
        }
        let number_style = if selected {
            Style::default()
                .fg(palette.overlay1)
                .bg(selection_background)
        } else if workspace.focused {
            Style::default().fg(palette.text).bg(palette.active_row_bg)
        } else {
            Style::default().fg(palette.overlay0)
        };
        put_text(
            buffer,
            rect.x,
            rect.y,
            rect.width.min(2),
            &format!("{:<2}", index + 1),
            number_style,
        );
        let status = workspace.agent_status;
        put_text(
            buffer,
            rect.x.saturating_add(2),
            rect.y,
            rect.width.saturating_sub(2),
            status_icon(status, config.status_indicators),
            Style::default().fg(status_color(status, palette)),
        );
        hits.workspaces.push(WorkspaceHit {
            rect,
            endpoint_id: ClientEndpointId::Local,
            workspace_id: workspace.workspace_id.clone(),
            indented: false,
            group_toggle: None,
        });
    }

    if let Some(divider_y) = divider_y {
        put_text(
            buffer,
            workspace_area.x,
            divider_y,
            workspace_area.width,
            &"─".repeat(workspace_area.width as usize),
            Style::default().fg(palette.surface_dim),
        );
    }

    let detail_content = Rect::new(
        detail_area.x,
        detail_area.y,
        detail_area.width,
        detail_area.height.saturating_sub(1),
    );
    let mut agent_index = 0usize;
    for (line, cell) in collapsed_agent_cells(snapshot, config, collapsed_groups)
        .into_iter()
        .take(detail_content.height as usize)
        .enumerate()
    {
        let rect = Rect::new(
            detail_content.x,
            detail_content.y + line as u16,
            detail_content.width,
            1,
        );
        match cell {
            CollapsedAgentCell::Agent {
                pane_id,
                tree_mark,
                group_key,
            } => {
                let Some(agent) = snapshot
                    .agents
                    .iter()
                    .find(|agent| agent.pane_id == pane_id)
                else {
                    continue;
                };
                agent_index += 1;
                if agent.focused && !rect.is_empty() {
                    // Like the expanded panel, the highlight runs through the sidebar
                    // separator up to the pane.
                    if let Some(cell) = buffer.cell_mut((rect.right(), rect.y)) {
                        cell.set_symbol(" ");
                    }
                    buffer.set_style(
                        Rect::new(rect.x, rect.y, rect.width.saturating_add(1), 1),
                        Style::default().bg(palette.active_row_bg),
                    );
                }
                let number_style = Style::default().fg(if agent.focused {
                    palette.text
                } else {
                    palette.overlay0
                });
                // Nested cells use the first column as the tree column: `┣`/`┗` on a child, in
                // its status color like the expanded panel's branches. The index moves to the
                // second column.
                let tree_glyph = tree_mark.map(|mark| {
                    (
                        mark,
                        Style::default()
                            .fg(status_color(agent.agent_status, palette))
                            .add_modifier(Modifier::DIM),
                    )
                });
                match tree_glyph {
                    // A child past 9 shows its two-digit index like a flat cell, in place of
                    // the tree mark.
                    Some(_) if agent_index >= 10 => put_text(
                        buffer,
                        rect.x,
                        rect.y,
                        rect.width.min(2),
                        &format!("{agent_index:<2}"),
                        number_style,
                    ),
                    Some((glyph, glyph_style)) => {
                        put_text(
                            buffer,
                            rect.x,
                            rect.y,
                            rect.width.min(1),
                            glyph,
                            glyph_style,
                        );
                        if agent_index < 10 {
                            put_text(
                                buffer,
                                rect.x.saturating_add(1),
                                rect.y,
                                rect.width.saturating_sub(1).min(1),
                                &agent_index.to_string(),
                                number_style,
                            );
                        }
                    }
                    None => put_text(
                        buffer,
                        rect.x,
                        rect.y,
                        rect.width.min(2),
                        &format!("{agent_index:<2}"),
                        number_style,
                    ),
                }
                // A parent's status mark folds and unfolds its group.
                if let Some(group_key) = group_key.filter(|_| rect.width > 2 && rect.height > 0) {
                    hits.agent_group_toggles.push((
                        Rect::new(rect.x.saturating_add(2), rect.y, 1, 1),
                        pane_id.clone(),
                        group_key,
                    ));
                }
                put_text(
                    buffer,
                    rect.x.saturating_add(2),
                    rect.y,
                    rect.width.saturating_sub(2),
                    status_icon(agent.agent_status, config.status_indicators),
                    Style::default().fg(status_color(agent.agent_status, palette)),
                );
                hits.agents.push((rect, pane_id));
            }
            CollapsedAgentCell::RollUp {
                pane_id,
                group_key,
                hidden_status_counts,
            } => {
                // The compact form of the expanded dot stack: up to three hidden statuses,
                // most urgent first, right-aligned so the last one sits in the status column.
                let width = usize::from(rect.width.min(3));
                let stack = super::super::agent_tree::status_stack(&hidden_status_counts, width)
                    .collect::<Vec<_>>();
                let x = rect.x.saturating_add((width - stack.len()) as u16);
                for (offset, status) in stack.into_iter().enumerate() {
                    put_text(
                        buffer,
                        x.saturating_add(offset as u16),
                        rect.y,
                        1,
                        status_icon(status, config.status_indicators),
                        Style::default().fg(status_color(status, palette)),
                    );
                }
                if !rect.is_empty() {
                    hits.agent_group_toggles.push((rect, pane_id, group_key));
                }
            }
        }
    }
    hits.sidebar_toggle = if area.is_empty() || workspace_area.width == 0 {
        Rect::default()
    } else {
        Rect::new(
            workspace_area.x + workspace_area.width / 2,
            area.bottom().saturating_sub(1),
            1,
            1,
        )
    };
    put_text(
        buffer,
        hits.sidebar_toggle.x,
        hits.sidebar_toggle.y,
        hits.sidebar_toggle.width,
        "»",
        if super::super::global_menu::global_menu_attention(snapshot) {
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.overlay0)
        },
    );
}

enum CollapsedAgentCell {
    Agent {
        pane_id: String,
        /// `┣` or `┗` for nested rows; `None` for top-level rows.
        tree_mark: Option<&'static str>,
        /// `collapsed_groups` key when this is a parent, so its status mark toggles the group.
        group_key: Option<String>,
    },
    /// Stands in for the hidden subtree of the collapsed parent directly above it.
    RollUp {
        pane_id: String,
        group_key: String,
        hidden_status_counts: super::super::agent_tree::StatusCounts,
    },
}

/// One-column cells for the collapsed sidebar's agent list, in the same order and with the
/// same `collapsed_groups` state as the expanded Agents panel.
fn collapsed_agent_cells(
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed_groups: &HashSet<String>,
) -> Vec<CollapsedAgentCell> {
    let ordered = super::ordered_agent_pane_ids(snapshot, config.agent_panel_sort);
    if !config.agent_parent_nesting {
        return ordered
            .into_iter()
            .map(|pane_id| CollapsedAgentCell::Agent {
                pane_id,
                tree_mark: None,
                group_key: None,
            })
            .collect();
    }
    let tree_rows =
        super::super::agent_tree::nest_agents(&ordered, snapshot, collapsed_groups, None);
    let last_child = super::super::agent_tree::last_child_flags(&tree_rows);
    let mut cells = Vec::with_capacity(tree_rows.len());
    for (row, is_last_child) in tree_rows.into_iter().zip(last_child) {
        let roll_up =
            (row.collapsed && row.hidden_descendants > 0).then(|| CollapsedAgentCell::RollUp {
                pane_id: row.pane_id.clone(),
                group_key: super::super::agent_tree::agent_group_key(None, &row.pane_id),
                hidden_status_counts: row.hidden_status_counts,
            });
        let group_key = (row.child_count > 0)
            .then(|| super::super::agent_tree::agent_group_key(None, &row.pane_id));
        cells.push(CollapsedAgentCell::Agent {
            pane_id: row.pane_id,
            tree_mark: (row.depth > 0).then_some(if is_last_child { "┗" } else { "┣" }),
            group_key,
        });
        cells.extend(roll_up);
    }
    cells
}

pub(crate) fn render_sidebar(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    state: &mut ShellRenderState<'_>,
    hits: &mut ShellHitMap,
) {
    let palette = &config.palette;
    render_sidebar_background(buffer, area, palette);
    hits.sidebar_divider = if area.is_empty() {
        Rect::default()
    } else {
        Rect::new(area.right().saturating_sub(1), area.y, 1, area.height)
    };
    let (workspace_area, detail_area) =
        crate::ui::expanded_sidebar_sections(area, state.sidebar_section_split);
    hits.sidebar_section_divider =
        crate::ui::sidebar_section_divider_rect(area, state.sidebar_section_split);
    put_text(
        buffer,
        workspace_area.x,
        workspace_area.y,
        workspace_area.width,
        " spaces",
        Style::default()
            .fg(palette.overlay0)
            .add_modifier(Modifier::BOLD),
    );

    let entries = workspace_entries(snapshot, state.collapsed_groups);
    let body = Rect::new(
        workspace_area.x,
        workspace_area.y.saturating_add(WORKSPACE_HEADER_ROWS),
        workspace_area.width,
        workspace_area
            .height
            .saturating_sub(WORKSPACE_HEADER_ROWS + 1),
    );
    hits.workspace_body = body;
    let paddings = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            space_entry_padding(
                entry.indented,
                entries.get(index + 1).is_some_and(|next| next.indented),
                config.spaces.row_padding,
            )
        })
        .collect::<Vec<_>>();
    let row_heights = entries
        .iter()
        .zip(&paddings)
        .map(|(entry, (top, bottom))| {
            snapshot
                .workspaces
                .get(entry.index)
                .map(|workspace| {
                    workspace_rows(
                        workspace,
                        displayed_workspace_status(snapshot, workspace, state.collapsed_groups),
                        entry.indented,
                        &config.spaces,
                    )
                    .len()
                    .max(1)
                    .min(u16::MAX as usize) as u16
                })
                .unwrap_or(1)
                .saturating_add(*top)
                .saturating_add(*bottom)
        })
        .collect::<Vec<_>>();
    let gaps = entries
        .iter()
        .enumerate()
        .map(|(index, _)| {
            entries
                .get(index + 1)
                .map_or(0, |next| u16::from(!next.indented) * config.spaces.row_gap)
        })
        .collect::<Vec<_>>();
    let mut metrics = super::scroll::list_scroll_metrics(
        &row_heights,
        &gaps,
        body.height,
        *state.workspace_scroll,
    );
    if !body.is_empty() && std::mem::take(state.reveal_focused_workspace) {
        if let Some(target) = entries
            .iter()
            .position(|entry| snapshot.workspaces[entry.index].focused)
        {
            *state.workspace_scroll = super::scroll::list_scroll_start_to_reveal(
                &row_heights,
                &gaps,
                body.height,
                *state.workspace_scroll,
                target,
            );
            metrics = super::scroll::list_scroll_metrics(
                &row_heights,
                &gaps,
                body.height,
                *state.workspace_scroll,
            );
        }
    }
    hits.workspace_max_scroll = metrics.max_offset_from_bottom;
    hits.workspace_scroll_metrics = Some(metrics);
    *state.workspace_scroll = metrics
        .max_offset_from_bottom
        .saturating_sub(metrics.offset_from_bottom);
    let show_scrollbar = metrics.max_offset_from_bottom > 0 && body.width > 1;
    let content_width = body.width.saturating_sub(u16::from(show_scrollbar));
    let mut y = body.y;
    for (entry_position, entry) in entries.iter().enumerate().skip(*state.workspace_scroll) {
        let Some(workspace) = snapshot.workspaces.get(entry.index) else {
            continue;
        };
        let status = displayed_workspace_status(snapshot, workspace, state.collapsed_groups);
        let rows = workspace_rows(workspace, status, entry.indented, &config.spaces);
        // Heights and gaps come from the same vectors the scroll metrics used, so the draw
        // loop cannot drift from the scroll model.
        let row_height = row_heights[entry_position].min(body.height);
        if y.saturating_add(row_height) > body.bottom() {
            break;
        }
        // `rect` is the whole padded entry: highlight and hit target. Text goes in `content`.
        let rect = Rect::new(body.x, y, content_width, row_height);
        let content = padded_content_rect(rect, paddings[entry_position]);
        let selected = state.selected_workspace_id == Some(workspace.workspace_id.as_str());
        let dragged = state.dragged_workspace_id == Some(workspace.workspace_id.as_str());
        if selected {
            buffer.set_style(rect, Style::default().bg(palette.selection_bg));
        } else if dragged {
            buffer.set_style(rect, Style::default().bg(palette.surface1));
        } else if workspace.focused {
            buffer.set_style(rect, Style::default().bg(palette.active_row_bg));
        }
        render_workspace_rows(
            buffer,
            content,
            workspace,
            status,
            config.status_indicators,
            entry,
            rows,
            true,
            selected,
            dragged,
            palette,
        );
        let group_toggle = parent_group_key(snapshot, entry.index).map(|key| {
            let rect = Rect::new(rect.right().saturating_sub(1), content.y, 1, 1);
            put_text(
                buffer,
                rect.x,
                rect.y,
                rect.width,
                if state.collapsed_groups.contains(&key) {
                    "▶"
                } else {
                    "▼"
                },
                Style::default().fg(palette.accent),
            );
            (rect, key)
        });
        hits.workspaces.push(WorkspaceHit {
            rect,
            endpoint_id: ClientEndpointId::Local,
            workspace_id: workspace.workspace_id.clone(),
            indented: entry.indented,
            group_toggle,
        });
        y = y
            .saturating_add(row_height)
            .saturating_add(gaps[entry_position]);
    }

    if show_scrollbar {
        let track = Rect::new(body.right().saturating_sub(1), body.y, 1, body.height);
        hits.workspace_scrollbar = track;
        super::scroll::render_list_scrollbar(buffer, track, metrics, palette);
    }

    if let Some(row) = state.workspace_drop_indicator_row.filter(|row| {
        *row >= workspace_area.y.saturating_add(1)
            && *row < workspace_area.bottom().saturating_sub(1)
    }) {
        put_text(
            buffer,
            body.x,
            row,
            body.width,
            &"─".repeat(body.width as usize),
            Style::default().fg(palette.accent),
        );
    }

    let footer_y = workspace_area.bottom().saturating_sub(1);
    if config.mouse_capture {
        hits.new_workspace = Rect::new(
            workspace_area.x,
            footer_y,
            5.min(workspace_area.width),
            u16::from(workspace_area.height > 0),
        );
        put_text(
            buffer,
            workspace_area.x,
            footer_y,
            workspace_area.width,
            " new",
            Style::default().fg(palette.overlay0),
        );
        let attention = super::super::global_menu::global_menu_attention(snapshot);
        let launcher_width = if attention { 8 } else { 6 }.min(workspace_area.width);
        hits.global_launcher = Rect::new(
            workspace_area.right().saturating_sub(launcher_width),
            footer_y,
            launcher_width,
            1,
        );
        if attention {
            let start_x = workspace_area.right().saturating_sub(6);
            put_text(
                buffer,
                start_x,
                footer_y,
                2,
                "● ",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            );
            put_text(
                buffer,
                start_x.saturating_add(2),
                footer_y,
                4,
                "menu",
                Style::default().fg(palette.overlay0),
            );
        } else {
            put_right_text(
                buffer,
                workspace_area,
                footer_y,
                "menu",
                Style::default().fg(palette.overlay0),
            );
        }
    }

    super::render_agent_panel(
        buffer,
        detail_area,
        snapshot,
        config,
        state.collapsed_groups,
        state.agent_scroll,
        state.agent_drag,
        hits,
    );

    hits.sidebar_toggle = Rect::new(
        area.right().saturating_sub(2),
        area.bottom().saturating_sub(1),
        u16::from(area.width > 1),
        u16::from(area.height > 0),
    );
    put_text(
        buffer,
        hits.sidebar_toggle.x,
        hits.sidebar_toggle.y,
        hits.sidebar_toggle.width,
        "«",
        Style::default().fg(palette.overlay0),
    );
}

pub(crate) fn workspace_entries(
    snapshot: &ClientShellSnapshot,
    collapsed_groups: &HashSet<String>,
) -> Vec<WorkspaceEntry> {
    let mut members = HashMap::<&str, Vec<usize>>::new();
    for (index, workspace) in snapshot.workspaces.iter().enumerate() {
        if let Some(worktree) = &workspace.worktree {
            members.entry(&worktree.key).or_default().push(index);
        }
    }
    let grouped = members
        .iter()
        .filter(|(_, indices)| {
            indices.len() >= 2
                && indices.iter().any(|index| {
                    snapshot.workspaces[*index]
                        .worktree
                        .as_ref()
                        .is_some_and(|worktree| !worktree.is_linked_worktree)
                })
        })
        .map(|(key, _)| *key)
        .collect::<HashSet<_>>();
    let mut emitted = HashSet::<&str>::new();
    let mut entries = Vec::new();
    for (index, workspace) in snapshot.workspaces.iter().enumerate() {
        let Some(worktree) = workspace
            .worktree
            .as_ref()
            .filter(|worktree| grouped.contains(worktree.key.as_str()))
        else {
            entries.push(WorkspaceEntry {
                index,
                indented: false,
                last_child: false,
            });
            continue;
        };
        if !emitted.insert(&worktree.key) {
            continue;
        }
        let Some(group_members) = members.get(worktree.key.as_str()) else {
            continue;
        };
        let parent = group_members
            .iter()
            .copied()
            .find(|member| {
                snapshot.workspaces[*member]
                    .worktree
                    .as_ref()
                    .is_some_and(|worktree| !worktree.is_linked_worktree)
            })
            .unwrap_or(index);
        entries.push(WorkspaceEntry {
            index: parent,
            indented: false,
            last_child: false,
        });
        if collapsed_groups.contains(&worktree.key) {
            if let Some(active) = group_members
                .iter()
                .copied()
                .find(|member| *member != parent && snapshot.workspaces[*member].focused)
            {
                entries.push(WorkspaceEntry {
                    index: active,
                    indented: true,
                    last_child: true,
                });
            }
            continue;
        }
        let children = group_members
            .iter()
            .copied()
            .filter(|member| *member != parent)
            .collect::<Vec<_>>();
        for (child_index, child) in children.iter().enumerate() {
            entries.push(WorkspaceEntry {
                index: *child,
                indented: true,
                last_child: child_index + 1 == children.len(),
            });
        }
    }
    entries
}

pub(super) fn parent_group_key(snapshot: &ClientShellSnapshot, index: usize) -> Option<String> {
    let workspace = snapshot.workspaces.get(index)?;
    let worktree = workspace.worktree.as_ref()?;
    if worktree.is_linked_worktree {
        return None;
    }
    (snapshot
        .workspaces
        .iter()
        .filter(|candidate| {
            candidate
                .worktree
                .as_ref()
                .is_some_and(|candidate| candidate.key == worktree.key)
        })
        .count()
        >= 2)
        .then(|| worktree.key.clone())
}

pub(super) fn displayed_workspace_status(
    snapshot: &ClientShellSnapshot,
    workspace: &ClientShellWorkspace,
    collapsed_groups: &HashSet<String>,
) -> crate::api::schema::AgentStatus {
    let Some(worktree) = workspace
        .worktree
        .as_ref()
        .filter(|worktree| !worktree.is_linked_worktree)
    else {
        return workspace.agent_status;
    };
    if !collapsed_groups.contains(&worktree.key) {
        return workspace.agent_status;
    }
    snapshot
        .workspaces
        .iter()
        .filter(|candidate| {
            candidate
                .worktree
                .as_ref()
                .is_some_and(|candidate| candidate.key == worktree.key)
        })
        .map(|candidate| candidate.agent_status)
        .max_by_key(|status| status_priority(*status))
        .unwrap_or(workspace.agent_status)
}

/// Blank rows `(above, below)` a Spaces entry for `ui.sidebar.spaces.row_padding`.
///
/// A worktree group is padded as one block, the same way `row_gap` already skips indented
/// children: the parent gets the top padding, the last visible child gets the bottom
/// padding, and nothing separates a parent from its indented children.
pub(in crate::client::shell) fn space_entry_padding(
    indented: bool,
    next_indented: bool,
    padding: u16,
) -> (u16, u16) {
    (
        if indented { 0 } else { padding },
        if next_indented { 0 } else { padding },
    )
}

/// The content lines of a padded row: `rect` minus `(top, bottom)` padding rows. When a
/// row is clamped to a body shorter than its padding, padding gives way so at least one
/// content line stays visible.
pub(in crate::client::shell) fn padded_content_rect(rect: Rect, (top, bottom): (u16, u16)) -> Rect {
    let top = top.min(rect.height.saturating_sub(1));
    let bottom = bottom.min(rect.height.saturating_sub(top).saturating_sub(1));
    Rect::new(
        rect.x,
        rect.y.saturating_add(top),
        rect.width,
        rect.height.saturating_sub(top).saturating_sub(bottom),
    )
}

pub(in crate::client::shell) fn workspace_rows(
    workspace: &ClientShellWorkspace,
    status: crate::api::schema::AgentStatus,
    indented: bool,
    config: &SpacesSidebarConfig,
) -> Vec<Vec<crate::ui::ResolvedToken>> {
    let label = if indented && !workspace.custom_label {
        workspace
            .branch
            .as_deref()
            .and_then(|branch| branch.strip_prefix("worktree/").or(Some(branch)))
            .unwrap_or(&workspace.label)
    } else {
        &workspace.label
    };
    let token_values = workspace.tokens.iter().cloned().collect::<HashMap<_, _>>();
    crate::ui::sidebar_space_rows(
        config,
        crate::ui::SpaceTokenContext {
            workspace: label,
            branch: workspace.branch.as_deref(),
            state_text: status_text(status),
            ahead_behind: workspace.git_ahead_behind,
            tokens: &token_values,
            suppress_git_details: indented,
        },
    )
}

pub(in crate::client::shell) fn render_workspace_rows(
    buffer: &mut Buffer,
    area: Rect,
    workspace: &ClientShellWorkspace,
    status: crate::api::schema::AgentStatus,
    indicators: crate::config::StatusIndicatorStyle,
    entry: &WorkspaceEntry,
    rows: Vec<Vec<crate::ui::ResolvedToken>>,
    endpoint_active: bool,
    selected: bool,
    dragged: bool,
    palette: &Palette,
) {
    for (row_index, row) in rows.iter().enumerate() {
        let y = area.y + row_index as u16;
        if y >= area.bottom() {
            break;
        }
        let mut x = area.x;
        if entry.indented {
            let prefix = if row_index == 0 {
                if entry.last_child {
                    "   └─ "
                } else {
                    "   ├─ "
                }
            } else if entry.last_child {
                "        "
            } else {
                "   │    "
            };
            x = put_segment(
                buffer,
                x,
                y,
                area.right(),
                prefix,
                Style::default().fg(palette.overlay0),
            );
        } else if row_index == 0 {
            x = x.saturating_add(1);
        } else {
            x = x.saturating_add(3);
        }
        let highlighted = endpoint_active && workspace.focused || dragged;
        let workspace_style = Style::default()
            .fg(if highlighted {
                palette.text
            } else {
                palette.subtext0
            })
            .add_modifier(if highlighted {
                Modifier::BOLD
            } else {
                Modifier::empty()
            });
        let secondary_style = Style::default().fg(if endpoint_active && workspace.focused {
            palette.mauve
        } else {
            palette.overlay0
        });
        let spans = crate::ui::resolved_token_spans(
            row,
            (
                status_icon(status, indicators),
                Style::default().fg(status_color(status, palette)),
            ),
            Style::default()
                .fg(status_color(status, palette))
                .add_modifier(Modifier::DIM),
            workspace_style,
            secondary_style,
            Style::default().fg(palette.overlay1),
            palette,
            area.right().saturating_sub(2).saturating_sub(x) as usize,
        );
        Paragraph::new(Line::from(spans)).render(
            Rect::new(x, y, area.right().saturating_sub(2).saturating_sub(x), 1),
            buffer,
        );
    }

    let background = if selected {
        Some(palette.selection_bg)
    } else if dragged {
        Some(palette.surface1)
    } else if endpoint_active && workspace.focused {
        Some(palette.active_row_bg)
    } else {
        None
    };
    if let Some(background) = background {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                buffer[(x, y)].set_bg(background);
            }
        }
    }
}
