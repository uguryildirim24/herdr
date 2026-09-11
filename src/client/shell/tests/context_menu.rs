use ratatui::layout::Rect;

use super::*;
use crate::api::schema::AgentStatus;
use crate::protocol::ClientShellAgent;

#[test]
fn context_menu_on_parent_agent_row_offers_expand_collapse_and_toggles_key() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let mut state = ClientShellState::new(config);

    let mut snap = snapshot();
    snap.agents = vec![
        ClientShellAgent {
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
        },
        ClientShellAgent {
            pane_id: "p2".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("worker".into()),
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
    state.set_snapshot(Box::new(snap));

    // The rendered chevron targets are the authority on which rows are parents.
    state
        .hits
        .agent_group_toggles
        .push((Rect::new(29, 5, 1, 1), "p1".into(), "agent:p1".into()));

    // Open context menu on parent agent
    assert!(state.open_agent_context_menu("p1", 10, 5));

    // Verify context menu overlay opened with "Collapse" option
    let Some(ClientShellOverlay::ContextMenu(menu)) = &state.overlay else {
        panic!("context menu overlay should be open");
    };
    assert_eq!(menu.items().len(), 1);
    assert_eq!(menu.items()[0].label, "Collapse");

    // Activate the item (ToggleGroup)
    let mut outcome = ClientShellInput::default();
    state.activate_context_menu_item(0, &mut outcome);

    // Verify key added to collapsed_groups
    assert!(state.collapsed_groups.contains("agent:p1"));

    // Open context menu on parent agent again (now collapsed)
    assert!(state.open_agent_context_menu("p1", 10, 5));
    let Some(ClientShellOverlay::ContextMenu(menu_collapsed)) = &state.overlay else {
        panic!("context menu overlay should be open");
    };
    assert_eq!(menu_collapsed.items()[0].label, "Expand");

    // Activate Expand
    let mut outcome = ClientShellInput::default();
    state.activate_context_menu_item(0, &mut outcome);
    assert!(!state.collapsed_groups.contains("agent:p1"));

    // A row without a rendered chevron is not a parent: no menu, and the caller is told so.
    state.overlay = None;
    assert!(!state.open_agent_context_menu("p2", 10, 6));
    assert!(state.overlay.is_none());
}

#[test]
fn agent_context_menu_uses_the_machine_prefixed_group_key() {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = true;
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(snapshot()));
    state.hits.agent_group_toggles.push((
        Rect::new(29, 5, 1, 1),
        "w1:p1".into(),
        "agent:remote:w1:p1".into(),
    ));

    assert!(state.open_agent_context_menu("w1:p1", 10, 5));
    let mut outcome = ClientShellInput::default();
    state.activate_context_menu_item(0, &mut outcome);

    assert!(state.collapsed_groups.contains("agent:remote:w1:p1"));
    assert!(!state.collapsed_groups.contains("agent:w1:p1"));
}
