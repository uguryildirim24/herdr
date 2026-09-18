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
        None,
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
        None,
        &mut hits_on,
    );

    assert_eq!(buffer_off, buffer_on);
    assert_eq!(hits_off.agents, hits_on.agents);
    assert_eq!(hits_off.agent_group_toggles, hits_on.agent_group_toggles);
}

#[test]
fn open_parent_toggles_on_its_status_mark_and_indents_children() {
    let config = ClientShellConfig::from_config(&Config::default());
    let area = Rect::new(0, 0, 30, 3);
    let mut buffer = Buffer::empty(area);

    let parent_row = AgentRow {
        pane_id: "parent".into(),
        status: AgentStatus::Working,
        focused: false,
        rows: vec![vec![
            crate::ui::ResolvedToken {
                kind: crate::ui::ResolvedTokenKind::StateIcon,
                style: Default::default(),
            },
            crate::ui::ResolvedToken {
                kind: crate::ui::ResolvedTokenKind::Custom("parent-agent".into()),
                style: Default::default(),
            },
        ]],
        depth: 0,
        collapsed: false,
        hidden_status_counts: Default::default(),
        is_last_child: false,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
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
        hidden_status_counts: Default::default(),
        is_last_child: false,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
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
        hidden_status_counts: Default::default(),
        is_last_child: true,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
        group_key: None,
    };

    let p_toggle = render_agent_row(&mut buffer, Rect::new(0, 0, 30, 1), &parent_row, &config);
    let c1_toggle = render_agent_row(&mut buffer, Rect::new(0, 1, 30, 1), &child1, &config);
    let c2_toggle = render_agent_row(&mut buffer, Rect::new(0, 2, 30, 1), &child2, &config);

    // An open parent draws no chevron: its own status mark is the toggle.
    let [(toggle_rect, toggle_pane_id, key)] = p_toggle.as_slice() else {
        panic!("one parent toggle, got {p_toggle:?}");
    };
    assert_eq!(toggle_pane_id, "parent");
    assert_eq!(key, "agent:parent");
    assert_eq!(*toggle_rect, Rect::new(1, 0, 1, 1));
    assert_eq!(buffer.cell((1, 0)).unwrap().symbol(), "●");
    let parent_text: String = (0..30)
        .map(|x| buffer.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(!parent_text.contains('▼') && !parent_text.contains('▶'));

    // Children should have no toggle
    assert!(c1_toggle.is_empty());
    assert!(c2_toggle.is_empty());

    // Check tree treatment: child1 gets ┣━━ , child2 gets ┗━━
    let row1_text: String = (0..30)
        .map(|x| buffer.cell((x, 1)).unwrap().symbol())
        .collect();
    let row2_text: String = (0..30)
        .map(|x| buffer.cell((x, 2)).unwrap().symbol())
        .collect();
    assert!(row1_text.contains("┣━━ "));
    assert!(row2_text.contains("┗━━ "));
}

#[test]
fn collapsed_parent_shows_a_dot_stack_of_hidden_statuses() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.status_indicators = crate::config::StatusIndicatorStyle::Dots;
    let area = Rect::new(0, 0, 30, 1);
    let mut buffer = Buffer::empty(area);

    // Hidden: one idle, two working, one blocked (indexed by `status_severity`).
    let parent_row = AgentRow {
        pane_id: "parent".into(),
        status: AgentStatus::Done,
        focused: false,
        rows: vec![vec![
            crate::ui::ResolvedToken {
                kind: crate::ui::ResolvedTokenKind::StateIcon,
                style: Default::default(),
            },
            crate::ui::ResolvedToken {
                kind: crate::ui::ResolvedTokenKind::Custom("parent-agent".into()),
                style: Default::default(),
            },
        ]],
        depth: 0,
        collapsed: true,
        hidden_status_counts: [0, 1, 0, 2, 1],
        is_last_child: false,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
        group_key: Some("agent:parent".into()),
    };

    let toggles = render_agent_row(&mut buffer, area, &parent_row, &config);
    // Both the parent's own status mark and the dot stack unfold the group.
    let [(icon_rect, icon_pane_id, icon_key), (stack_rect, stack_pane_id, stack_key)] =
        toggles.as_slice()
    else {
        panic!("icon and stack toggles, got {toggles:?}");
    };
    assert_eq!(*icon_rect, Rect::new(1, 0, 1, 1));
    assert_eq!(
        (icon_pane_id.as_str(), icon_key.as_str()),
        ("parent", "agent:parent")
    );
    assert_eq!(
        (stack_pane_id.as_str(), stack_key.as_str()),
        ("parent", "agent:parent")
    );

    // Four dots, most urgent first, right-aligned one column in from the edge.
    assert_eq!(*stack_rect, Rect::new(25, 0, 4, 1));
    let stack: Vec<_> = (stack_rect.x..stack_rect.right())
        .map(|x| {
            let cell = buffer.cell((x, 0)).unwrap();
            (cell.symbol().to_owned(), cell.fg)
        })
        .collect();
    let palette = &config.palette;
    assert_eq!(
        stack,
        vec![
            ("●".to_owned(), palette.red),
            ("●".to_owned(), palette.yellow),
            ("●".to_owned(), palette.yellow),
            ("○".to_owned(), palette.green),
        ]
    );
    assert_eq!(buffer.cell((29, 0)).unwrap().symbol(), " ");
    let text: String = (0..30)
        .map(|x| buffer.cell((x, 0)).unwrap().symbol())
        .collect();
    assert!(!text.contains('▶') && !text.contains('▼'));
}

