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
    /// Hidden descendants per status while this row is collapsed; drawn as its dot stack.
    pub(super) hidden_status_counts: super::agent_tree::StatusCounts,
    pub(super) is_last_child: bool,
    /// Tree lines passing between this row and the next (see
    /// `agent_tree::tree_lines_below`). Zero outside nesting.
    pub(super) tree_lines_below: u8,
    /// Status of the row each line in `tree_lines_below` leads to, per column; a line is
    /// drawn in that row's status color. `None` where no line runs.
    pub(super) tree_line_status_below:
        [Option<crate::api::schema::AgentStatus>; super::agent_tree::TREE_LINE_COLUMNS],
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

/// An agent row being dragged onto a new parent, for drawing the drag. The endpoint scopes the
/// drag to one machine's rows; the aggregate panel has the same pane id on several machines.
#[derive(Clone, Copy)]
pub(super) struct AgentDragView<'a> {
    pub(super) endpoint_id: &'a ClientEndpointId,
    pub(super) pane_id: &'a str,
    pub(super) target: Option<&'a AgentDropTarget>,
}

#[allow(clippy::too_many_arguments)] // One more than the shared panel inputs: the live drag.
pub(super) fn render_agent_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed_groups: &HashSet<String>,
    agent_scroll: &mut usize,
    drag: Option<AgentDragView<'_>>,
    hits: &mut ShellHitMap,
) {
    // While dragging, the header's sort label says what a release would do.
    let drop_hint = drag.map(|drag| match drag.target {
        Some(AgentDropTarget::Parent(_)) => "nest",
        Some(AgentDropTarget::Unnest) => "un-nest",
        None => "drag to a parent",
    });
    if !render_agent_panel_header(
        buffer,
        area,
        drop_hint.or(snapshot.agent_view_label.as_deref()),
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
            let toggles = render_padded_agent_row(buffer, rect, row, config);
            hits.agent_group_toggles.extend(toggles);
            if let Some(drag) = drag {
                render_agent_drag_marks(buffer, rect, row, drag, config);
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
                &row.tree_line_status_below,
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

    // `ui.sidebar.agents.row_padding` pads every agent row below its content (see
    // `agent_row_padding`); `render_row` receives the padded rect.
    let row_heights = rows
        .iter()
        .map(|row| {
            (agent_row(row).rows.len().max(1).min(u16::MAX as usize) as u16)
                .saturating_add(agent_row_padding(config))
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
        highlight_focused_agent_row(buffer, rect, body, agent_row(row), config);
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
    hidden_status_counts: super::agent_tree::StatusCounts,
    is_last_child: bool,
    tree_lines_below: u8,
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
        hidden_status_counts,
        is_last_child,
        tree_lines_below,
        tree_line_status_below: Default::default(),
        group_key,
    })
}

/// A child in its parent's workspace would repeat the parent's workspace name, so it takes the
/// agent label as its name instead: the workspace token shows the label and the agent token,
/// now redundant, is dropped (with its line, if nothing else is left on it), as is a tab label
/// that only repeats the agent label.
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
        line.retain(|token| match &token.kind {
            ResolvedTokenKind::Agent(_) => false,
            ResolvedTokenKind::Tab(tab) => tab != label,
            _ => true,
        });
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
    nested_agent_rows(snapshot, &ordered, config, collapsed_groups, machine)
}

/// Nested rows for one endpoint's agents. `ordered` is that endpoint's pane ids in its own
/// configured order; nesting moves each child directly behind its parent. `collapsed_groups`
/// is `None` (or the nesting flag off) for a flat list, and `machine` qualifies collapse keys
/// and the `$machine` token. The single-endpoint and aggregate agent panels share this.
pub(super) fn nested_agent_rows(
    snapshot: &ClientShellSnapshot,
    ordered: &[String],
    config: &ClientShellConfig,
    collapsed_groups: Option<&HashSet<String>>,
    machine: Option<&str>,
) -> Vec<AgentRow> {
    let Some(collapsed_groups) = collapsed_groups.filter(|_| config.agent_parent_nesting) else {
        return ordered
            .iter()
            .filter_map(|pane_id| {
                build_agent_row(
                    pane_id,
                    None,
                    0,
                    false,
                    Default::default(),
                    false,
                    0,
                    None,
                    snapshot,
                    config,
                    machine,
                )
            })
            .collect();
    };

    let tree_rows = super::agent_tree::nest_agents(ordered, snapshot, collapsed_groups, machine);
    let last_child = super::agent_tree::last_child_flags(&tree_rows);
    let line_targets = super::agent_tree::tree_line_targets_below(&tree_rows);
    let lines_below = line_targets
        .iter()
        .map(super::agent_tree::tree_line_mask)
        .collect::<Vec<_>>();
    let status_by_pane_id = snapshot
        .agents
        .iter()
        .map(|agent| (agent.pane_id.as_str(), agent.agent_status))
        .collect::<HashMap<_, _>>();
    let statuses = tree_rows
        .iter()
        .map(|row| status_by_pane_id.get(row.pane_id.as_str()).copied())
        .collect::<Vec<_>>();
    let line_statuses =
        |index: usize| line_targets[index].map(|target| target.and_then(|target| statuses[target]));
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
        let group_key = (tree_row.child_count > 0)
            .then(|| super::agent_tree::agent_group_key(machine, &tree_row.pane_id));
        if let Some(mut row) = build_agent_row(
            &tree_row.pane_id,
            parent,
            tree_row.depth,
            tree_row.collapsed,
            tree_row.hidden_status_counts,
            is_last_child,
            lines_below[index],
            group_key,
            snapshot,
            config,
            machine,
        ) {
            row.tree_line_status_below = line_statuses(index);
            rows.push(row);
        }
    }
    rows
}

