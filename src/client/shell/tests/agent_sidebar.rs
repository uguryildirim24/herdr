use std::collections::HashSet;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::api::schema::AgentStatus;
use crate::client::shell::agent_sidebar::{
    agent_rows, render_agent_panel, render_agent_row, visible_agent_pane_ids, AgentRow,
};
use crate::protocol::ClientShellAgent;

#[test]
fn byte_identical_rendering_flag_off_vs_flag_on_without_tokens() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        ClientShellAgent {
            pane_id: "p1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("worker-1".into()),
            display_agent: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: AgentStatus::Working,
            state_change_seq: 1,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: true,
        },
        ClientShellAgent {
            pane_id: "p2".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("worker-2".into()),
            display_agent: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: AgentStatus::Idle,
            state_change_seq: 2,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: false,
        },
    ];

    let mut config_off = ClientShellConfig::from_config(&Config::default());
    config_off.agent_parent_nesting = false;
    let mut config_on = ClientShellConfig::from_config(&Config::default());
    config_on.agent_parent_nesting = true;
    let collapsed = HashSet::new();

    let area = Rect::new(0, 0, 30, 10);
    let mut buffer_off = Buffer::empty(area);
    let mut hits_off = ShellHitMap::default();
    let mut scroll_off = 0;
    render_agent_panel(
        &mut buffer_off,
        area,
        &snapshot,
        &config_off,
        &collapsed,
        &mut scroll_off,
        &mut hits_off,
    );

    let mut buffer_on = Buffer::empty(area);
    let mut hits_on = ShellHitMap::default();
    let mut scroll_on = 0;
    render_agent_panel(
        &mut buffer_on,
        area,
        &snapshot,
        &config_on,
        &collapsed,
        &mut scroll_on,
        &mut hits_on,
    );

    assert_eq!(buffer_off, buffer_on);
    assert_eq!(hits_off.agents, hits_on.agents);
    assert_eq!(hits_off.agent_group_toggles, hits_on.agent_group_toggles);
}

#[test]
fn parent_with_two_children_expanded_chevron_and_indentation() {
    let config = ClientShellConfig::from_config(&Config::default());
    let area = Rect::new(0, 0, 30, 3);
    let mut buffer = Buffer::empty(area);

    let parent_row = AgentRow {
        pane_id: "parent".into(),
        status: AgentStatus::Working,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::Custom("parent-agent".into()),
            style: Default::default(),
        }]],
        depth: 0,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
        group_key: Some("agent:parent".into()),
    };
    let child1 = AgentRow {
        pane_id: "child1".into(),
        status: AgentStatus::Idle,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::Custom("child-1".into()),
            style: Default::default(),
        }]],
        depth: 1,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
        group_key: None,
    };
    let child2 = AgentRow {
        pane_id: "child2".into(),
        status: AgentStatus::Done,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::Custom("child-2".into()),
            style: Default::default(),
        }]],
        depth: 1,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: true,
        group_key: None,
    };

    let p_toggle = render_agent_row(&mut buffer, Rect::new(0, 0, 30, 1), &parent_row, &config);
    let c1_toggle = render_agent_row(&mut buffer, Rect::new(0, 1, 30, 1), &child1, &config);
    let c2_toggle = render_agent_row(&mut buffer, Rect::new(0, 2, 30, 1), &child2, &config);

    // Parent row should return toggle hit and display expanded chevron
    let (toggle_rect, toggle_pane_id, key) = p_toggle.expect("parent toggle hit");
    assert_eq!(toggle_pane_id, "parent");
    assert_eq!(key, "agent:parent");
    assert_eq!(toggle_rect, Rect::new(29, 0, 1, 1));
    assert_eq!(buffer.cell((29, 0)).unwrap().symbol(), "▾");

    // Children should have no toggle
    assert!(c1_toggle.is_none());
    assert!(c2_toggle.is_none());

    // Check tree treatment: child1 gets ├─ , child2 gets └─
    let row1_text: String = (0..30)
        .map(|x| buffer.cell((x, 1)).unwrap().symbol())
        .collect();
    let row2_text: String = (0..30)
        .map(|x| buffer.cell((x, 2)).unwrap().symbol())
        .collect();
    assert!(row1_text.contains("├─ "));
    assert!(row2_text.contains("└─ "));
}