#[test]
fn collapsed_parent_dot_stack_is_capped_at_five() {
    let config = ClientShellConfig::from_config(&Config::default());
    let area = Rect::new(0, 0, 30, 1);
    let mut buffer = Buffer::empty(area);
    let parent_row = AgentRow {
        pane_id: "parent".into(),
        status: AgentStatus::Idle,
        focused: false,
        rows: vec![vec![crate::ui::ResolvedToken {
            kind: crate::ui::ResolvedTokenKind::StateIcon,
            style: Default::default(),
        }]],
        depth: 0,
        collapsed: true,
        hidden_status_counts: [0, 9, 0, 0, 0],
        is_last_child: false,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
        group_key: Some("agent:parent".into()),
    };

    let toggles = render_agent_row(&mut buffer, area, &parent_row, &config);
    let stack_rect = toggles.last().expect("stack toggle").0;
    assert_eq!(stack_rect, Rect::new(24, 0, 5, 1));
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
        hidden_status_counts: Default::default(),
        is_last_child: false,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
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
        hidden_status_counts: Default::default(),
        is_last_child: false,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
        group_key: None,
    };

    assert!(
        render_agent_row(&mut buffer, Rect::new(0, 0, 30, 1), &depth_3_row, &config).is_empty()
    );
    assert!(
        render_agent_row(&mut buffer, Rect::new(0, 1, 30, 1), &depth_4_row, &config).is_empty()
    );

    // Indentation prefix should be identical (clamped at depth 3)
    let d3_prefix: String = (0..10)
        .map(|x| buffer.cell((x, 0)).unwrap().symbol())
        .collect();
    let d4_prefix: String = (0..10)
        .map(|x| buffer.cell((x, 1)).unwrap().symbol())
        .collect();
    assert_eq!(d3_prefix, d4_prefix);
    assert!(d3_prefix.starts_with("       ┣━━"));
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