/// Dims the dragged row and marks the parent it would nest under with an accent bar in the
/// free first column. Callers scope this to the dragged endpoint's rows.
pub(super) fn render_agent_drag_marks(
    buffer: &mut Buffer,
    rect: Rect,
    row: &AgentRow,
    drag: AgentDragView<'_>,
    config: &ClientShellConfig,
) {
    if row.pane_id == drag.pane_id {
        buffer.set_style(
            rect,
            Style::default()
                .bg(config.palette.surface1)
                .add_modifier(Modifier::DIM),
        );
    } else if matches!(drag.target, Some(AgentDropTarget::Parent(parent)) if *parent == row.pane_id)
    {
        let accent = Style::default().fg(config.palette.accent);
        for y in rect.y..rect.bottom() {
            put_text(buffer, rect.x, y, 1, "▌", accent);
        }
    }
}

/// Padding rows below an agent row. The highlight leaves padding unlit, so padding above a
/// row as well would double the blank space between neighbours; every agent sits
/// `row_padding` rows below the one before it.
pub(super) fn agent_row_padding(config: &ClientShellConfig) -> u16 {
    config.agents.row_padding
}

/// Highlights the focused agent row. Its content lines are lit in full, and when blank rows
/// separate it from its neighbours the highlight reaches half a row into the blank row above
/// and below, so it ends on the border between agents with even room around the text. It runs
/// across the scrollbar and the sidebar separator right of `body` up to the pane, so the
/// focused agent reads as attached to the terminal it drives.
fn highlight_focused_agent_row(
    buffer: &mut Buffer,
    rect: Rect,
    body: Rect,
    row: &AgentRow,
    config: &ClientShellConfig,
) {
    if !row.focused {
        return;
    }
    let padding = agent_row_padding(config);
    let content = super::sidebar::padded_content_rect(rect, (0, padding));
    let background = config.palette.active_row_bg;
    super::render::highlight_sidebar_row(buffer, content, body.right(), background);
    if padding.saturating_add(config.agents.row_gap) == 0 || content.is_empty() {
        return;
    }
    let half = Style::default()
        .fg(background)
        .remove_modifier(Modifier::all());
    // The blank row above is the previous row's padding or gap, or the blank row under the
    // panel header; its tree lines are already drawn there and keep their cells.
    if let Some(above) = content.y.checked_sub(1) {
        for x in body.x..=body.right() {
            if let Some(cell) = buffer.cell_mut((x, above)) {
                if matches!(cell.symbol(), " " | "│") {
                    cell.set_symbol("▄").set_style(half);
                }
            }
        }
    }
    // The blank row below is this row's own padding (or the gap after it); tree lines drawn
    // into it afterwards replace the half block in their cells.
    if content.bottom() < body.bottom() {
        for x in body.x..=body.right() {
            if let Some(cell) = buffer.cell_mut((x, content.bottom())) {
                cell.set_symbol("▀").set_style(half);
            }
        }
    }
}

