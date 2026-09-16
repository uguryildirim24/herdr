use std::collections::{HashMap, HashSet};

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
    /// Tree lines passing between the previous visible row and this one, and between this
    /// row and the next (see `agent_tree::tree_lines_below`). Zero outside nesting.
    pub(super) tree_lines_above: u8,
    pub(super) tree_lines_below: u8,
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
        |row| row,
        |buffer, rect, row, hits| {
            hits.agents.push((rect, row.pane_id.clone()));
            if let Some(toggle) = render_padded_agent_row(buffer, rect, row, config) {
                hits.agent_group_toggles.push(toggle);
            }
            // `row_gap` rows below a nested row carry the same lines as its bottom padding.
            // The last row never has lines below it, so no gap is drawn past the list.
            let gap_end = rect
                .bottom()
                .saturating_add(config.agents.row_gap)
                .min(hits.agent_body.bottom());
            render_tree_lines(
                buffer,
                rect,
                rect.bottom()..gap_end,
                row.tree_lines_below,
                config,
            );
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
    agent_row: impl Fn(&T) -> &AgentRow,
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

    // `ui.sidebar.agents.row_padding` pads every agent row the same way `row_gap` spaces
    // every agent row, except that nested rows drop their top padding (see
    // `agent_row_padding`); `render_row` receives the padded rect.
    let row_heights = rows
        .iter()
        .map(|row| {
            let row = agent_row(row);
            let (top, bottom) = agent_row_padding(row, config);
            (row.rows.len().max(1).min(u16::MAX as usize) as u16)
                .saturating_add(top)
                .saturating_add(bottom)
        })
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
    parent_pane_id: Option<&str>,
    depth: usize,
    collapsed: bool,
    hidden_descendants: usize,
    worst_hidden_status: Option<crate::api::schema::AgentStatus>,
    is_last_child: bool,
    tree_lines: (u8, u8),
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
    let mut rows = crate::ui::sidebar_agent_rows(
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
    let parent_workspace = parent_pane_id.and_then(|parent| {
        snapshot
            .agents
            .iter()
            .find(|candidate| candidate.pane_id == parent)
            .map(|parent| parent.workspace_id.as_str())
    });
    if let Some(label) =
        agent_label.filter(|_| parent_workspace == Some(agent.workspace_id.as_str()))
    {
        name_nested_row_by_agent(&mut rows, label);
    }
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
        tree_lines_above: tree_lines.0,
        tree_lines_below: tree_lines.1,
        group_key,
    })
}