#[test]
fn hiding_the_parent_token_does_not_drop_lineage() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Working, 1, None),
        tree_agent("p2", AgentStatus::Idle, 2, Some("p1")),
    ];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    config.agents = toml::from_str(
        r#"
rows = [[{ token = "$parent", rules = [{ contains = "", hide = true }] }, "agent"]]
"#,
    )
    .unwrap();
    let collapsed = HashSet::new();
    let rows = agent_rows(&snapshot, &config, Some(&collapsed), None);

    assert_eq!(
        rows.iter()
            .map(|row| (row.pane_id.as_str(), row.depth))
            .collect::<Vec<_>>(),
        vec![("p1", 0), ("p2", 1)]
    );
    assert!(
        rows.iter().all(|row| {
            !row.rows.iter().flatten().any(|token| {
                matches!(
                    &token.kind,
                    crate::ui::ResolvedTokenKind::Custom(value) if value == "p1"
                )
            })
        }),
        "hide = true must drop the displayed parent token without flattening the tree"
    );
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
        None,
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
        // The collapsed parent's status mark and its dot stack.
        vec![("p1", "agent:p1"), ("p1", "agent:p1")]
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
fn machine_prefixed_group_key_reaches_the_toggle_hit() {
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
    let toggles = render_agent_row(&mut buffer, area, &rows[0], &config);
    let (_, pane_id, key) = toggles.first().expect("toggle hit");
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
        hidden_status_counts: Default::default(),
        is_last_child,
        tree_lines_below: 0,
        tree_line_status_below: Default::default(),
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

    // A non-last child keeps the vertical connector under its branch glyph, which hangs from
    // the parent's icon column, and continuation rows start their text where a child's name
    // starts after its icon.
    assert!(row(0).starts_with(" ┣━━ first"));
    assert!(row(1).starts_with(" ┃    second"));
    assert!(row(2).starts_with(" ┗━━ first"));
    assert!(row(3).starts_with("      second"));
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
                None,
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
    // A parent draws like a flat cell; its status mark is the collapse toggle.
    assert_eq!(
        compact_line(&buffer, y, 3),
        format!("1 {}", icon(AgentStatus::Working))
    );
    assert_eq!(
        compact_line(&buffer, y + 1, 3),
        format!("┣2{}", icon(AgentStatus::Blocked))
    );
    // A child's tree mark takes its own status color, dimmed like the expanded branches.
    assert_eq!(
        buffer[(0, y + 1)].fg,
        status_color(AgentStatus::Blocked, &config.palette)
    );
    assert!(buffer[(0, y + 1)].modifier.contains(Modifier::DIM));
    assert_eq!(
        compact_line(&buffer, y + 2, 3),
        format!("┗3{}", icon(AgentStatus::Done))
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
    assert_eq!(
        hits.agent_group_toggles,
        vec![(
            Rect::new(2, y, 1, 1),
            "p1".to_string(),
            "agent:p1".to_string()
        )]
    );
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
    // The hidden children show as their status dots, most urgent first, right-aligned.
    assert_eq!(
        compact_line(&buffer, y + 1, 3),
        format!(" {}{}", icon(AgentStatus::Blocked), icon(AgentStatus::Done))
    );
    assert_eq!(
        buffer[(1, y + 1)].fg,
        status_color(AgentStatus::Blocked, &config.palette)
    );
    assert_eq!(
        buffer[(2, y + 1)].fg,
        status_color(AgentStatus::Done, &config.palette)
    );
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
    let toggle = |rect| (rect, "p1".to_string(), "agent:p1".to_string());
    assert_eq!(
        hits.agent_group_toggles,
        vec![
            toggle(Rect::new(2, y, 1, 1)),
            toggle(Rect::new(0, y + 1, 3, 1))
        ]
    );
}

#[test]
fn compact_sidebar_roll_up_shows_at_most_three_dots() {
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
        status_icon(AgentStatus::Working, config.status_indicators).repeat(3)
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
    let icon = |status| status_icon(status, config.status_indicators);
    // Too narrow for the parent's status mark, so the roll-up is the only toggle left.
    let (buffer, hits) = render_compact(&snapshot, &config, &collapsed, Rect::new(0, 0, 3, 20));
    assert_eq!(
        compact_line(&buffer, COMPACT_AGENT_Y + 1, 2),
        format!("{}{}", icon(AgentStatus::Blocked), icon(AgentStatus::Done))
    );
    assert_eq!(hits.agent_group_toggles.len(), 1);
    assert_eq!(hits.agent_group_toggles[0].0.width, 2);
    let (buffer, hits) = render_compact(&snapshot, &config, &collapsed, Rect::new(0, 0, 2, 20));
    assert_eq!(
        compact_line(&buffer, COMPACT_AGENT_Y + 1, 1),
        icon(AgentStatus::Blocked)
    );
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
    // c1 has a sibling after it, g1 is the last child of c1, and c2 the last child of p1.
    assert_eq!(marks, vec!["1", "┣", "┗", "┗"]);
}

#[test]
fn compact_sidebar_ignores_row_padding_with_nesting() {
    let snapshot = compact_nesting_snapshot();
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let collapsed: HashSet<String> = ["agent:p1".to_string()].into_iter().collect();
    let area = Rect::new(0, 0, 4, 20);

    for groups in [HashSet::new(), collapsed] {
        let (plain_buffer, plain_hits) = render_compact(&snapshot, &config, &groups, area);
        let mut padded = ClientShellConfig::from_config(&Config::default());
        padded.agent_parent_nesting = true;
        padded.agents.row_padding = 2;
        padded.spaces.row_padding = 2;
        let (padded_buffer, padded_hits) = render_compact(&snapshot, &padded, &groups, area);

        assert_eq!(padded_buffer, plain_buffer);
        assert_eq!(padded_hits.agents, plain_hits.agents);
        assert_eq!(
            padded_hits
                .workspaces
                .iter()
                .map(|hit| hit.rect)
                .collect::<Vec<_>>(),
            plain_hits
                .workspaces
                .iter()
                .map(|hit| hit.rect)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            padded_hits.agent_group_toggles,
            plain_hits.agent_group_toggles
        );
    }
}

fn compact_click(state: &mut ClientShellState, column: u16, row: u16) -> ClientShellInput {
    let event = |kind| {
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::empty(),
        })
    };
    state.handle_raw_events(vec![
        event(MouseEventKind::Down(MouseButton::Left)),
        event(MouseEventKind::Up(MouseButton::Left)),
    ])
}

