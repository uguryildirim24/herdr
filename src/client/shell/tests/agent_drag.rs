use ratatui::layout::Rect;

use super::*;
use crate::api::schema::{AgentStatus, Method, PaneTarget};
use crate::protocol::ClientShellAgent;

fn agent(pane_id: &str, seq: u64, parent: Option<&str>) -> ClientShellAgent {
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
        state_change_seq: seq,
        state_labels: Vec::new(),
        tokens: parent
            .map(|parent| vec![("parent".to_string(), parent.to_string())])
            .unwrap_or_default(),
        focused: false,
    }
}

/// coord with child lane, a flat worker, and an agent in another workspace.
fn nesting_state(nesting: bool) -> ClientShellState {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.agent_parent_nesting = nesting;
    let mut projected = snapshot();
    let mut other = projected.workspaces[0].clone();
    other.workspace_id = "ws_2".into();
    other.focused = false;
    projected.workspaces.push(other);
    let mut elsewhere = agent("elsewhere", 4, None);
    elsewhere.workspace_id = "ws_2".into();
    projected.agents = vec![
        agent("coord", 1, None),
        agent("lane", 2, Some("coord")),
        agent("worker", 3, None),
        elsewhere,
    ];
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(surface());
    compose(&mut state);
    state
}

fn compose(state: &mut ClientShellState) -> ratatui::buffer::Buffer {
    let frame = state.compose(106, 40).expect("frame");
    frame.to_ratatui_buffer().expect("frame should reconstruct")
}

fn row(state: &ClientShellState, pane_id: &str) -> Rect {
    state
        .hits
        .agents
        .iter()
        .find(|(_, id)| id == pane_id)
        .map(|(rect, _)| *rect)
        .expect("agent row")
}

fn mouse(kind: MouseEventKind, (column, row): (u16, u16)) -> RawInputEvent {
    RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::empty(),
    })
}

fn at(rect: Rect) -> (u16, u16) {
    (rect.x + 3, rect.y)
}

fn drag(state: &mut ClientShellState, from: (u16, u16), to: (u16, u16)) -> ClientShellInput {
    state.handle_raw_events(vec![
        mouse(MouseEventKind::Down(MouseButton::Left), from),
        mouse(MouseEventKind::Drag(MouseButton::Left), to),
        mouse(MouseEventKind::Up(MouseButton::Left), to),
    ])
}

/// The `parent` token a drop reported for `pane_id`: `Some(None)` clears it.
fn reported_parent(outcome: &ClientShellInput, pane_id: &str) -> Option<Option<String>> {
    outcome.actions.iter().find_map(|action| match action {
        ClientShellAction::Endpoint { request, .. } => match &request.method {
            Method::PaneReportMetadata(params) if params.pane_id == pane_id => {
                params.tokens.get("parent").cloned()
            }
            _ => None,
        },
        _ => None,
    })
}

fn focused_pane(outcome: &ClientShellInput) -> Option<String> {
    outcome.actions.iter().find_map(|action| match action {
        ClientShellAction::Endpoint { request, .. } => match &request.method {
            Method::PaneFocus(PaneTarget { pane_id }) => Some(pane_id.clone()),
            _ => None,
        },
        _ => None,
    })
}

#[test]
fn dropping_an_agent_on_another_nests_it() {
    let mut state = nesting_state(true);
    let (worker, coord) = (row(&state, "worker"), row(&state, "coord"));
    let outcome = drag(&mut state, at(worker), at(coord));
    assert_eq!(
        reported_parent(&outcome, "worker"),
        Some(Some("coord".to_string()))
    );
    assert_eq!(focused_pane(&outcome), None);
}

#[test]
fn dropping_a_child_on_empty_panel_space_unnests_it() {
    let mut state = nesting_state(true);
    let lane = row(&state, "lane");
    let body = state.hits.agent_body;
    let outcome = drag(&mut state, at(lane), (body.x + 3, body.bottom() - 1));
    assert_eq!(reported_parent(&outcome, "lane"), Some(None));
}

#[test]
fn invalid_drops_report_nothing() {
    let mut state = nesting_state(true);
    let (coord, lane, worker) = (
        row(&state, "coord"),
        row(&state, "lane"),
        row(&state, "worker"),
    );
    let body = state.hits.agent_body;
    // Onto its own descendant: would make a cycle.
    assert_eq!(
        reported_parent(&drag(&mut state, at(coord), at(lane)), "coord"),
        None
    );
    // Onto its current parent: nothing changes.
    assert_eq!(
        reported_parent(&drag(&mut state, at(lane), at(coord)), "lane"),
        None
    );
    // A top-level agent onto empty space: nothing to un-nest.
    let empty = (body.x + 3, body.bottom() - 1);
    assert_eq!(
        reported_parent(&drag(&mut state, at(worker), empty), "worker"),
        None
    );
    // Onto an agent in another workspace: it could never nest there.
    let elsewhere = row(&state, "elsewhere");
    assert_eq!(
        reported_parent(&drag(&mut state, at(worker), at(elsewhere)), "worker"),
        None
    );
}

#[test]
fn a_click_without_movement_still_focuses_the_agent() {
    let mut state = nesting_state(true);
    let worker = row(&state, "worker");
    let outcome = drag(&mut state, at(worker), at(worker));
    assert_eq!(focused_pane(&outcome).as_deref(), Some("worker"));
    assert_eq!(reported_parent(&outcome, "worker"), None);
}

#[test]
fn without_nesting_a_press_focuses_and_rows_do_not_drag() {
    let mut state = nesting_state(false);
    let (worker, coord) = (row(&state, "worker"), row(&state, "coord"));
    let press = state.handle_raw_events(vec![mouse(
        MouseEventKind::Down(MouseButton::Left),
        at(worker),
    )]);
    assert_eq!(focused_pane(&press).as_deref(), Some("worker"));
    let rest = state.handle_raw_events(vec![
        mouse(MouseEventKind::Drag(MouseButton::Left), at(coord)),
        mouse(MouseEventKind::Up(MouseButton::Left), at(coord)),
    ]);
    assert_eq!(reported_parent(&rest, "worker"), None);
}

#[test]
fn dragging_marks_the_target_and_names_the_drop_in_the_header() {
    let mut state = nesting_state(true);
    let (worker, coord) = (row(&state, "worker"), row(&state, "coord"));
    state.handle_raw_events(vec![
        mouse(MouseEventKind::Down(MouseButton::Left), at(worker)),
        mouse(MouseEventKind::Drag(MouseButton::Left), at(coord)),
    ]);
    let buffer = compose(&mut state);
    let accent = state.config.palette.accent;
    assert_eq!(buffer[(coord.x, coord.y)].symbol(), "▌");
    assert_eq!(buffer[(coord.x, coord.y)].fg, accent);
    let body = state.hits.agent_body;
    let header: String = (body.x..body.right())
        .map(|x| buffer[(x, body.y - 2)].symbol())
        .collect();
    assert!(header.trim_end().ends_with("nest"), "{header:?}");
    let worker = row(&state, "worker");
    assert!(buffer[(worker.x + 3, worker.y)]
        .modifier
        .contains(ratatui::style::Modifier::DIM));
}
