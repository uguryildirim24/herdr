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

    // Open context menu on parent agent
    state.open_agent_context_menu("p1".into(), 10, 5);

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
    assert!(state.config.collapsed_groups.contains("agent:p1"));

    // Open context menu on parent agent again (now collapsed)
    state.open_agent_context_menu("p1".into(), 10, 5);
    let Some(ClientShellOverlay::ContextMenu(menu_collapsed)) = &state.overlay else {
        panic!("context menu overlay should be open");
    };
    assert_eq!(menu_collapsed.items()[0].label, "Expand");

    // Activate Expand
    let mut outcome = ClientShellInput::default();
    state.activate_context_menu_item(0, &mut outcome);
    assert!(!state.collapsed_groups.contains("agent:p1"));
    assert!(!state.config.collapsed_groups.contains("agent:p1"));
}