#[test]
fn collapsed_parent_shows_badge_and_worst_status() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.status_indicators = crate::config::StatusIndicatorStyle::Dots;
    let area = Rect::new(0, 0, 30, 1);
    let mut buffer = Buffer::empty(area);

    let parent_row = AgentRow {
        pane_id: "parent".into(),
        status: AgentStatus::Done,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::Custom("parent-agent".into()),
            style: Default::default(),
        }]],
        depth: 0,
        collapsed: true,
        hidden_descendants: 2,
        worst_hidden_status: Some(AgentStatus::Blocked),
        is_last_child: false,
        group_key: Some("agent:parent".into()),
    };

    let toggle = render_agent_row(&mut buffer, area, &parent_row, &config);
    let (toggle_rect, toggle_pane_id, key) = toggle.expect("collapsed toggle hit");
    assert_eq!(toggle_pane_id, "parent");
    assert_eq!(key, "agent:parent");

    // Badge width is 4 + count len = 5
    assert_eq!(toggle_rect.width, 5);
    let badge_text: String = (toggle_rect.x..toggle_rect.right())
        .map(|x| buffer.cell((x, 0)).unwrap().symbol())
        .collect();
    assert_eq!(badge_text, "▸ 2 ●");

    // Worst status mark should have blocked color (red)
    let icon_cell = buffer.cell((toggle_rect.right() - 1, 0)).unwrap();
    assert_eq!(icon_cell.symbol(), "●");
    assert_eq!(icon_cell.fg, config.palette.red);
}

#[test]
fn depth_four_clamps_to_depth_three_indentation() {
    let config = ClientShellConfig::from_config(&Config::default());
    let area = Rect::new(0, 0, 30, 2);
    let mut buffer = Buffer::empty(area);

    let depth_3_row = AgentRow {
        pane_id: "d3".into(),
        status: AgentStatus::Idle,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::Custom("agent".into()),
            style: Default::default(),
        }]],
        depth: 3,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
        group_key: None,
    };
    let depth_4_row = AgentRow {
        pane_id: "d4".into(),
        status: AgentStatus::Idle,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::Custom("agent".into()),
            style: Default::default(),
        }]],
        depth: 4,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
        group_key: None,
    };

    assert!(render_agent_row(&mut buffer, Rect::new(0, 0, 30, 1), &depth_3_row, &config).is_none());
    assert!(render_agent_row(&mut buffer, Rect::new(0, 1, 30, 1), &depth_4_row, &config).is_none());

    // Indentation prefix should be identical (clamped at depth 3)
    let d3_prefix: String = (0..10)
        .map(|x| buffer.cell((x, 0)).unwrap().symbol())
        .collect();
    let d4_prefix: String = (0..10)
        .map(|x| buffer.cell((x, 1)).unwrap().symbol())
        .collect();
    assert_eq!(d3_prefix, d4_prefix);
    assert!(d3_prefix.starts_with("    ├─ "));
}

#[test]
fn agent_navigation_visible_pane_ids_respects_flag() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        ClientShellAgent {
            pane_id: "p1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("first".into()),
            display_agent: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: AgentStatus::Idle,
            state_change_seq: 1,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: true,
        },
        ClientShellAgent {
            pane_id: "p2".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("second".into()),
            display_agent: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: AgentStatus::Idle,
            state_change_seq: 2,
            state_labels: Vec::new(),
            tokens: vec![("parent".into(), "p1".into())],
            focused: false,
        },
    ];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = false;
    let collapsed = HashSet::new();

    let visible = visible_agent_pane_ids(&snapshot, &config, Some(&collapsed), None);
    assert_eq!(visible, vec!["p1", "p2"]);
}

