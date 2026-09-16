use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::api::schema::{AgentStatus, Method, PaneTarget};
use crate::client::endpoint::{
    ClientEndpointId, ClientEndpointStatus, ProfileId, SavedSshEndpoint,
};
use crate::protocol::ClientShellAgent;

fn padded_config(spaces: u16, agents: u16) -> ClientShellConfig {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.spaces.row_padding = spaces;
    config.agents.row_padding = agents;
    config
}

fn workspaces(count: usize, focused: usize) -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    let template = snapshot.workspaces[0].clone();
    snapshot.workspaces = (1..=count)
        .map(|number| ClientShellWorkspace {
            workspace_id: format!("ws_{number}"),
            number,
            label: format!("space-{number}"),
            focused: number == focused,
            ..template.clone()
        })
        .collect();
    snapshot.focused_workspace_id = Some(format!("ws_{focused}"));
    snapshot
}

fn agent(pane_id: &str, seq: u64, parent: Option<&str>, focused: bool) -> ClientShellAgent {
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
        focused,
    }
}

fn row_text(buffer: &Buffer, rect: Rect, y: u16) -> String {
    (rect.x..rect.right())
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

fn row_has_bg(buffer: &Buffer, rect: Rect, y: u16, bg: ratatui::style::Color) -> bool {
    (rect.x..rect.right()).all(|x| buffer[(x, y)].bg == bg)
}

fn click(state: &mut ClientShellState, column: u16, row: u16) -> ClientShellInput {
    state.handle_raw_events(vec![
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }),
        RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::empty(),
        }),
    ])
}

#[test]
fn default_space_and_agent_rows_have_no_padding() {
    let mut snapshot = workspaces(2, 1);
    snapshot.agents = vec![agent("p1", 1, None, true), agent("p2", 2, None, false)];
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 40).expect("default sidebar");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");

    let first = state.hits.workspaces[0].rect;
    let second = state.hits.workspaces[1].rect;
    assert_eq!(first.height, 2);
    assert_eq!(second.y, first.bottom());
    assert!(row_text(&buffer, first, first.y).contains("space-1"));

    let (agent_rect, _) = &state.hits.agents[0];
    assert_eq!(agent_rect.height, 2);
    assert_eq!(state.hits.agents[1].0.y, agent_rect.bottom());
    assert!(row_text(&buffer, *agent_rect, agent_rect.y + 1).contains("p1"));
}

#[test]
fn padded_space_rows_extend_highlight_and_hit_target() {
    let mut state = ClientShellState::new(padded_config(1, 0));
    state.set_snapshot(Box::new(workspaces(2, 1)));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 30).expect("padded spaces");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    let active = state.config.palette.active_row_bg;

    let first = state.hits.workspaces[0].rect;
    let second = state.hits.workspaces[1].rect;
    assert_eq!(first.height, 2 + 2);
    assert_eq!(second.height, 2 + 2);
    assert_eq!(second.y, first.bottom());
    assert!(row_has_bg(&buffer, first, first.y, active));
    assert!(row_has_bg(&buffer, first, first.bottom() - 1, active));
    assert!(row_text(&buffer, first, first.y).trim().is_empty());
    assert!(row_text(&buffer, first, first.y + 1).contains("space-1"));
    assert!(row_text(&buffer, first, first.y + 2).contains("main"));
    assert!(!row_has_bg(&buffer, second, second.y, active));

    // The top padding row of the second space belongs to its hit rect.
    let outcome = click(&mut state, second.x + 2, second.y);
    assert!(matches!(
        &outcome.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                Method::WorkspaceFocus(target) if target.workspace_id == "ws_2"
            )
    ));
}

#[test]
fn padded_worktree_group_is_padded_as_one_block() {
    let mut projected = snapshot();
    projected.workspaces[0].worktree = Some(ClientShellWorktree {
        key: "repo".into(),
        label: "repo".into(),
        is_linked_worktree: false,
    });
    let mut child = projected.workspaces[0].clone();
    child.workspace_id = "ws_child".into();
    child.number = 2;
    child.label = "feature".into();
    child.focused = false;
    child.worktree = Some(ClientShellWorktree {
        key: "repo".into(),
        label: "repo".into(),
        is_linked_worktree: true,
    });
    let mut other = projected.workspaces[0].clone();
    other.workspace_id = "ws_other".into();
    other.number = 3;
    other.label = "other".into();
    other.focused = false;
    other.worktree = None;
    projected.workspaces.extend([child, other]);

    let mut unpadded = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    unpadded.set_snapshot(Box::new(projected.clone()));
    unpadded.set_pane_surface(surface());
    unpadded.compose(106, 30).expect("unpadded group");
    let heights = unpadded
        .hits
        .workspaces
        .iter()
        .map(|hit| hit.rect.height)
        .collect::<Vec<_>>();

    let mut state = ClientShellState::new(padded_config(1, 0));
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 30).expect("padded group");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    let parent = &state.hits.workspaces[0];
    let child = &state.hits.workspaces[1];
    let other = &state.hits.workspaces[2];
    assert!(child.indented);

    // Parent: top padding only. Child: bottom padding only. Standalone: both.
    assert_eq!(parent.rect.height, heights[0] + 1);
    assert_eq!(child.rect.y, parent.rect.bottom());
    assert_eq!(child.rect.height, heights[1] + 1);
    assert_eq!(other.rect.y, child.rect.bottom());
    assert_eq!(other.rect.height, heights[2] + 2);
    assert!(row_text(&buffer, child.rect, child.rect.y).contains("└─"));

    let (toggle, _) = parent.group_toggle.as_ref().expect("group chevron");
    assert_eq!(toggle.y, parent.rect.y + 1);
    assert_eq!(buffer[(toggle.x, toggle.y)].symbol(), "▾");
}