/// Renders an agent row into a rect that includes its `agent_row_padding` rows. The padding
/// rows carry the tree lines that pass through them; the content lines are drawn by
/// `render_agent_row`.
pub(super) fn render_padded_agent_row(
    buffer: &mut Buffer,
    rect: Rect,
    row: &AgentRow,
    config: &ClientShellConfig,
) -> Vec<(Rect, String, String)> {
    let padding = (0, agent_row_padding(config));
    let bottom = rect.bottom().saturating_sub(padding.1).max(rect.y)..rect.bottom();
    render_tree_lines(
        buffer,
        rect,
        bottom,
        row.tree_lines_below,
        &row.tree_line_status_below,
        config,
    );
    render_agent_row(
        buffer,
        super::sidebar::padded_content_rect(rect, padding),
        row,
        config,
    )
}

/// Draws the tree rail in every tree column set in `mask` on rows `ys`, each in the status
/// color of the row its line leads to, the way `render_agent_row` draws connectors. Used for
/// padding and `row_gap` rows between nested agents.
pub(super) fn render_tree_lines(
    buffer: &mut Buffer,
    rect: Rect,
    ys: std::ops::Range<u16>,
    mask: u8,
    statuses: &[Option<crate::api::schema::AgentStatus>; super::agent_tree::TREE_LINE_COLUMNS],
    config: &ClientShellConfig,
) {
    if mask == 0 {
        return;
    }
    for y in ys {
        for (column, status) in statuses.iter().enumerate() {
            let x = tree_line_x(column + 1);
            if mask & (1 << column) != 0 && x < rect.width {
                let style = tree_line_style(*status, &config.palette);
                put_text(buffer, rect.x + x, y, 1, TREE_RAIL, style);
            }
        }
    }
}