fn tree_agent(
    pane_id: &str,
    status: AgentStatus,
    seq: u64,
    parent: Option<&str>,
) -> ClientShellAgent {
    ClientShellAgent {
        pane_id: pane_id.into(),
        workspace_id: "ws_1".into(),
        tab_id: "tab_1".into(),
        name: Some(pane_id.into()),
        display_agent: None,
        agent: None,
        title: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: status,
        state_change_seq: seq,
        state_labels: Vec::new(),
        tokens: parent
            .map(|parent| vec![("parent".to_string(), parent.to_string())])
            .unwrap_or_default(),
        focused: false,
    }
}

#[test]
fn collapsed_subtree_is_absent_from_navigation_and_hit_map() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Working, 1, None),
        tree_agent("p2", AgentStatus::Idle, 2, Some("p1")),
        tree_agent("p3", AgentStatus::Idle, 3, Some("p2")),
        tree_agent("p4", AgentStatus::Idle, 4, None),
    ];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;

    let expanded = HashSet::new();
    assert_eq!(
        visible_agent_pane_ids(&snapshot, &config, Some(&expanded), None),
        vec!["p1", "p2", "p3", "p4"]
    );

    let collapsed: HashSet<String> = ["agent:p1".to_string()].into_iter().collect();
    assert_eq!(
        visible_agent_pane_ids(&snapshot, &config, Some(&collapsed), None),
        vec!["p1", "p4"]
    );

    let area = Rect::new(0, 0, 30, 12);
    let mut buffer = Buffer::empty(area);
    let mut hits = ShellHitMap::default();
    let mut scroll = 0;
    render_agent_panel(
        &mut buffer,
        area,
        &snapshot,
        &config,
        &collapsed,
        &mut scroll,
        &mut hits,
    );

    let hit_pane_ids = hits
        .agents
        .iter()
        .map(|(_, pane_id)| pane_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(hit_pane_ids, vec!["p1", "p4"]);
    assert_eq!(
        hits.agent_group_toggles
            .iter()
            .map(|(_, pane_id, key)| (pane_id.as_str(), key.as_str()))
            .collect::<Vec<_>>(),
        vec![("p1", "agent:p1")]
    );
}

#[test]
fn priority_sort_orders_parents_and_keeps_children_behind_them() {
    let mut snapshot = snapshot();
    // Priority sort ranks blocked above working above idle; children follow their parent.
    snapshot.agents = vec![
        tree_agent("idle_root", AgentStatus::Idle, 1, None),
        tree_agent("idle_child", AgentStatus::Blocked, 2, Some("idle_root")),
        tree_agent("working_root", AgentStatus::Working, 3, None),
    ];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    config.agent_panel_sort = crate::config::AgentPanelSortConfig::Priority;
    let collapsed = HashSet::new();

    // Flat priority order puts the blocked child first, then working, then idle.
    let mut flat_config = ClientShellConfig::from_config(&Config::default());
    flat_config.agent_panel_sort = crate::config::AgentPanelSortConfig::Priority;
    assert_eq!(
        visible_agent_pane_ids(&snapshot, &flat_config, Some(&collapsed), None),
        vec!["idle_child", "working_root", "idle_root"]
    );

    // Nesting keeps that top-level order and moves the child directly behind its parent.
    let rows = agent_rows(&snapshot, &config, Some(&collapsed), None);
    assert_eq!(
        rows.iter()
            .map(|row| (row.pane_id.as_str(), row.depth))
            .collect::<Vec<_>>(),
        vec![("working_root", 0), ("idle_root", 0), ("idle_child", 1)]
    );
}

#[test]
fn machine_prefixed_group_key_reaches_the_chevron_hit() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Working, 1, None),
        tree_agent("p2", AgentStatus::Idle, 2, Some("p1")),
    ];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let collapsed: HashSet<String> = ["agent:remote:p1".to_string()].into_iter().collect();

    let rows = agent_rows(&snapshot, &config, Some(&collapsed), Some("remote"));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].group_key.as_deref(), Some("agent:remote:p1"));
    assert!(rows[0].collapsed);

    let area = Rect::new(0, 0, 30, 1);
    let mut buffer = Buffer::empty(area);
    let (_, pane_id, key) =
        render_agent_row(&mut buffer, area, &rows[0], &config).expect("chevron hit");
    assert_eq!(pane_id, "p1");
    assert_eq!(key, "agent:remote:p1");
}

