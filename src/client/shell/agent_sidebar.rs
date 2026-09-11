use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

use super::*;

pub(super) struct AgentRow {
    pub(super) pane_id: String,
    pub(super) status: crate::api::schema::AgentStatus,
    pub(super) focused: bool,
    pub(super) rows: Vec<Vec<crate::ui::ResolvedToken>>,
    pub(super) depth: usize,
    pub(super) collapsed: bool,
    pub(super) hidden_descendants: usize,
    pub(super) worst_hidden_status: Option<crate::api::schema::AgentStatus>,
    pub(super) is_last_child: bool,
    /// `collapsed_groups` key for this row; `Some` only when the row has children.
    pub(super) group_key: Option<String>,
}

/// Pane ids in rendered order. `collapsed_groups` is `None` for surfaces that render the
/// agent list flat regardless of the nesting flag.
pub(super) fn visible_agent_pane_ids(
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed_groups: Option<&HashSet<String>>,
    machine: Option<&str>,
) -> Vec<String> {
    let ordered = ordered_agent_pane_ids(snapshot, config.agent_panel_sort);
    let Some(collapsed_groups) = collapsed_groups.filter(|_| config.agent_parent_nesting) else {
        return ordered;
    };
    super::agent_tree::nest_agents(&ordered, snapshot, collapsed_groups, machine)
        .into_iter()
        .map(|row| row.pane_id)
        .collect()
}

pub(super) fn ordered_agent_pane_ids(
    snapshot: &ClientShellSnapshot,
    sort: crate::config::AgentPanelSortConfig,
) -> Vec<String> {
    if snapshot.agent_view_label.is_some() {
        return snapshot
            .agent_order
            .iter()
            .filter(|pane_id| {
                snapshot
                    .agents
                    .iter()
                    .any(|agent| agent.pane_id == pane_id.as_str())
            })
            .cloned()
            .collect();
    }
    let mut agents = snapshot.agents.iter().collect::<Vec<_>>();
    if sort == crate::config::AgentPanelSortConfig::Priority {
        agents.sort_by_key(|agent| {
            (
                std::cmp::Reverse(status_priority(agent.agent_status)),
                std::cmp::Reverse(agent.state_change_seq),
            )
        });
    }
    agents
        .into_iter()
        .map(|agent| agent.pane_id.clone())
        .collect()
}

pub(super) fn render_agent_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed_groups: &HashSet<String>,
    agent_scroll: &mut usize,
    hits: &mut ShellHitMap,
) {
    if !render_agent_panel_header(
        buffer,
        area,
        snapshot.agent_view_label.as_deref(),
        config,
        hits,
    ) {
        return;
    }

    let rows = agent_rows(snapshot, config, Some(collapsed_groups), None);
    render_agent_list(
        buffer,
        area,
        &rows,
        snapshot
            .agent_view_label
            .as_ref()
            .map(|_| " no matching agents"),
        config,
        agent_scroll,
        hits,
        |row| row.rows.len(),
        |buffer, rect, row, hits| {
            hits.agents.push((rect, row.pane_id.clone()));
            if let Some(toggle) = render_agent_row(buffer, rect, row, config) {
                hits.agent_group_toggles.push(toggle);
            }
        },
    );
}

pub(super) fn render_agent_panel_header(
    buffer: &mut Buffer,
    area: Rect,
    agent_view_label: Option<&str>,
    config: &ClientShellConfig,
    hits: &mut ShellHitMap,
) -> bool {
    if area.height == 0 {
        return false;
    }
    put_text(
        buffer,
        area.x,
        area.y,
        area.width,
        &"─".repeat(area.width as usize),
        Style::default().fg(config.palette.surface_dim),
    );
    if area.height < 2 {
        return false;
    }
    put_text(
        buffer,
        area.x,
        area.y + 1,
        area.width,
        " agents",
        Style::default()
            .fg(config.palette.overlay0)
            .add_modifier(Modifier::BOLD),
    );
    let sort_label = agent_view_label.unwrap_or(match config.agent_panel_sort {
        crate::config::AgentPanelSortConfig::Spaces => "grouped",
        crate::config::AgentPanelSortConfig::Priority => "priority",
    });
    let sort_width = display_width(sort_label).min(area.width as usize) as u16;
    let sort_rect = Rect::new(
        area.right().saturating_sub(sort_width),
        area.y + 1,
        sort_width,
        1,
    );
    hits.agent_sort_toggle = if config.mouse_capture && agent_view_label.is_none() {
        sort_rect
    } else {
        Rect::default()
    };
    put_text(
        buffer,
        sort_rect.x,
        sort_rect.y,
        sort_rect.width,
        sort_label,
        Style::default()
            .fg(if agent_view_label.is_some() {
                config.palette.accent
            } else {
                config.palette.overlay0
            })
            .add_modifier(Modifier::BOLD),
    );
    true
}