pub(super) fn render_agent_row(
    buffer: &mut Buffer,
    rect: Rect,
    row: &AgentRow,
    config: &ClientShellConfig,
) -> Vec<(Rect, String, String)> {
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

    // A collapsed parent stands in for its hidden agents with a stack of their status dots,
    // most urgent first, right-aligned one column in from the edge.
    let stack = if row.group_key.is_some() && row.collapsed {
        super::agent_tree::status_stack(&row.hidden_status_counts, MAX_STACK_DOTS).collect()
    } else {
        Vec::new()
    };
    let stack_width = stack.len() as u16;
    let stack_reserved = if stack.is_empty() {
        0
    } else {
        stack_width.saturating_add(2)
    };
    let mut icon_x = None;

    for (index, tokens) in rows.iter().take(rect.height as usize).enumerate() {
        let starts_with_icon = tokens
            .first()
            .is_some_and(|token| matches!(token.kind, crate::ui::ResolvedTokenKind::StateIcon));
        let mut spans = tree_prefix(row, index == 0, starts_with_icon, palette);

        let prefix_width = spans
            .iter()
            .map(|s| display_width(&s.content))
            .sum::<usize>();
        if index == 0 && starts_with_icon {
            icon_x = Some(
                rect.x
                    .saturating_add(prefix_width.min(u16::MAX as usize) as u16),
            );
        }
        let right_reserved = if index == 0 {
            stack_reserved as usize
        } else {
            0
        };
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

    let Some(key) = row.group_key.as_ref() else {
        return Vec::new();
    };
    let mut toggles = Vec::new();
    // The parent's own status mark folds and unfolds its group.
    if let Some(x) = icon_x.filter(|x| *x < rect.right() && rect.height > 0) {
        toggles.push((Rect::new(x, rect.y, 1, 1), row.pane_id.clone(), key.clone()));
    }
    // Clicking the dot stack unfolds the group too.
    if !stack.is_empty() && rect.width > stack_reserved && rect.height > 0 {
        let x = rect.right().saturating_sub(stack_width.saturating_add(1));
        for (offset, status) in stack.iter().enumerate() {
            put_text(
                buffer,
                x.saturating_add(offset as u16),
                rect.y,
                1,
                status_icon(*status, config.status_indicators),
                Style::default().fg(status_color(*status, palette)),
            );
        }
        toggles.push((
            Rect::new(x, rect.y, stack_width, 1),
            row.pane_id.clone(),
            key.clone(),
        ));
    }
    // A parent with no room for either target still reports itself, so the row menu can
    // offer collapse and expand.
    if toggles.is_empty() {
        toggles.push((
            Rect::new(rect.x, rect.y, 0, 0),
            row.pane_id.clone(),
            key.clone(),
        ));
    }
    toggles
}

/// A collapsed parent's dot stack shows at most this many of its hidden agents.
const MAX_STACK_DOTS: usize = 5;

/// Tree glyphs: a heavy rail, and a heavy branch with a two-column arm into the child's icon.
const TREE_RAIL: &str = "┃";
const TREE_BRANCH: &str = "┣━━";
const TREE_LAST_BRANCH: &str = "┗━━";

/// Column of the tree line for depth `level` (1-based): directly under the status icon of the
/// depth `level - 1` row it hangs from.
fn tree_line_x(level: usize) -> u16 {
    (level as u16).saturating_mul(3).saturating_sub(2)
}

/// A tree line in the status color of the row it leads to, dimmed so it stays behind the
/// names; lines to no known row fall back to the muted text color.
fn tree_line_style(status: Option<crate::api::schema::AgentStatus>, palette: &Palette) -> Style {
    Style::default()
        .fg(status.map_or(palette.overlay0, |status| status_color(status, palette)))
        .add_modifier(Modifier::DIM)
}

/// The tree prefix of one content line. A row's icon sits at column `3 * depth + 1` and its
/// text two columns further in; lines hang from the parent's icon, so a child's branch
/// (`┣━━`/`┗━━`) runs straight into its own icon, or into a space when the line has no icon.
/// Each line segment takes the status color of the row it leads to: a branch its own row's,
/// a rail the next row it reaches:
///
/// ```text
///  ● herdr
///  ┃ working · coordinator
///  ┣━━● reviewer
///  ┃   working
///  ┗━━○ lane-a
///     ┃ done
///     ┗━━● codex
/// ```
fn tree_prefix(
    row: &AgentRow,
    first_line: bool,
    starts_with_icon: bool,
    palette: &Palette,
) -> Vec<ratatui::text::Span<'static>> {
    use ratatui::text::Span;
    let depth = row.depth.min(super::agent_tree::TREE_LINE_COLUMNS);
    let line = |level: usize| row.tree_lines_below & (1 << (level - 1)) != 0;
    let rail = |level: usize, trailing: &'static str| {
        Span::styled(
            format!("{TREE_RAIL}{trailing}"),
            tree_line_style(row.tree_line_status_below[level - 1], palette),
        )
    };
    let mut spans = vec![Span::raw(" ")];
    for level in 1..depth {
        spans.push(if line(level) {
            rail(level, "  ")
        } else {
            Span::raw("   ")
        });
    }
    if first_line {
        if depth > 0 {
            spans.push(Span::styled(
                if row.is_last_child {
                    TREE_LAST_BRANCH
                } else {
                    TREE_BRANCH
                },
                tree_line_style(Some(row.status), palette),
            ));
            if !starts_with_icon {
                spans.push(Span::raw(" "));
            }
        }
        return spans;
    }
    if depth > 0 {
        spans.push(if row.is_last_child {
            Span::raw("   ")
        } else {
            rail(depth, "  ")
        });
    }
    // The row's own line down to its first child starts under its icon.
    let own_child = row.depth < super::agent_tree::TREE_LINE_COLUMNS && line(row.depth + 1);
    spans.push(if own_child {
        rail(row.depth + 1, " ")
    } else {
        Span::raw("  ")
    });
    spans
}

fn put_text(buffer: &mut Buffer, x: u16, y: u16, width: u16, text: &str, style: Style) {
    for (offset, character) in text.chars().take(width as usize).enumerate() {
        if let Some(cell) = buffer.cell_mut((x + offset as u16, y)) {
            cell.set_char(character).set_style(style);
        }
    }
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