#[test]
fn nested_continuation_rows_keep_the_two_column_text_offset() {
    let config = ClientShellConfig::from_config(&Config::default());
    let area = Rect::new(0, 0, 30, 4);
    let mut buffer = Buffer::empty(area);

    let two_row = |pane_id: &str, is_last_child: bool| AgentRow {
        pane_id: pane_id.into(),
        status: AgentStatus::Idle,
        focused: false,
        rows: vec![
            vec![crate::ui::ResolvedToken {
                kind: crate::ui::ResolvedTokenKind::Custom("first".into()),
                style: Default::default(),
            }],
            vec![crate::ui::ResolvedToken {
                kind: crate::ui::ResolvedTokenKind::Custom("second".into()),
                style: Default::default(),
            }],
        ],
        depth: 1,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child,
        group_key: None,
    };

    render_agent_row(
        &mut buffer,
        Rect::new(0, 0, 30, 2),
        &two_row("a", false),
        &config,
    );
    render_agent_row(
        &mut buffer,
        Rect::new(0, 2, 30, 2),
        &two_row("b", true),
        &config,
    );

    let row = |y: u16| -> String {
        (0..30)
            .map(|x| buffer.cell((x, y)).unwrap().symbol())
            .collect()
    };

    // A non-last child keeps the vertical connector under its branch glyph, and both
    // continuation rows start their text two columns right of the first row.
    assert!(row(0).starts_with("├─ first"));
    assert!(row(1).starts_with("│    second"));
    assert!(row(2).starts_with("└─ first"));
    assert!(row(3).starts_with("     second"));
}

#[test]
fn narrow_sidebar_truncates_nested_rows_without_panicking() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Working, 1, None),
        tree_agent("p2", AgentStatus::Blocked, 2, Some("p1")),
        tree_agent("p3", AgentStatus::Idle, 3, Some("p2")),
        tree_agent("p4", AgentStatus::Idle, 4, Some("p3")),
        tree_agent("p5", AgentStatus::Idle, 5, Some("p4")),
    ];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;

    for width in 1..=14u16 {
        for collapsed in [
            HashSet::new(),
            ["agent:p2".to_string()].into_iter().collect(),
        ] {
            let area = Rect::new(0, 0, width, 10);
            let mut buffer = Buffer::empty(area);
            let mut hits = ShellHitMap::default();
            let mut scroll = 0;
            render_agent_panel(
                &mut buffer,
                area,
                &snapshot,
                &config,
                &collapsed,
                &mut scroll,
                &mut hits,
            );
            for (rect, _, _) in &hits.agent_group_toggles {
                assert!(rect.right() <= area.right(), "toggle escapes width {width}");
            }
        }
    }
}

fn render_compact(
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    collapsed: &HashSet<String>,
    area: Rect,
) -> (Buffer, ShellHitMap) {
    let mut buffer = Buffer::empty(area);
    let mut hits = ShellHitMap::default();
    crate::client::shell::render::render_collapsed_sidebar(
        &mut buffer,
        area,
        snapshot,
        config,
        collapsed,
        None,
        &mut hits,
    );
    (buffer, hits)
}

/// First agent line of the compact sidebar for a 20-row area: the workspace half takes
/// ten rows, then the divider.
const COMPACT_AGENT_Y: u16 = 11;

fn compact_line(buffer: &Buffer, y: u16, width: u16) -> String {
    (0..width).map(|x| buffer[(x, y)].symbol()).collect()
}

fn compact_nesting_snapshot() -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Working, 1, None),
        tree_agent("p2", AgentStatus::Idle, 2, None),
        tree_agent("c1", AgentStatus::Blocked, 3, Some("p1")),
        tree_agent("c2", AgentStatus::Done, 4, Some("p1")),
    ];
    snapshot
}

