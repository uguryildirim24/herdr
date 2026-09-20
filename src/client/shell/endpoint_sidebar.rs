use super::render::{display_width, put_right_text, put_text, ShellRenderState};
use super::*;

fn collapsed_groups_for_endpoint<'a>(
    state: &'a ShellRenderState<'_>,
    endpoint_id: &ClientEndpointId,
) -> Option<&'a HashSet<String>> {
    if endpoint_id.is_local() {
        Some(state.collapsed_groups)
    } else {
        state.remote_collapsed_groups.get(endpoint_id)
    }
}

pub(super) fn render_collapsed(
    buffer: &mut Buffer,
    area: Rect,
    config: &ClientShellConfig,
    state: &mut ShellRenderState<'_>,
    hits: &mut ShellHitMap,
) {
    let palette = &config.palette;
    super::render::render_sidebar_background(buffer, area, palette);
    let (workspace_area, divider_y, detail_area) = super::sidebar::collapsed_sidebar_sections(area);
    let mut total_rows = 0usize;
    let mut selected_row = None;
    let reveal = std::mem::take(state.reveal_navigation_workspace);
    for endpoint in state.endpoints {
        total_rows += 1;
        if state.collapsed_endpoints.contains(&endpoint.endpoint_id) {
            continue;
        }
        if let Some(snapshot) = endpoint.snapshot.as_deref() {
            if reveal {
                if let Some(target) = state
                    .selected_workspace_id
                    .filter(|target| target.endpoint_id == endpoint.endpoint_id)
                {
                    selected_row = snapshot
                        .workspaces
                        .iter()
                        .position(|workspace| workspace.workspace_id == target.workspace_id)
                        .map(|index| total_rows + index);
                }
            }
            total_rows += snapshot.workspaces.len();
        }
    }
    let height = usize::from(workspace_area.height);
    let max_scroll = total_rows.saturating_sub(height);
    *state.workspace_scroll = (*state.workspace_scroll).min(max_scroll);
    if let Some(row) = selected_row {
        if row < *state.workspace_scroll {
            *state.workspace_scroll = row;
        } else if row >= state.workspace_scroll.saturating_add(height) {
            *state.workspace_scroll = row.saturating_add(1).saturating_sub(height).min(max_scroll);
        }
    }
    hits.workspace_max_scroll = max_scroll;
    let mut skip = *state.workspace_scroll;
    let mut y = workspace_area.y;
    for (index, endpoint) in state.endpoints.iter().enumerate() {
        if y >= workspace_area.bottom() {
            break;
        }
        let rect = Rect::new(workspace_area.x, y, workspace_area.width, 1);
        let active = &endpoint.endpoint_id == state.active_endpoint_id;
        let collapsed = state.collapsed_endpoints.contains(&endpoint.endpoint_id);
        if skip > 0 {
            skip -= 1;
        } else {
            if active && collapsed {
                super::render::highlight_sidebar_row(
                    buffer,
                    rect,
                    workspace_area.right(),
                    palette.active_row_bg,
                );
            }
            let label = if endpoint.endpoint_id.is_local() {
                "L".to_owned()
            } else {
                (index + 1).to_string()
            };
            let marker = if collapsed { "▶" } else { "▼" };
            put_text(
                buffer,
                rect.x,
                rect.y,
                rect.width.saturating_sub(1),
                &format!("{marker}{label}"),
                Style::default().fg(if endpoint.status == ClientEndpointStatus::Online {
                    palette.text
                } else {
                    palette.overlay0
                }),
            );
            if !endpoint.endpoint_id.is_local() {
                let (glyph, _, color) = endpoint_status_presentation(endpoint.status, palette);
                put_right_text(buffer, rect, rect.y, glyph, Style::default().fg(color));
            }
            hits.machines.push(MachineHit {
                rect,
                collapse_toggle: Rect::new(rect.x, rect.y, u16::from(rect.width > 1), 1),
                endpoint_id: endpoint.endpoint_id.clone(),
            });
            y = y.saturating_add(1);
        }
        if collapsed {
            continue;
        }
        let Some(snapshot) = endpoint.snapshot.as_deref() else {
            continue;
        };
        for workspace in &snapshot.workspaces {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if y >= workspace_area.bottom() {
                break;
            }
            let rect = Rect::new(workspace_area.x, y, workspace_area.width, 1);
            let focused = active && workspace.focused;
            let selected = state.selected_workspace_id.is_some_and(|target| {
                target.matches(&endpoint.endpoint_id, &workspace.workspace_id)
            });
            let selection_background = if palette.selection_bg == ratatui::style::Color::Reset {
                palette.active_row_bg
            } else {
                palette.selection_bg
            };
            if selected {
                super::render::highlight_sidebar_row(
                    buffer,
                    rect,
                    workspace_area.right(),
                    selection_background,
                );
            } else if focused {
                super::render::highlight_sidebar_row(
                    buffer,
                    rect,
                    workspace_area.right(),
                    palette.active_row_bg,
                );
            }
            let stale = endpoint.status != ClientEndpointStatus::Online;
            let number = format!(" {}", workspace.number);
            let number_width = super::render::display_width(&number).min(rect.width);
            let dim = if stale {
                Modifier::DIM
            } else {
                Modifier::empty()
            };
            put_text(
                buffer,
                rect.x,
                rect.y,
                number_width,
                &number,
                Style::default()
                    .fg(if focused && !stale {
                        palette.text
                    } else {
                        palette.overlay0
                    })
                    .add_modifier(dim),
            );
            put_text(
                buffer,
                rect.x.saturating_add(number_width),
                rect.y,
                rect.width.saturating_sub(number_width),
                status_icon(workspace.agent_status, config.status_indicators),
                Style::default()
                    .fg(if stale {
                        palette.overlay0
                    } else {
                        status_color(workspace.agent_status, palette)
                    })
                    .add_modifier(dim),
            );
            hits.workspaces.push(WorkspaceHit {
                rect,
                endpoint_id: endpoint.endpoint_id.clone(),
                workspace_id: workspace.workspace_id.clone(),
                indented: false,
                group_toggle: None,
            });
            y = y.saturating_add(1);
        }
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
    super::endpoint_agents::render_collapsed(
        buffer,
        detail_area,
        state.endpoints,
        state.active_endpoint_id,
        state.collapsed_groups,
        config,
        hits,
    );
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
        Style::default().fg(palette.overlay0),
    );
}

