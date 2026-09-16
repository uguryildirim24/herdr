use ratatui::layout::Rect;

use super::*;
use crate::api::schema::{AgentStatus, Method, PaneTarget};
use crate::protocol::ClientShellAgent;
use crate::raw_input::RawInputEvent;

#[test]
fn mouse_click_on_agent_chevron_toggles_collapsed_groups_and_click_elsewhere_focuses_pane() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let mut state = ClientShellState::new(config);

    // Setup snapshot with an agent
    let mut snap = snapshot();
    snap.agents = vec![ClientShellAgent {
        pane_id: "p1".into(),
        workspace_id: "ws_1".into(),
        tab_id: "tab_1".into(),
        name: Some("orchestrator".into()),
        display_agent: None,
        agent: None,
        title: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: AgentStatus::Working,
        state_change_seq: 1,
        state_labels: Vec::new(),
        tokens: Vec::new(),
        focused: false,
    }];
    state.set_snapshot(Box::new(snap));
    state.set_pane_surface(surface());

    // Populate hits: simulate an agent row at (0, 5, 30, 1) with chevron toggle at (29, 5, 1, 1)
    state
        .hits
        .agents
        .push((Rect::new(0, 5, 30, 1), "p1".into()));
    state
        .hits
        .agent_group_toggles
        .push((Rect::new(29, 5, 1, 1), "p1".into(), "agent:p1".into()));

    // 1. Click on chevron at (29, 5): should toggle collapsed_groups without focusing pane
    let chevron_click = state.handle_raw_events(vec![
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 29,
            row: 5,
            modifiers: KeyModifiers::empty(),
        }),
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 29,
            row: 5,
            modifiers: KeyModifiers::empty(),
        }),
    ]);
    assert!(state.collapsed_groups.contains("agent:p1"));
    assert!(chevron_click.actions.is_empty());

    // 2. Click chevron again at (29, 5): should expand (remove from collapsed_groups)
    let chevron_unclick = state.handle_raw_events(vec![
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 29,
            row: 5,
            modifiers: KeyModifiers::empty(),
        }),
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 29,
            row: 5,
            modifiers: KeyModifiers::empty(),
        }),
    ]);
    assert!(!state.collapsed_groups.contains("agent:p1"));
    assert!(chevron_unclick.actions.is_empty());

    // 3. Click elsewhere on the agent row at (10, 5): should focus pane
    let row_click = state.handle_raw_events(vec![
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        }),
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        }),
    ]);
    assert!(matches!(
        &row_click.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                Method::PaneFocus(PaneTarget { pane_id }) if pane_id == "p1"
            )
    ));
}

fn compact_tree_agent(pane_id: &str, parent: Option<&str>) -> ClientShellAgent {
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
        agent_status: AgentStatus::Working,
        state_change_seq: 1,
        state_labels: Vec::new(),
        tokens: parent
            .map(|parent| vec![("parent".to_string(), parent.to_string())])
            .unwrap_or_default(),
        focused: false,
    }
}

fn click(state: &mut ClientShellState, rect: Rect) -> ClientShellInput {
    state.handle_raw_events(vec![
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: rect.x,
            row: rect.y,
            modifiers: KeyModifiers::empty(),
        }),
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: rect.x,
            row: rect.y,
            modifiers: KeyModifiers::empty(),
        }),
    ])
}

#[test]
fn compact_sidebar_shares_collapse_state_with_the_agents_panel_and_focuses_children() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let mut state = ClientShellState::new(config);
    let mut snap = snapshot();
    snap.agents = vec![
        compact_tree_agent("p1", None),
        compact_tree_agent("c1", Some("p1")),
    ];
    state.set_snapshot(Box::new(snap));
    state.set_pane_surface(surface());
    let agent_ids = |state: &ClientShellState| {
        state
            .hits
            .agents
            .iter()
            .map(|(_, pane_id)| pane_id.clone())
            .collect::<Vec<_>>()
    };

    // Collapse from the expanded Agents panel chevron.
    state.compose(106, 30).expect("expanded sidebar");
    let chevron = state.hits.agent_group_toggles[0].0;
    click(&mut state, chevron);
    assert!(state.collapsed_groups.contains("agent:p1"));

    // The compact sidebar reads the same state: the child is hidden behind a roll-up cell.
    state.sidebar_collapsed = true;
    state.compose(106, 30).expect("compact sidebar");
    assert_eq!(agent_ids(&state), vec!["p1"]);
    let [(roll_up, pane_id, key)] = &state.hits.agent_group_toggles[..] else {
        panic!("compact sidebar should draw one roll-up cell");
    };
    assert_eq!((pane_id.as_str(), key.as_str()), ("p1", "agent:p1"));
    let roll_up = *roll_up;
    assert!(roll_up.right() <= 3);

    // Clicking the roll-up cell expands through the same collapsed_groups key.
    let expand = click(&mut state, roll_up);
    assert!(expand.actions.is_empty());
    assert!(!state.collapsed_groups.contains("agent:p1"));
    state.compose(106, 30).expect("compact sidebar, expanded");
    assert_eq!(agent_ids(&state), vec!["p1", "c1"]);
    // The expanded parent now carries the one-column `▾` collapse toggle instead.
    let [(collapse, pane_id, key)] = &state.hits.agent_group_toggles[..] else {
        panic!("compact sidebar should draw one collapse toggle");
    };
    assert_eq!((pane_id.as_str(), key.as_str()), ("p1", "agent:p1"));
    assert_eq!((collapse.width, collapse.height), (1, 1));

    // Clicking the child cell focuses the child.
    let child = state
        .hits
        .agents
        .iter()
        .find(|(_, pane_id)| pane_id == "c1")
        .map(|(rect, _)| *rect)
        .expect("child cell");
    let focus = click(&mut state, child);
    assert!(matches!(
        &focus.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                Method::PaneFocus(PaneTarget { pane_id }) if pane_id == "c1"
            )
    ));

    // And the expanded panel shows the subtree again.
    state.sidebar_collapsed = false;
    state.compose(106, 30).expect("expanded sidebar again");
    assert_eq!(agent_ids(&state), vec!["p1", "c1"]);
}