pub(super) fn render_agent_list<T>(
    buffer: &mut Buffer,
    area: Rect,
    rows: &[T],
    empty_message: Option<&str>,
    config: &ClientShellConfig,
    agent_scroll: &mut usize,
    hits: &mut ShellHitMap,
    row_lines: impl Fn(&T) -> usize,
    mut render_row: impl FnMut(&mut Buffer, Rect, &T, &mut ShellHitMap),
) {
    let body = Rect::new(
        area.x,
        area.y.saturating_add(3),
        area.width,
        area.height.saturating_sub(3),
    );
    hits.agent_body = body;
    if body.is_empty() || rows.is_empty() {
        *agent_scroll = 0;
        if let Some(message) = empty_message.filter(|_| !body.is_empty()) {
            put_text(
                buffer,
                body.x,
                body.y,
                body.width,
                message,
                Style::default()
                    .fg(config.palette.overlay0)
                    .add_modifier(Modifier::DIM),
            );
        }
        return;
    }

    let row_heights = rows
        .iter()
        .map(|row| row_lines(row).max(1).min(u16::MAX as usize) as u16)
        .collect::<Vec<_>>();
    let gaps = rows
        .iter()
        .enumerate()
        .map(|(index, _)| {
            if index + 1 < rows.len() {
                config.agents.row_gap
            } else {
                0
            }
        })
        .collect::<Vec<_>>();
    let metrics =
        super::scroll::list_scroll_metrics(&row_heights, &gaps, body.height, *agent_scroll);
    hits.agent_max_scroll = metrics.max_offset_from_bottom;
    hits.agent_scroll_metrics = Some(metrics);
    *agent_scroll = metrics
        .max_offset_from_bottom
        .saturating_sub(metrics.offset_from_bottom);
    let show_scrollbar = metrics.max_offset_from_bottom > 0 && body.width > 1;
    let content_width = body.width.saturating_sub(u16::from(show_scrollbar));
    let mut y = body.y;
    for (index, row) in rows.iter().enumerate().skip(*agent_scroll) {
        let height = row_heights[index].min(body.height);
        if y.saturating_add(height) > body.bottom() {
            break;
        }
        let rect = Rect::new(body.x, y, content_width, height);
        render_row(buffer, rect, row, hits);
        y = y
            .saturating_add(height)
            .saturating_add(if index + 1 < rows.len() {
                config.agents.row_gap
            } else {
                0
            });
    }

    if show_scrollbar {
        let track = Rect::new(body.right().saturating_sub(1), body.y, 1, body.height);
        hits.agent_scrollbar = track;
        super::scroll::render_list_scrollbar(buffer, track, metrics, &config.palette);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_agent_row(
    pane_id: &str,
    depth: usize,
    collapsed: bool,
    hidden_descendants: usize,
    worst_hidden_status: Option<crate::api::schema::AgentStatus>,
    is_last_child: bool,
    group_key: Option<String>,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    machine: Option<&str>,
) -> Option<AgentRow> {
    let agent = snapshot
        .agents
        .iter()
        .find(|agent| agent.pane_id == pane_id)?;
    let workspace = snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == agent.workspace_id)?;
    let tab = snapshot.tabs.iter().find(|tab| tab.tab_id == agent.tab_id);
    let pane = snapshot
        .panes
        .iter()
        .find(|pane| pane.pane_id == agent.pane_id);
    let tab_count = snapshot
        .tabs
        .iter()
        .filter(|candidate| candidate.workspace_id == agent.workspace_id)
        .count();
    let tab_label = tab
        .filter(|tab| tab_count > 1 || tab.custom_label)
        .map(|tab| tab.label.as_str());
    let agent_label = agent
        .display_agent
        .as_deref()
        .or(agent.name.as_deref())
        .or(agent.agent.as_deref())
        .or(agent.title.as_deref());
    let labels = agent
        .state_labels
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    let tokens = agent.tokens.iter().cloned().collect::<HashMap<_, _>>();
    let state_text = labels
        .get(status_text(agent.agent_status))
        .map(String::as_str)
        .unwrap_or_else(|| sidebar_status_text(agent.agent_status));
    let canonical_agent = agent
        .agent
        .as_deref()
        .and_then(crate::detect::parse_agent_label);
    let rows = crate::ui::sidebar_agent_rows(
        &config.agents,
        crate::ui::AgentTokenContext {
            machine,
            workspace: &workspace.label,
            tab: tab_label,
            pane: agent
                .title
                .as_deref()
                .or_else(|| pane.and_then(|pane| pane.label.as_deref())),
            agent_label,
            terminal_title: agent.terminal_title.as_deref(),
            terminal_title_stripped: agent.terminal_title_stripped.as_deref(),
            canonical_agent,
            tokens: &tokens,
        },
        state_text,
    );
    Some(AgentRow {
        pane_id: agent.pane_id.clone(),
        status: agent.agent_status,
        focused: agent.focused,
        rows,
        depth,
        collapsed,
        hidden_descendants,
        worst_hidden_status,
        is_last_child,
        group_key,
    })
}

/// Rows in rendered order. `collapsed_groups` is `None` for surfaces that render the agent
/// list flat regardless of the nesting flag.
pub(super) fn agent_rows(
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed_groups: Option<&HashSet<String>>,
    machine: Option<&str>,
) -> Vec<AgentRow> {
    let ordered = ordered_agent_pane_ids(snapshot, config.agent_panel_sort);
    let Some(collapsed_groups) = collapsed_groups.filter(|_| config.agent_parent_nesting) else {
        return ordered
            .into_iter()
            .filter_map(|pane_id| {
                build_agent_row(
                    &pane_id, 0, false, 0, None, false, None, snapshot, config, machine,
                )
            })
            .collect();
    };

    let tree_rows = super::agent_tree::nest_agents(&ordered, snapshot, collapsed_groups, machine);
    let mut rows = Vec::with_capacity(tree_rows.len());
    for (i, tree_row) in tree_rows.iter().enumerate() {
        let is_last_child = if tree_row.depth == 0 {
            false
        } else {
            let mut last = true;
            for next in &tree_rows[i + 1..] {
                if next.depth == tree_row.depth {
                    last = false;
                    break;
                }
                if next.depth < tree_row.depth {
                    break;
                }
            }
            last
        };
        let group_key = (tree_row.child_count > 0)
            .then(|| super::agent_tree::agent_group_key(machine, &tree_row.pane_id));
        if let Some(row) = build_agent_row(
            &tree_row.pane_id,
            tree_row.depth,
            tree_row.collapsed,
            tree_row.hidden_descendants,
            tree_row.worst_hidden_status,
            is_last_child,
            group_key,
            snapshot,
            config,
            machine,
        ) {
            rows.push(row);
        }
    }
    rows
}

pub(super) fn render_agent_row(
    buffer: &mut Buffer,
    rect: Rect,
    row: &AgentRow,
    config: &ClientShellConfig,
) -> Option<(Rect, String, String)> {
    let palette = &config.palette;
    let row_style = if row.focused {
        Style::default().bg(palette.active_row_bg)
    } else {
        Style::default()
    };
    let name_style = if row.focused {
        Style::default()
            .fg(palette.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(palette.subtext0)
            .add_modifier(Modifier::BOLD)
    };
    let status_style = Style::default()
        .fg(status_color(row.status, palette))
        .add_modifier(if row.focused {
            Modifier::empty()
        } else {
            Modifier::DIM
        });
    let secondary = Style::default()
        .fg(palette.overlay0)
        .add_modifier(Modifier::DIM);
    let icon = (
        status_icon(row.status, config.status_indicators),
        Style::default().fg(status_color(row.status, palette)),
    );
    let rows = if row.rows.is_empty() {
        vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::StateIcon,
            style: Default::default(),
        }]]
    } else {
        row.rows.clone()
    };

    let clamped_depth = row.depth.min(3);
    let prefix_style = Style::default().fg(palette.overlay0);

    let toggle_info = if let Some(key) = row.group_key.clone() {
        if row.collapsed {
            let count_str = format!("{}", row.hidden_descendants);
            let worst_status = row.worst_hidden_status.unwrap_or(row.status);
            let worst_icon = status_icon(worst_status, config.status_indicators);
            let width = 4 + count_str.len() as u16;
            Some((key, true, width, count_str, worst_status, worst_icon))
        } else {
            Some((key, false, 1u16, String::new(), row.status, ""))
        }
    } else {
        None
    };

    let toggle_width = toggle_info
        .as_ref()
        .map_or(0, |(_, _, w, _, _, _)| (*w).min(rect.width));

    for (index, tokens) in rows.iter().take(rect.height as usize).enumerate() {
        let mut spans = if row.depth == 0 {
            let indent = if index == 0 { 1 } else { 3 };
            vec![ratatui::text::Span::raw(" ".repeat(indent))]
        } else {
            let leading = "  ".repeat(clamped_depth.saturating_sub(1));
            let branch = if index == 0 {
                if row.is_last_child {
                    "└─ "
                } else {
                    "├─ "
                }
            } else if row.is_last_child {
                // Continuation rows sit two columns further in than the first row, the way
                // flat agent rows and the Spaces panel's worktree children already do.
                "     "
            } else {
                "│    "
            };
            vec![ratatui::text::Span::styled(
                format!("{leading}{branch}"),
                prefix_style,
            )]
        };

        let prefix_width = spans
            .iter()
            .map(|s| display_width(&s.content))
            .sum::<usize>();
        let right_reserved = if index == 0 { toggle_width as usize } else { 0 };
        let available = (rect.width as usize).saturating_sub(prefix_width + right_reserved);

        spans.extend(crate::ui::resolved_token_spans(
            tokens,
            icon,
            status_style,
            name_style,
            secondary,
            secondary,
            palette,
            available,
        ));
        Paragraph::new(Line::from(spans)).style(row_style).render(
            Rect::new(rect.x, rect.y + index as u16, rect.width, 1),
            buffer,
        );
    }

    if let Some((key, collapsed, _, count_str, worst_status, worst_icon)) = toggle_info {
        if toggle_width == 0 {
            return None;
        }
        let toggle_rect = Rect::new(
            rect.right().saturating_sub(toggle_width),
            rect.y,
            toggle_width,
            1,
        );
        let right = toggle_rect.right();
        let mut x = put_segment(
            buffer,
            toggle_rect.x,
            toggle_rect.y,
            right,
            if collapsed { "▸" } else { "▾" },
            Style::default().fg(palette.accent),
        );
        if collapsed {
            x = put_segment(buffer, x, toggle_rect.y, right, " ", Style::default());
            x = put_segment(
                buffer,
                x,
                toggle_rect.y,
                right,
                &count_str,
                Style::default().fg(palette.overlay0),
            );
            x = put_segment(buffer, x, toggle_rect.y, right, " ", Style::default());
            put_segment(
                buffer,
                x,
                toggle_rect.y,
                right,
                worst_icon,
                Style::default().fg(status_color(worst_status, palette)),
            );
        }
        Some((toggle_rect, row.pane_id.clone(), key))
    } else {
        None
    }
}

fn put_text(buffer: &mut Buffer, x: u16, y: u16, width: u16, text: &str, style: Style) {
    for (offset, character) in text.chars().take(width as usize).enumerate() {
        if let Some(cell) = buffer.cell_mut((x + offset as u16, y)) {
            cell.set_char(character).set_style(style);
        }
    }
}

/// Writes `text` at `x`, clipped to `right`, and returns the next free column.
fn put_segment(buffer: &mut Buffer, x: u16, y: u16, right: u16, text: &str, style: Style) -> u16 {
    let width = (display_width(text) as u16).min(right.saturating_sub(x));
    put_text(buffer, x, y, width, text, style);
    x.saturating_add(width)
}

fn display_width(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(text)
}

fn sidebar_status_text(status: crate::api::schema::AgentStatus) -> &'static str {
    use crate::api::schema::AgentStatus;
    match status {
        AgentStatus::Blocked => "blocked",
        AgentStatus::Done => "done",
        AgentStatus::Working => "working",
        AgentStatus::Idle | AgentStatus::Unknown => "idle",
    }
}