pub(super) fn render_expanded(
    buffer: &mut Buffer,
    area: Rect,
    active_snapshot: Option<&ClientShellSnapshot>,
    config: &ClientShellConfig,
    state: &mut ShellRenderState<'_>,
    hits: &mut ShellHitMap,
) {
    let palette = &config.palette;
    super::render::render_sidebar_background(buffer, area, palette);
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
        " machines",
        Style::default()
            .fg(palette.overlay0)
            .add_modifier(Modifier::BOLD),
    );
    render_machine_strip(buffer, workspace_area, config, state, hits);

    // One merged list of spaces; the same custom name on several machines is one row.
    let spaces = super::linked_spaces::linked_spaces(
        state.endpoints,
        state.collapsed_endpoints,
        state.collapsed_groups,
        state.remote_collapsed_groups,
    );
    let body = Rect::new(
        workspace_area.x,
        workspace_area.y.saturating_add(WORKSPACE_HEADER_ROWS),
        workspace_area.width,
        workspace_area
            .height
            .saturating_sub(WORKSPACE_HEADER_ROWS + 1),
    );
    hits.workspace_body = body;
    let active = state.active_endpoint_id;
    let paddings = spaces
        .iter()
        .enumerate()
        .map(|(index, space)| {
            let part = &space.parts[space.primary_part_index(active)];
            let next_indented = spaces
                .get(index + 1)
                .is_some_and(|next| next.parts[next.primary_part_index(active)].entry.indented);
            super::sidebar::space_entry_padding(
                part.entry.indented,
                next_indented,
                config.spaces.row_padding,
            )
        })
        .collect::<Vec<_>>();
    let row_heights = spaces
        .iter()
        .zip(&paddings)
        .map(|(space, (top, bottom))| {
            let part = &space.parts[space.primary_part_index(active)];
            (super::sidebar::workspace_rows(
                part.workspace,
                space.worst_status(),
                part.entry.indented,
                &config.spaces,
            )
            .len()
            .max(1)
            .min(u16::MAX as usize) as u16)
                .saturating_add(*top)
                .saturating_add(*bottom)
        })
        .collect::<Vec<_>>();
    let gaps = spaces
        .iter()
        .enumerate()
        .map(|(index, _)| {
            spaces.get(index + 1).map_or(0, |next| {
                u16::from(!next.parts[next.primary_part_index(active)].entry.indented)
                    * config.spaces.row_gap
            })
        })
        .collect::<Vec<_>>();
    let reveal_navigation = !body.is_empty() && std::mem::take(state.reveal_navigation_workspace);
    let reveal_focus = !body.is_empty() && std::mem::take(state.reveal_focused_workspace);
    if reveal_navigation || reveal_focus {
        let selected_row = spaces.iter().position(|space| {
            if reveal_navigation {
                state.selected_workspace_id.is_some_and(|target| {
                    space
                        .parts
                        .iter()
                        .any(|part| target.matches(part.endpoint_id, &part.workspace.workspace_id))
                })
            } else {
                space.parts.iter().any(|part| {
                    part.endpoint_id == state.active_endpoint_id
                        && active_snapshot.is_some_and(|snapshot| {
                            snapshot.focused_workspace_id.as_deref()
                                == Some(part.workspace.workspace_id.as_str())
                        })
                })
            }
        });
        if let Some(selected_row) = selected_row {
            *state.workspace_scroll = super::scroll::list_scroll_start_to_reveal(
                &row_heights,
                &gaps,
                body.height,
                *state.workspace_scroll,
                selected_row,
            );
        }
    }
    let metrics = super::scroll::list_scroll_metrics(
        &row_heights,
        &gaps,
        body.height,
        *state.workspace_scroll,
    );
    hits.workspace_max_scroll = metrics.max_offset_from_bottom;
    hits.workspace_scroll_metrics = Some(metrics);
    *state.workspace_scroll = metrics
        .max_offset_from_bottom
        .saturating_sub(metrics.offset_from_bottom);
    let show_scrollbar = metrics.max_offset_from_bottom > 0 && body.width > 1;
    let content_width = body.width.saturating_sub(u16::from(show_scrollbar));
    let empty_groups = HashSet::new();
    let mut y = body.y;
    for (row_index, space) in spaces.iter().enumerate().skip(*state.workspace_scroll) {
        let part = &space.parts[space.primary_part_index(active)];
        let status = space.worst_status();
        let mut rows = super::sidebar::workspace_rows(
            part.workspace,
            status,
            part.entry.indented,
            &config.spaces,
        );
        let marks = space.machine_marks();
        if !marks.is_empty() {
            if let Some(first) = rows.first_mut() {
                first.push(crate::ui::ResolvedToken {
                    kind: crate::ui::ResolvedTokenKind::Machine(marks),
                    style: crate::config::SidebarTokenStyle::default(),
                });
            }
        }
        let height = row_heights[row_index].min(body.height);
        if y.saturating_add(height) > body.bottom() {
            break;
        }
        let rect = Rect::new(body.x, y, content_width, height);
        let endpoint_active = part.endpoint_id == state.active_endpoint_id;
        let selected = state.selected_workspace_id.is_some_and(|target| {
            space.parts.iter().any(|candidate| {
                target.matches(candidate.endpoint_id, &candidate.workspace.workspace_id)
            })
        });
        if endpoint_active && part.workspace.focused {
            // Padding rows carry the highlight; content rows are repainted below.
            super::render::highlight_sidebar_row(buffer, rect, body.right(), palette.active_row_bg);
        }
        super::sidebar::render_workspace_rows(
            buffer,
            super::sidebar::padded_content_rect(rect, paddings[row_index]),
            part.workspace,
            status,
            config.status_indicators,
            &part.entry,
            rows,
            endpoint_active,
            selected,
            false,
            palette,
        );
        if selected && palette.selection_bg == ratatui::style::Color::Reset {
            buffer.set_style(rect, Style::default().bg(palette.active_row_bg));
        }
        if !part.online {
            buffer.set_style(
                rect,
                Style::default()
                    .fg(palette.overlay0)
                    .add_modifier(Modifier::DIM),
            );
        }
        let collapsed_groups =
            collapsed_groups_for_endpoint(state, part.endpoint_id).unwrap_or(&empty_groups);
        let group_toggle = super::sidebar::render_parent_group_toggle(
            buffer,
            rect,
            part.snapshot,
            part.workspace_index,
            collapsed_groups,
            palette,
        );
        hits.workspaces.push(WorkspaceHit {
            rect,
            endpoint_id: part.endpoint_id.clone(),
            workspace_id: part.workspace.workspace_id.clone(),
            indented: part.entry.indented,
            group_toggle,
        });
        y = y
            .saturating_add(height)
            .saturating_add(gaps.get(row_index).copied().unwrap_or(0));
    }
    if show_scrollbar {
        let track = Rect::new(body.right().saturating_sub(1), body.y, 1, body.height);
        hits.workspace_scrollbar = track;
        super::scroll::render_list_scrollbar(buffer, track, metrics, palette);
    }

    let footer_y = workspace_area.bottom().saturating_sub(1);
    if config.mouse_capture {
        let label = format!(" new · {}", active_endpoint_label(state));
        hits.new_workspace = Rect::new(
            workspace_area.x,
            footer_y,
            display_width(&label).min(workspace_area.width),
            u16::from(workspace_area.height > 0),
        );
        put_text(
            buffer,
            workspace_area.x,
            footer_y,
            workspace_area.width,
            &label,
            Style::default().fg(palette.overlay0),
        );
        let attention = active_snapshot.is_some_and(super::global_menu::global_menu_attention);
        let width = if attention { 8 } else { 6 }.min(workspace_area.width);
        hits.global_launcher = Rect::new(
            workspace_area.right().saturating_sub(width),
            footer_y,
            width,
            1,
        );
        put_right_text(
            buffer,
            workspace_area,
            footer_y,
            if attention { "● menu" } else { "menu" },
            Style::default().fg(if attention {
                palette.accent
            } else {
                palette.overlay0
            }),
        );
    }
    super::endpoint_agents::render_expanded(
        buffer,
        detail_area,
        active_snapshot.and_then(|snapshot| snapshot.agent_view_label.as_deref()),
        state.endpoints,
        state.active_endpoint_id,
        state.collapsed_groups,
        config,
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

/// The one-line machine strip above the merged list: `Local ● · oci ●`. Each
/// name toggles that machine's parts in the list; a hidden machine is dim.
fn render_machine_strip(
    buffer: &mut Buffer,
    workspace_area: Rect,
    config: &ClientShellConfig,
    state: &ShellRenderState<'_>,
    hits: &mut ShellHitMap,
) {
    if workspace_area.height < 2 {
        return;
    }
    let palette = &config.palette;
    let y = workspace_area.y.saturating_add(1);
    let mut x = workspace_area.x.saturating_add(1);
    for (index, endpoint) in state.endpoints.iter().enumerate() {
        let (glyph, _, color) = endpoint_status_presentation(endpoint.status, palette);
        let collapsed = state.collapsed_endpoints.contains(&endpoint.endpoint_id);
        let active = &endpoint.endpoint_id == state.active_endpoint_id;
        let label = format!("{} {glyph}", endpoint.label);
        let label_width = display_width(&label);
        if label_width == 0 || x >= workspace_area.right() {
            break;
        }
        if index > 0 {
            if workspace_area.right().saturating_sub(x) <= 3 {
                break;
            }
            put_text(
                buffer,
                x,
                y,
                3,
                " · ",
                Style::default().fg(palette.overlay0),
            );
            x = x.saturating_add(3);
        }
        let width = label_width.min(workspace_area.right().saturating_sub(x));
        if width == 0 {
            break;
        }
        let rect = Rect::new(x, y, width, 1);
        if active && collapsed {
            super::render::highlight_sidebar_row(
                buffer,
                rect,
                workspace_area.right(),
                palette.active_row_bg,
            );
        }
        let mut style = Style::default().fg(color);
        if collapsed || endpoint.status == ClientEndpointStatus::Disabled {
            style = style.add_modifier(Modifier::DIM);
        } else if active {
            style = style.add_modifier(Modifier::BOLD);
        }
        put_text(buffer, x, y, width, &label, style);
        hits.machines.push(MachineHit {
            rect,
            collapse_toggle: rect,
            endpoint_id: endpoint.endpoint_id.clone(),
        });
        x = x.saturating_add(width);
    }
}

fn active_endpoint_label<'a>(state: &'a ShellRenderState<'_>) -> &'a str {
    state
        .endpoints
        .iter()
        .find(|endpoint| &endpoint.endpoint_id == state.active_endpoint_id)
        .map_or("Local", |endpoint| endpoint.label.as_str())
}