/// A child in its parent's workspace would repeat the parent's workspace name, so it takes the
/// agent label as its name instead: the workspace token shows the label and the agent token,
/// now redundant, is dropped (with its line, if nothing else is left on it).
fn name_nested_row_by_agent(rows: &mut Vec<Vec<crate::ui::ResolvedToken>>, label: &str) {
    use crate::ui::ResolvedTokenKind;
    let mut renamed = false;
    for token in rows.iter_mut().flatten() {
        if let ResolvedTokenKind::Workspace(name) = &mut token.kind {
            *name = label.to_string();
            renamed = true;
        }
    }
    if !renamed {
        return;
    }
    for line in rows.iter_mut() {
        line.retain(|token| !matches!(token.kind, ResolvedTokenKind::Agent(_)));
    }
    rows.retain(|line| !line.is_empty());
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
                    &pane_id,
                    None,
                    0,
                    false,
                    0,
                    None,
                    false,
                    (0, 0),
                    None,
                    snapshot,
                    config,
                    machine,
                )
            })
            .collect();
    };

    let tree_rows = super::agent_tree::nest_agents(&ordered, snapshot, collapsed_groups, machine);
    let last_child = super::agent_tree::last_child_flags(&tree_rows);
    let lines_below = super::agent_tree::tree_lines_below(&tree_rows);
    let mut rows = Vec::with_capacity(tree_rows.len());
    // Visible rows come depth-first, so a row's parent is the last row seen one level up.
    let mut ancestors: Vec<&str> = Vec::new();
    for (index, (tree_row, is_last_child)) in tree_rows.iter().zip(last_child).enumerate() {
        ancestors.truncate(tree_row.depth);
        let parent = tree_row
            .depth
            .checked_sub(1)
            .and_then(|level| ancestors.get(level))
            .copied();
        ancestors.push(&tree_row.pane_id);
        let lines_above = index
            .checked_sub(1)
            .map_or(0, |previous| lines_below[previous]);
        let group_key = (tree_row.child_count > 0)
            .then(|| super::agent_tree::agent_group_key(machine, &tree_row.pane_id));
        if let Some(row) = build_agent_row(
            &tree_row.pane_id,
            parent,
            tree_row.depth,
            tree_row.collapsed,
            tree_row.hidden_descendants,
            tree_row.worst_hidden_status,
            is_last_child,
            (lines_above, lines_below[index]),
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

pub(super) fn agent_row(
    snapshot: &ClientShellSnapshot,
    pane_id: &str,
    config: &ClientShellConfig,
    machine: Option<&str>,
) -> Option<AgentRow> {
    build_agent_row(
        pane_id,
        None,
        0,
        false,
        0,
        None,
        false,
        (0, 0),
        None,
        snapshot,
        config,
        machine,
    )
}

/// Padding rows above and below an agent row. A nested row keeps only its bottom padding: the
/// row above already ends in padding, so a parent and its children sit one row apart while
/// separate top-level groups keep the full gap between them.
pub(super) fn agent_row_padding(row: &AgentRow, config: &ClientShellConfig) -> (u16, u16) {
    let padding = config.agents.row_padding;
    (if row.depth > 0 { 0 } else { padding }, padding)
}

/// Renders an agent row into a rect that includes its `agent_row_padding` rows. The padding
/// rows share the row highlight and carry the tree lines that pass through them; the content
/// lines are drawn by `render_agent_row`.
pub(super) fn render_padded_agent_row(
    buffer: &mut Buffer,
    rect: Rect,
    row: &AgentRow,
    config: &ClientShellConfig,
) -> Option<(Rect, String, String)> {
    if row.focused {
        buffer.set_style(rect, Style::default().bg(config.palette.active_row_bg));
    }
    let padding = agent_row_padding(row, config);
    let top = rect.y..rect.y.saturating_add(padding.0).min(rect.bottom());
    let bottom = rect.bottom().saturating_sub(padding.1).max(top.end)..rect.bottom();
    render_tree_lines(buffer, rect, top, row.tree_lines_above, config);
    render_tree_lines(buffer, rect, bottom, row.tree_lines_below, config);
    render_agent_row(
        buffer,
        super::sidebar::padded_content_rect(rect, padding),
        row,
        config,
    )
}

/// Draws `│` in every tree column set in `mask` on rows `ys`, in the connector style
/// `render_agent_row` uses. Used for padding and `row_gap` rows between nested agents.
pub(super) fn render_tree_lines(
    buffer: &mut Buffer,
    rect: Rect,
    ys: std::ops::Range<u16>,
    mask: u8,
    config: &ClientShellConfig,
) {
    if mask == 0 {
        return;
    }
    let style = Style::default().fg(config.palette.overlay0);
    for y in ys {
        for column in 0..super::agent_tree::TREE_LINE_COLUMNS {
            let x = tree_line_x(column + 1);
            if mask & (1 << column) != 0 && x < rect.width {
                put_text(buffer, rect.x + x, y, 1, "│", style);
            }
        }
    }
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
    let status_style = Style::default().fg(status_color(row.status, palette));
    let secondary = Style::default().fg(palette.overlay0);
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
        let starts_with_icon = tokens
            .first()
            .is_some_and(|token| matches!(token.kind, crate::ui::ResolvedTokenKind::StateIcon));
        let prefix = tree_prefix(row, index == 0, starts_with_icon);
        let mut spans = vec![ratatui::text::Span::styled(prefix, prefix_style)];

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
            if collapsed { "▶" } else { "▼" },
            // An open group already shows its tree, so its chevron stays quiet; a collapsed
            // one hides agents and keeps the accent.
            Style::default().fg(if collapsed {
                palette.accent
            } else {
                palette.overlay0
            }),
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

/// Column of the tree line for depth `level` (1-based): directly under the status icon of the
/// depth `level - 1` row it hangs from.
fn tree_line_x(level: usize) -> u16 {
    (level as u16).saturating_mul(2).saturating_sub(1)
}

/// The tree prefix of one content line. A row's icon sits at column `2 * depth + 1` and its
/// text two columns further in; lines hang from the parent's icon, so a child's branch
/// (`├─`/`╰─`) runs straight into its own icon, or into a space when the line has no icon:
///
/// ```text
///  ● herdr
///  │ working · coordinator
///  ├─● reviewer
///  │   working
///  ╰─○ lane-a
///    │ done
///    ╰─● codex
/// ```
fn tree_prefix(row: &AgentRow, first_line: bool, starts_with_icon: bool) -> String {
    let depth = row.depth.min(super::agent_tree::TREE_LINE_COLUMNS);
    let line = |level: usize| row.tree_lines_below & (1 << (level - 1)) != 0;
    let mut prefix = String::from(" ");
    for level in 1..depth {
        prefix.push_str(if line(level) { "│ " } else { "  " });
    }
    if first_line {
        if depth > 0 {
            prefix.push_str(if row.is_last_child {
                "╰─"
            } else {
                "├─"
            });
            if !starts_with_icon {
                prefix.push(' ');
            }
        }
        return prefix;
    }
    if depth > 0 {
        prefix.push_str(if row.is_last_child { "  " } else { "│ " });
    }
    // The row's own line down to its first child starts under its icon.
    let own_child = row.depth < super::agent_tree::TREE_LINE_COLUMNS && line(row.depth + 1);
    prefix.push_str(if own_child { "│ " } else { "  " });
    prefix
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