#[test]
fn reveal_scrolls_padded_focused_space_fully_into_view() {
    let mut state = ClientShellState::new(padded_config(1, 0));
    state.set_snapshot(Box::new(workspaces(12, 1)));
    state.set_pane_surface(surface());
    state.compose(106, 20).expect("padded sidebar");
    assert!(state.hits.workspace_max_scroll > 0);
    assert!(state
        .hits
        .workspaces
        .iter()
        .all(|hit| hit.workspace_id != "ws_12"));

    let mut update = workspaces(12, 12);
    update.revision = 2;
    let mut updated_surface = surface();
    updated_surface.projection_revision = 2;
    state.set_snapshot(Box::new(update));
    state.set_pane_surface(updated_surface);
    state.compose(106, 20).expect("revealed padded sidebar");

    let body = state.hits.workspace_body;
    let focused = state
        .hits
        .workspaces
        .iter()
        .find(|hit| hit.workspace_id == "ws_12")
        .expect("focused space revealed")
        .rect;
    assert_eq!(focused.height, 4);
    assert!(focused.y >= body.y && focused.bottom() <= body.bottom());
}

#[test]
fn padded_nested_agent_rows_extend_highlight_and_hit_target() {
    let mut config = padded_config(0, 1);
    config.agent_parent_nesting = true;
    let mut projected = snapshot();
    projected.agents = vec![
        agent("p1", 1, None, true),
        agent("p2", 2, Some("p1"), false),
    ];
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 40).expect("padded agents");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    let active = state.config.palette.active_row_bg;

    let (parent, parent_id) = state.hits.agents[0].clone();
    let (child, child_id) = state.hits.agents[1].clone();
    assert_eq!((parent_id.as_str(), child_id.as_str()), ("p1", "p2"));
    assert_eq!(parent.height, 2 + 2);
    assert_eq!(child.height, 2 + 2);
    assert_eq!(child.y, parent.bottom());
    assert!(row_has_bg(&buffer, parent, parent.y, active));
    assert!(row_has_bg(&buffer, parent, parent.bottom() - 1, active));
    assert!(row_text(&buffer, parent, parent.y).trim().is_empty());
    assert!(row_text(&buffer, parent, parent.y + 2).contains("p1"));
    assert!(row_text(&buffer, child, child.y + 1).contains("└─"));
    assert!(!row_has_bg(&buffer, child, child.y, active));

    let (toggle, pane_id, _) = &state.hits.agent_group_toggles[0];
    assert_eq!(pane_id, "p1");
    assert_eq!(toggle.y, parent.y + 1);

    let outcome = click(&mut state, child.x + 4, child.bottom() - 1);
    assert!(matches!(
        &outcome.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                Method::PaneFocus(PaneTarget { pane_id }) if pane_id == "p2"
            )
    ));
}

#[test]
fn padded_rows_apply_to_the_multi_machine_sidebar() {
    let mut state = ClientShellState::new(padded_config(1, 1));
    let profile = SavedSshEndpoint {
        id: ProfileId::parse("0123456789abcdef0123456789abcdef").expect("profile id"),
        label: "Build".into(),
        target: "dev@build.example".into(),
        session: "agents".into(),
        enabled: true,
    };
    let endpoint_id = ClientEndpointId::Ssh(profile.id.clone());
    state.set_endpoint_catalog(&[profile]);
    state.set_endpoint_status(&endpoint_id, ClientEndpointStatus::Online);
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    let mut remote = snapshot();
    remote.boot_id = "remote-boot".into();
    remote.workspaces[0].label = "remote-workspace".into();
    remote.agents = vec![agent("remote-agent", 1, None, true)];
    state.set_endpoint_snapshot(&endpoint_id, Box::new(remote));
    assert!(state.activate_endpoint_projection(&endpoint_id));
    let mut remote_surface = surface();
    remote_surface.boot_id = "remote-boot".into();
    state.set_pane_surface(remote_surface);

    let frame = state.compose(100, 40).expect("multi-machine frame");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    let active = state.config.palette.active_row_bg;
    let workspace = state
        .hits
        .workspaces
        .iter()
        .find(|hit| hit.endpoint_id == endpoint_id)
        .expect("remote workspace hit")
        .rect;
    assert_eq!(workspace.height, 2 + 2);
    assert_eq!(buffer[(workspace.x + 2, workspace.y)].bg, active);
    assert_eq!(buffer[(workspace.x + 2, workspace.bottom() - 1)].bg, active);
    assert!(row_text(&buffer, workspace, workspace.y + 1).contains("remote-workspace"));

    let (agent_rect, _, _) = state
        .hits
        .endpoint_agents
        .iter()
        .find(|(_, id, _)| *id == endpoint_id)
        .expect("remote agent hit");
    assert_eq!(agent_rect.height, 2 + 2);
    assert!(row_has_bg(&buffer, *agent_rect, agent_rect.y, active));
    assert!(row_text(&buffer, *agent_rect, agent_rect.y + 2).contains("remote-agent"));
}
