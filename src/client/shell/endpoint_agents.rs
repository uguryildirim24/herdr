use std::collections::{HashMap, HashSet};

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
/// nesting on, each endpoint's agents are nested with that machine's label as the collapse
/// key, the way the single-endpoint panel nests, and the machines are laid out in order.
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

    let mut ordered_by_endpoint = HashMap::<usize, Vec<String>>::new();
    for row in &aggregate {
        ordered_by_endpoint
            .entry(row.endpoint.endpoint_index)
            .or_default()
            .push(row.agent.pane_id.clone());
    }

    let mut rows = Vec::new();
    for (endpoint_index, endpoint) in endpoints.iter().enumerate() {
        let Some(snapshot) = endpoint.snapshot.as_deref() else {
            continue;
        };
        let Some(ordered) = ordered_by_endpoint.remove(&endpoint_index) else {
            continue;
        };
        let stale = endpoint.status != ClientEndpointStatus::Online;
        for mut agent in super::agent_sidebar::nested_agent_rows(
            snapshot,
            &ordered,
            config,
            Some(collapsed_groups),
            Some(&endpoint.label),
        ) {
            agent.focused &= &endpoint.endpoint_id == active_endpoint_id;
            rows.push(EndpointAgentRow {
                endpoint_id: endpoint.endpoint_id.clone(),
                machine_label: endpoint.label.clone(),
                stale,
                agent,
            });
        }
    }
    rows
}