#[test]
fn compact_sidebar_nests_children_after_their_parent_with_tree_marks() {
    let snapshot = compact_nesting_snapshot();
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let icon = |status| status_icon(status, config.status_indicators);
    let area = Rect::new(0, 0, 4, 20);

    let (buffer, hits) = render_compact(&snapshot, &config, &HashSet::new(), area);

    let y = COMPACT_AGENT_Y;
    assert_eq!(
        compact_line(&buffer, y, 3),
        format!("1 {}", icon(AgentStatus::Working))
    );
    assert_eq!(
        compact_line(&buffer, y + 1, 3),
        format!("├2{}", icon(AgentStatus::Blocked))
    );
    assert_eq!(
        compact_line(&buffer, y + 2, 3),
        format!("└3{}", icon(AgentStatus::Done))
    );
    assert_eq!(
        compact_line(&buffer, y + 3, 3),
        format!("4 {}", icon(AgentStatus::Idle))
    );
    assert_eq!(
        hits.agents
            .iter()
            .map(|(rect, pane_id)| (rect.y, pane_id.as_str()))
            .collect::<Vec<_>>(),
        vec![(y, "p1"), (y + 1, "c1"), (y + 2, "c2"), (y + 3, "p2")]
    );
    assert!(hits.agent_group_toggles.is_empty());
    // The numbers are the indices indexed focus jumps use.
    assert_eq!(
        visible_agent_pane_ids(&snapshot, &config, Some(&HashSet::new()), None),
        vec!["p1", "c1", "c2", "p2"]
    );
}

#[test]
fn compact_sidebar_collapsed_parent_hides_children_behind_a_roll_up_cell() {
    let snapshot = compact_nesting_snapshot();
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let icon = |status| status_icon(status, config.status_indicators);
    let collapsed: HashSet<String> = ["agent:p1".to_string()].into_iter().collect();
    let area = Rect::new(0, 0, 4, 20);

    let (buffer, hits) = render_compact(&snapshot, &config, &collapsed, area);

    let y = COMPACT_AGENT_Y;
    assert_eq!(
        compact_line(&buffer, y, 3),
        format!("1 {}", icon(AgentStatus::Working))
    );
    assert_eq!(
        compact_line(&buffer, y + 1, 3),
        format!("▸2{}", icon(AgentStatus::Blocked))
    );
    assert_eq!(
        buffer[(2, y + 1)].fg,
        status_color(AgentStatus::Blocked, &config.palette)
    );
    assert_eq!(buffer[(0, y + 1)].fg, config.palette.accent);
    // The roll-up line carries no number, so the next agent keeps index 2 like the
    // visible order that indexed focus uses.
    assert_eq!(
        compact_line(&buffer, y + 2, 3),
        format!("2 {}", icon(AgentStatus::Idle))
    );
    assert_eq!(
        hits.agents
            .iter()
            .map(|(_, pane_id)| pane_id.as_str())
            .collect::<Vec<_>>(),
        vec!["p1", "p2"]
    );
    assert_eq!(
        hits.agent_group_toggles,
        vec![(
            Rect::new(0, y + 1, 3, 1),
            "p1".to_string(),
            "agent:p1".to_string()
        )]
    );
}

#[test]
fn compact_sidebar_roll_up_count_stays_one_column_past_nine() {
    let mut snapshot = snapshot();
    snapshot.agents = std::iter::once(tree_agent("p1", AgentStatus::Idle, 0, None))
        .chain(
            (1..=12)
                .map(|seq| tree_agent(&format!("c{seq}"), AgentStatus::Working, seq, Some("p1"))),
        )
        .collect();
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let collapsed: HashSet<String> = ["agent:p1".to_string()].into_iter().collect();

    let (buffer, _) = render_compact(&snapshot, &config, &collapsed, Rect::new(0, 0, 4, 20));

    assert_eq!(
        compact_line(&buffer, COMPACT_AGENT_Y + 1, 3),
        format!(
            "▸+{}",
            status_icon(AgentStatus::Working, config.status_indicators)
        )
    );
}