fn compact_nesting_state() -> ClientShellState {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(compact_nesting_snapshot()));
    state.set_pane_surface(surface());
    state.sidebar_collapsed = true;
    state
}

fn compact_agent_y(state: &ClientShellState, pane_id: &str) -> u16 {
    state
        .hits
        .agents
        .iter()
        .find(|(_, id)| id == pane_id)
        .map(|(rect, _)| rect.y)
        .expect("compact agent cell")
}

#[test]
fn compact_sidebar_collapse_marker_toggles_the_shared_group_state() {
    let mut state = compact_nesting_state();
    state.compose(106, 30).expect("compact frame");
    let parent_y = compact_agent_y(&state, "p1");
    assert_eq!(
        state.hits.agent_group_toggles,
        vec![(
            Rect::new(2, parent_y, 1, 1),
            "p1".to_string(),
            "agent:p1".to_string()
        )]
    );

    // The parent's status mark collapses through the same `collapsed_groups` key the
    // expanded panel uses.
    let outcome = compact_click(&mut state, 2, parent_y);
    assert!(outcome.actions.is_empty(), "the toggle must not focus");
    assert!(state.collapsed_groups.contains("agent:p1"));
    state.compose(106, 30).expect("collapsed compact frame");
    let (roll_up, _, key) = state.hits.agent_group_toggles[1].clone();
    assert_eq!(key, "agent:p1");
    assert_eq!(roll_up.y, parent_y + 1);
    assert_eq!(
        state
            .hits
            .agents
            .iter()
            .map(|(_, pane_id)| pane_id.as_str())
            .collect::<Vec<_>>(),
        vec!["p1", "p2"]
    );
    assert_eq!(
        visible_agent_pane_ids(
            state.snapshot.as_deref().expect("snapshot"),
            &state.config,
            Some(&state.collapsed_groups),
            None
        ),
        vec!["p1", "p2"]
    );

    // The roll-up expands it again, and the expanded panel sees the same state.
    compact_click(&mut state, roll_up.x, roll_up.y);
    assert!(!state.collapsed_groups.contains("agent:p1"));
    state.sidebar_collapsed = false;
    state.compose(106, 30).expect("expanded frame");
    assert!(state.hits.agents.iter().any(|(_, pane_id)| pane_id == "c1"));

    // The rest of the parent cell still focuses the parent.
    state.sidebar_collapsed = true;
    state.compose(106, 30).expect("compact frame again");
    let outcome = compact_click(&mut state, 1, parent_y);
    assert!(matches!(
        &outcome.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                crate::api::schema::Method::PaneFocus(crate::api::schema::PaneTarget { pane_id })
                    if pane_id == "p1"
            )
    ));
    assert!(!state.collapsed_groups.contains("agent:p1"));
}

#[test]
fn focused_agent_highlight_has_no_half_rows_when_agents_touch() {
    let mut snapshot = snapshot();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Idle, 1, None),
        tree_agent("p2", AgentStatus::Idle, 2, None),
        tree_agent("p3", AgentStatus::Idle, 3, None),
    ];
    snapshot.agents[1].focused = true;
    let config = ClientShellConfig::from_config(&Config::default());
    let area = Rect::new(0, 0, 30, 12);
    let mut buffer = Buffer::empty(area);
    let mut hits = ShellHitMap::default();
    render_agent_panel(
        &mut buffer,
        area,
        &snapshot,
        &config,
        &HashSet::new(),
        &mut 0,
        None,
        &mut hits,
    );

    let text: String = (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .map(|(x, y)| buffer[(x, y)].symbol().to_string())
        .collect();
    assert!(!text.contains('▀') && !text.contains('▄'));
}

#[test]
fn compact_sidebar_focused_agent_highlight_reaches_the_pane() {
    let mut snapshot = compact_nesting_snapshot();
    snapshot.agents[1].focused = true;
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;

    let (buffer, _) = render_compact(&snapshot, &config, &HashSet::new(), Rect::new(0, 0, 4, 20));

    let active = config.palette.active_row_bg;
    // p2 sorts last: p1, c1, c2, p2.
    let y = COMPACT_AGENT_Y + 3;
    assert!((0..4).all(|x| buffer[(x, y)].bg == active));
    assert_eq!(buffer[(3, y)].symbol(), " ");
    assert_eq!(buffer[(3, y - 1)].symbol(), "│");
    assert_ne!(buffer[(3, y - 1)].bg, active);
}

