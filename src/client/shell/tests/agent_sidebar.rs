use std::collections::HashSet;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::api::schema::AgentStatus;
use crate::client::shell::agent_sidebar::{
    render_agent_panel, render_agent_row, visible_agent_pane_ids, AgentRow,
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
    let mut config_on = config_off.clone();
    config_on.agent_parent_nesting = true;

    let area = Rect::new(0, 0, 30, 10);
    let mut buffer_off = Buffer::empty(area);
    let mut hits_off = ShellHitMap::default();
    let mut scroll_off = 0;
    render_agent_panel(
        &mut buffer_off,
        area,
        &snapshot,
        &config_off,
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
        child_count: 2,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
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
        child_count: 0,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
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
        child_count: 0,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: true,
    };

    let p_toggle = render_agent_row(&mut buffer, Rect::new(0, 0, 30, 1), &parent_row, &config);
    let c1_toggle = render_agent_row(&mut buffer, Rect::new(0, 1, 30, 1), &child1, &config);
    let c2_toggle = render_agent_row(&mut buffer, Rect::new(0, 2, 30, 1), &child2, &config);

    // Parent row should return toggle hit and display expanded chevron
    let (toggle_rect, key) = p_toggle.expect("parent toggle hit");
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
        child_count: 2,
        collapsed: true,
        hidden_descendants: 2,
        worst_hidden_status: Some(AgentStatus::Blocked),
        is_last_child: false,
    };

    let toggle = render_agent_row(&mut buffer, area, &parent_row, &config);
    let (toggle_rect, key) = toggle.expect("collapsed toggle hit");
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
        child_count: 0,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
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
        child_count: 0,
        collapsed: false,
        hidden_descendants: 0,
        worst_hidden_status: None,
        is_last_child: false,
    };

    render_agent_row(&mut buffer, Rect::new(0, 0, 30, 1), &depth_3_row, &config);
    render_agent_row(&mut buffer, Rect::new(0, 1, 30, 1), &depth_4_row, &config);

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

    let visible = visible_agent_pane_ids(&snapshot, &config, &collapsed, None);
    assert_eq!(visible, vec!["p1", "p2"]);
}