#[test]
fn compact_sidebar_is_unchanged_with_nesting_off_or_without_parent_tokens() {
    let nested = compact_nesting_snapshot();
    let mut flat = nested.clone();
    for agent in &mut flat.agents {
        agent.tokens.clear();
    }
    let off = ClientShellConfig::from_config(&Config::default());
    let mut on = ClientShellConfig::from_config(&Config::default());
    on.agent_parent_nesting = true;
    let collapsed: HashSet<String> = ["agent:p1".to_string()].into_iter().collect();
    let area = Rect::new(0, 0, 4, 20);

    let (flat_off, flat_off_hits) = render_compact(&flat, &off, &HashSet::new(), area);
    let (nested_off, nested_off_hits) = render_compact(&nested, &off, &collapsed, area);
    let (flat_on, flat_on_hits) = render_compact(&flat, &on, &collapsed, area);

    assert_eq!(flat_off, nested_off);
    assert_eq!(flat_off, flat_on);
    assert_eq!(flat_off_hits.agents, nested_off_hits.agents);
    assert_eq!(flat_off_hits.agents, flat_on_hits.agents);
    assert!(nested_off_hits.agent_group_toggles.is_empty());
    assert!(flat_on_hits.agent_group_toggles.is_empty());
    let icon = |status| status_icon(status, off.status_indicators);
    assert_eq!(
        compact_line(&nested_off, COMPACT_AGENT_Y + 2, 3),
        format!("3 {}", icon(AgentStatus::Blocked))
    );
}

#[test]
fn compact_sidebar_clamps_marks_and_roll_up_to_narrow_widths() {
    let snapshot = compact_nesting_snapshot();
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;

    for collapsed in [
        HashSet::new(),
        ["agent:p1".to_string()].into_iter().collect::<HashSet<_>>(),
    ] {
        for width in 0..=4u16 {
            let area = Rect::new(0, 0, width, 20);
            let (buffer, hits) = render_compact(&snapshot, &config, &collapsed, area);
            let content_width = width.saturating_sub(1);
            for (rect, _) in &hits.agents {
                assert!(rect.right() <= content_width, "agent escapes width {width}");
            }
            for (rect, _, _) in &hits.agent_group_toggles {
                assert!(
                    rect.right() <= content_width,
                    "roll-up escapes width {width}"
                );
                assert!(!rect.is_empty());
            }
            if width == 0 {
                continue;
            }
            // The last column belongs to the sidebar edge, never to agent cells.
            for y in COMPACT_AGENT_Y..COMPACT_AGENT_Y + 4 {
                assert_eq!(
                    buffer[(content_width, y)].symbol(),
                    "│",
                    "cell drawn past width {width}"
                );
            }
        }
    }

    let collapsed: HashSet<String> = ["agent:p1".to_string()].into_iter().collect();
    let (buffer, hits) = render_compact(&snapshot, &config, &collapsed, Rect::new(0, 0, 3, 20));
    assert_eq!(compact_line(&buffer, COMPACT_AGENT_Y + 1, 2), "▸2");
    assert_eq!(hits.agent_group_toggles[0].0.width, 2);
    let (buffer, hits) = render_compact(&snapshot, &config, &collapsed, Rect::new(0, 0, 2, 20));
    assert_eq!(compact_line(&buffer, COMPACT_AGENT_Y + 1, 1), "▸");
    assert_eq!(hits.agent_group_toggles[0].0.width, 1);
    let (_, hits) = render_compact(&snapshot, &config, &collapsed, Rect::new(0, 0, 1, 20));
    assert!(hits.agent_group_toggles.is_empty());
}

#[test]
fn compact_sidebar_marks_deeper_levels_by_their_own_siblings() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Idle, 1, None),
        tree_agent("c1", AgentStatus::Idle, 2, Some("p1")),
        tree_agent("g1", AgentStatus::Idle, 3, Some("c1")),
        tree_agent("c2", AgentStatus::Idle, 4, Some("p1")),
    ];
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;

    let (buffer, _) = render_compact(&snapshot, &config, &HashSet::new(), Rect::new(0, 0, 4, 20));

    let marks = (0..4)
        .map(|line| buffer[(0, COMPACT_AGENT_Y + line)].symbol().to_string())
        .collect::<Vec<_>>();
    assert_eq!(marks, vec!["1", "├", "└", "└"]);
}