#[test]
fn compact_sidebar_numbers_children_past_nine_like_flat_cells() {
    let mut snapshot = snapshot();
    snapshot.agents = std::iter::once(tree_agent("p1", AgentStatus::Idle, 0, None))
        .chain(
            (1..=11)
                .map(|seq| tree_agent(&format!("c{seq}"), AgentStatus::Working, seq, Some("p1"))),
        )
        .collect();
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let icon = status_icon(AgentStatus::Working, config.status_indicators);

    let (buffer, hits) =
        render_compact(&snapshot, &config, &HashSet::new(), Rect::new(0, 0, 4, 40));
    let (flat_buffer, _) = render_compact(
        &snapshot,
        &ClientShellConfig::from_config(&Config::default()),
        &HashSet::new(),
        Rect::new(0, 0, 4, 40),
    );

    let first = hits.agents[0].0.y;
    assert_eq!(compact_line(&buffer, first + 8, 3), format!("┣9{icon}"));
    // Index 10 and up: the two-digit index takes the tree column, exactly like the flat list.
    for index in [10u16, 11, 12] {
        let y = first + index - 1;
        let line = compact_line(&buffer, y, 3);
        assert!(
            line.starts_with(&index.to_string()),
            "index {index}: {line}"
        );
        assert_eq!(line, compact_line(&flat_buffer, y, 3));
    }
    assert_eq!(hits.agents.len(), 12);
}

#[test]
fn child_in_its_parents_workspace_is_named_by_its_agent_label() {
    let mut snapshot = snapshot();
    let mut other_workspace = snapshot.workspaces[0].clone();
    other_workspace.workspace_id = "ws_2".into();
    other_workspace.label = "elsewhere".into();
    snapshot.workspaces.push(other_workspace);
    let mut remote_child = tree_agent("c2", AgentStatus::Idle, 3, Some("p1"));
    remote_child.workspace_id = "ws_2".into();
    snapshot.agents = vec![
        tree_agent("p1", AgentStatus::Working, 1, None),
        tree_agent("c1", AgentStatus::Done, 2, Some("p1")),
        remote_child,
    ];
    let parsed: Config = toml::from_str(
        r#"
[ui.sidebar.agents]
rows = [["state_icon", "workspace"], ["state_text", "agent"]]
"#,
    )
    .expect("agent rows config");
    let mut config = ClientShellConfig::from_config(&parsed);
    config.agent_parent_nesting = true;

    let rows = agent_rows(&snapshot, &config, Some(&HashSet::new()), None);
    let texts = |row: &AgentRow| {
        row.rows
            .iter()
            .map(|line| {
                line.iter()
                    .filter_map(|token| match &token.kind {
                        crate::ui::ResolvedTokenKind::Workspace(text)
                        | crate::ui::ResolvedTokenKind::StateText(text)
                        | crate::ui::ResolvedTokenKind::Agent(text) => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let workspace = snapshot.workspaces[0].label.clone();
    assert_eq!(
        texts(&rows[0]),
        [vec![workspace], vec!["working".into(), "p1".into()]]
    );
    // Same workspace as the parent: the agent label replaces the repeated workspace name.
    assert_eq!(
        texts(&rows[1]),
        [vec!["c1".to_string()], vec!["done".into()]]
    );
    // A child in another workspace does not nest, so it keeps its workspace name.
    assert_eq!(
        texts(&rows[2]),
        [
            vec!["elsewhere".to_string()],
            vec!["idle".into(), "c2".into()]
        ]
    );
}

#[test]
fn nested_child_drops_a_tab_label_that_repeats_its_name() {
    let mut snapshot = snapshot();
    let mut second_tab = snapshot.tabs[0].clone();
    second_tab.tab_id = "tab_2".into();
    second_tab.label = "c1".into();
    second_tab.custom_label = true;
    snapshot.tabs.push(second_tab);
    let mut child = tree_agent("c1", AgentStatus::Done, 2, Some("p1"));
    child.tab_id = "tab_2".into();
    snapshot.agents = vec![tree_agent("p1", AgentStatus::Working, 1, None), child];
    let parsed: Config = toml::from_str(
        r#"
[ui.sidebar.agents]
rows = [["state_icon", "workspace", "tab"]]
"#,
    )
    .expect("agent rows config");
    let mut config = ClientShellConfig::from_config(&parsed);
    config.agent_parent_nesting = true;

    let rows = agent_rows(&snapshot, &config, Some(&HashSet::new()), None);
    let kinds = rows[1].rows[0]
        .iter()
        .map(|token| format!("{:?}", token.kind))
        .collect::<Vec<_>>();
    assert_eq!(kinds, ["StateIcon", "Workspace(\"c1\")"]);
}
