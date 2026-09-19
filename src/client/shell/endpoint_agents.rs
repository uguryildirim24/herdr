use std::collections::HashSet;

use super::render::put_text;
use super::*;

pub(super) fn render_collapsed(
    buffer: &mut Buffer,
    area: Rect,
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    collapsed_groups: &HashSet<String>,
    config: &ClientShellConfig,
    hits: &mut ShellHitMap,
) {
    let rows = agent_rows(endpoints, active_endpoint_id, collapsed_groups, config);
    for (index, row) in rows.into_iter().take(area.height as usize).enumerate() {
        let rect = Rect::new(area.x, area.y + index as u16, area.width, 1);
        if row.agent.focused {
            buffer.set_style(rect, Style::default().bg(config.palette.active_row_bg));
        }
        let initial = row.machine_label.chars().next().unwrap_or('?');
        put_text(
            buffer,
            rect.x,
            rect.y,
            rect.width,
            &format!(
                "{initial}{}",
                status_icon(row.agent.status, config.status_indicators)
            ),
            Style::default()
                .fg(if row.stale {
                    config.palette.overlay0
                } else {
                    status_color(row.agent.status, &config.palette)
                })
                .add_modifier(if row.stale {
                    Modifier::DIM
                } else {
                    Modifier::empty()
                }),
        );
        // A parent's status mark folds its group, exactly like the single-endpoint compact
        // sidebar; the key is machine-qualified for the aggregate. The mark is the second
        // column, after the machine's initial.
        if let Some(key) = row.agent.group_key.filter(|_| rect.width > 1) {
            hits.agent_group_toggles.push((
                Rect::new(rect.x.saturating_add(1), rect.y, 1, 1),
                row.agent.pane_id.clone(),
                key,
            ));
        }
        hits.endpoint_agents
            .push((rect, row.endpoint_id, row.agent.pane_id));
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_expanded(
    buffer: &mut Buffer,
    area: Rect,
    agent_view_label: Option<&str>,
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    collapsed_groups: &HashSet<String>,
    config: &ClientShellConfig,
    agent_scroll: &mut usize,
    drag: Option<super::agent_sidebar::AgentDragView<'_>>,
    hits: &mut ShellHitMap,
) {
    if !super::agent_sidebar::render_agent_panel_header(
        buffer,
        area,
        agent_view_label,
        config,
        hits,
    ) {
        return;
    }
    let rows = agent_rows(endpoints, active_endpoint_id, collapsed_groups, config);
    super::agent_sidebar::render_agent_list(
        buffer,
        area,
        &rows,
        agent_view_label.map(|_| " no matching agents"),
        config,
        agent_scroll,
        hits,
        |row| &row.agent,
        |buffer, rect, row, hits| {
            // A nested parent's status mark and dot stack fold and unfold its group, exactly
            // like the single-endpoint panel; the key is machine-qualified for the aggregate.
            let toggles =
                super::agent_sidebar::render_padded_agent_row(buffer, rect, &row.agent, config);
            hits.agent_group_toggles.extend(toggles);
            // The drag marks come from the single-machine panel's renderer; the endpoint scope
            // keeps the same pane id on another machine from being dimmed or accent-marked.
            if let Some(drag) = drag.filter(|drag| drag.endpoint_id == &row.endpoint_id) {
                super::agent_sidebar::render_agent_drag_marks(
                    buffer, rect, &row.agent, drag, config,
                );
            }
            if row.stale {
                buffer.set_style(
                    rect,
                    Style::default()
                        .fg(config.palette.overlay0)
                        .add_modifier(Modifier::DIM),
                );
            }
            hits.endpoint_agents
                .push((rect, row.endpoint_id.clone(), row.agent.pane_id.clone()));
            // `row_gap` rows below a nested row carry the same lines as its bottom padding.
            // The last row never has lines below it, so no gap is drawn past the list.
            let gap_end = rect
                .bottom()
                .saturating_add(config.agents.row_gap)
                .min(hits.agent_body.bottom());
            super::agent_sidebar::render_tree_lines(
                buffer,
                rect,
                rect.bottom()..gap_end,
                row.agent.tree_lines_below,
                &row.agent.tree_line_status_below,
                config,
            );
        },
    );
}

pub(super) struct EndpointAgentRow {
    pub(super) endpoint_id: ClientEndpointId,
    pub(super) machine_label: String,
    pub(super) stale: bool,
    pub(super) agent: super::agent_sidebar::AgentRow,
}

/// Aggregate agent rows in panel order. With nesting off the aggregate order is final. With
/// nesting on, the endpoints' agents are nested together: a bare `parent` token means the same
/// machine, a `<machine>:<pane>` token names another, and the machines are laid out in order.
/// `collapsed_groups` is the shared agent group set.
pub(super) fn agent_rows(
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    collapsed_groups: &HashSet<String>,
    config: &ClientShellConfig,
) -> Vec<EndpointAgentRow> {
    // `aggregate_agent_rows` owns the aggregate ordering and agent-view filtering.
    let aggregate = super::aggregate_navigation::aggregate_agent_rows(
        endpoints,
        active_endpoint_id,
        config.agent_panel_sort,
    );

    if !config.agent_parent_nesting {
        return aggregate
            .into_iter()
            .filter_map(|row| {
                let mut agent = super::agent_sidebar::nested_agent_rows(
                    row.endpoint.snapshot,
                    std::slice::from_ref(&row.agent.pane_id),
                    config,
                    None,
                    Some(row.endpoint.label),
                )
                .into_iter()
                .next()?;
                agent.focused &= row.endpoint.endpoint_id == active_endpoint_id;
                Some(EndpointAgentRow {
                    endpoint_id: row.endpoint.endpoint_id.clone(),
                    machine_label: row.endpoint.label.to_owned(),
                    stale: row.endpoint.stale(),
                    agent,
                })
            })
            .collect();
    }

    let nested =
        super::aggregate_navigation::nested_aggregate_rows(aggregate, endpoints, collapsed_groups);
    let tree_rows = &nested.trees;
    let last_child = super::agent_tree::last_child_flags(tree_rows);
    let line_targets = super::agent_tree::tree_line_targets_below(tree_rows);
    let lines_below = line_targets
        .iter()
        .map(super::agent_tree::tree_line_mask)
        .collect::<Vec<_>>();
    let statuses = nested
        .rows
        .iter()
        .map(|row| Some(row.agent.agent_status))
        .collect::<Vec<_>>();
    let endpoint_of = nested
        .rows
        .iter()
        .map(|row| row.endpoint.endpoint_index)
        .collect::<Vec<_>>();
    let workspace_of = nested
        .rows
        .iter()
        .map(|row| row.agent.workspace_id.as_str())
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(nested.rows.len());
    // Visible rows come depth-first, so a row's parent is the last row seen one level up.
    let mut ancestors: Vec<usize> = Vec::new();
    for (index, (nested_row, tree_row)) in nested.rows.iter().zip(tree_rows).enumerate() {
        ancestors.truncate(tree_row.depth);
        let parent = tree_row
            .depth
            .checked_sub(1)
            .and_then(|level| ancestors.get(level))
            .copied();
        ancestors.push(index);
        // A parent on another machine is not in this snapshot, so only a same-endpoint parent
        // can rename the child's workspace token.
        let parent_workspace_id = parent
            .filter(|&parent| endpoint_of[parent] == endpoint_of[index])
            .map(|parent| workspace_of[parent]);
        let Some(mut agent) = super::agent_sidebar::build_agent_row(
            &tree_row.pane_id,
            parent_workspace_id,
            tree_row.depth,
            tree_row.collapsed,
            tree_row.hidden_status_counts,
            last_child[index],
            lines_below[index],
            (tree_row.child_count > 0).then(|| tree_row.group_key.clone()),
            nested_row.endpoint.snapshot,
            config,
            Some(nested_row.endpoint.label),
        ) else {
            continue;
        };
        agent.tree_line_status_below =
            line_targets[index].map(|target| target.and_then(|target| statuses[target]));
        agent.focused &= nested_row.endpoint.endpoint_id == active_endpoint_id;
        // A lane drawn in its parent's section but running on another machine gets that
        // machine's letter, the way the narrow rail marks machines.
        if parent.is_some_and(|parent| endpoint_of[parent] != endpoint_of[index]) {
            agent.machine_mark = nested_row.endpoint.label.chars().next();
        }
        rows.push(EndpointAgentRow {
            endpoint_id: nested_row.endpoint.endpoint_id.clone(),
            machine_label: nested_row.endpoint.label.to_owned(),
            stale: nested_row.endpoint.stale(),
            agent,
        });
    }
    rows
}
