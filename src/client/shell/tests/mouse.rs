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
        .push((Rect::new(29, 5, 1, 1), "agent:p1".into()));

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
    assert!(state.config.collapsed_groups.contains("agent:p1"));
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
    assert!(!state.config.collapsed_groups.contains("agent:p1"));
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
