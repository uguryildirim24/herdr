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
    assert_eq!(buffer[(toggle.x, toggle.y)].symbol(), "▼");
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
    // A nested row drops its top padding (the parent's bottom padding already separates them),
    // and a child in its parent's workspace is named by its agent label, so the default second
    // line holding that label goes away.
    assert_eq!(child.height, 1 + 1);
    assert_eq!(child.y, parent.bottom());
    assert!(row_has_bg(&buffer, parent, parent.y, active));
    assert!(row_has_bg(&buffer, parent, parent.bottom() - 1, active));
    assert!(row_text(&buffer, parent, parent.y).trim().is_empty());
    assert!(row_text(&buffer, parent, parent.y + 2).contains("p1"));
    assert!(row_text(&buffer, child, child.y).contains("╰─"));
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

#[test]
fn row_gap_applies_to_the_multi_machine_sidebar() {
    let multi_machine = |row_gap: u16, rows: u16| {
        let mut config = padded_config(0, 0);
        config.spaces.row_gap = row_gap;
        let mut state = ClientShellState::new(config);
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
        state.set_snapshot(Box::new(workspaces(2, 1)));
        state.set_pane_surface(surface());
        let mut remote = workspaces(2, 1);
        remote.boot_id = "remote-boot".into();
        state.set_endpoint_snapshot(&endpoint_id, Box::new(remote));
        state.compose(100, rows).expect("multi-machine frame");
        state
    };

    let state = multi_machine(1, 60);
    let machines = state
        .hits
        .machines
        .iter()
        .map(|hit| hit.rect)
        .collect::<Vec<_>>();
    let spaces = state
        .hits
        .workspaces
        .iter()
        .map(|hit| hit.rect)
        .collect::<Vec<_>>();
    assert_eq!(machines.len(), 2);
    assert_eq!(spaces.len(), 4);
    // Header, space, gap, space, gap, header, space, gap, space.
    assert_eq!(spaces[0].y, machines[0].bottom());
    assert_eq!(spaces[1].y, spaces[0].bottom() + 1);
    assert_eq!(machines[1].y, spaces[1].bottom() + 1);
    assert_eq!(spaces[2].y, machines[1].bottom());
    assert_eq!(spaces[3].y, spaces[2].bottom() + 1);

    // Gap rows count toward the scroll range: 3 gaps more content than without gaps.
    let short = |row_gap| {
        let state = multi_machine(row_gap, 22);
        state.hits.workspace_scroll_metrics.expect("scroll metrics")
    };
    let (flat, gapped) = (short(0), short(1));
    assert!(gapped.max_offset_from_bottom > flat.max_offset_from_bottom);
}

/// p1 ├ c1 (child g1) ├ c2 └, then top-level p2: exercises sibling lines, a parent's line
/// down to its child, ancestor lines past a grandchild, and a last child with no line.
fn tree_line_snapshot() -> ClientShellSnapshot {
    let mut projected = snapshot();
    projected.agents = vec![
        agent("p1", 1, None, false),
        agent("c1", 2, Some("p1"), false),
        agent("g1", 3, Some("c1"), false),
        agent("c2", 4, Some("p1"), false),
        agent("p2", 5, None, false),
    ];
    projected
}

fn agent_hit(state: &ClientShellState, pane_id: &str) -> Rect {
    state
        .hits
        .agents
        .iter()
        .find(|(_, id)| id == pane_id)
        .map(|(rect, _)| *rect)
        .expect("agent hit")
}

/// The tree column glyphs (columns 1, 3 and 5, under the icons of depth 0, 1 and 2) on row `y`.
fn tree_columns(buffer: &Buffer, x: u16, y: u16) -> [String; 3] {
    [1, 3, 5].map(|offset| buffer[(x + offset, y)].symbol().to_string())
}

fn tree_line_frame(padding: u16, gap: u16) -> (ClientShellState, Buffer) {
    let mut config = padded_config(0, padding);
    config.agents.row_gap = gap;
    config.agent_parent_nesting = true;
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(tree_line_snapshot()));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 60).expect("tree line frame");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    (state, buffer)
}

#[test]
fn padded_nested_agent_rows_keep_tree_lines_continuous() {
    let (state, buffer) = tree_line_frame(1, 0);
    let [p1, c1, g1, c2, p2] = ["p1", "c1", "g1", "c2", "p2"].map(|id| agent_hit(&state, id));
    let x = p1.x;
    let col = |y| tree_columns(&buffer, x, y);
    let s = |glyphs: [&str; 3]| glyphs.map(str::to_string);

    // p1: no line above a top-level row; its line to c1 starts under its icon on its second
    // content row and runs through its bottom padding.
    assert_eq!(col(p1.y), s([" ", " ", " "]));
    assert_eq!(col(p1.y + 2)[0], "│");
    assert_eq!(col(p1.bottom() - 1), s(["│", " ", " "]));
    // c1: no top padding; the content row branches, and its bottom padding carries its sibling
    // line (c2 follows) and its own line down to g1.
    assert_eq!(c1.y, p1.bottom());
    assert_eq!(col(c1.y)[0], "├");
    assert_eq!(col(c1.bottom() - 1), s(["│", "│", " "]));
    // g1: the ancestor line of c1 runs through every row, including its content row.
    assert_eq!(col(g1.y), s(["│", "╰", "●"]));
    assert_eq!(col(g1.bottom() - 1), s(["│", " ", " "]));
    // c2: last child, so nothing continues below its branch.
    assert_eq!(col(c2.y)[0], "╰");
    assert_eq!(col(c2.bottom() - 1), s([" ", " ", " "]));
    assert_eq!(col(p2.y), s([" ", " ", " "]));

    // Connector glyphs use the existing connector colour.
    let overlay0 = state.config.palette.overlay0;
    assert_eq!(buffer[(x + 1, c1.bottom() - 1)].fg, overlay0);
    assert_eq!(buffer[(x + 1, c1.y)].fg, overlay0);
}

#[test]
fn row_gap_rows_between_nested_agents_carry_tree_lines() {
    let (state, buffer) = tree_line_frame(0, 1);
    let [p1, c1, g1, c2, p2] = ["p1", "c1", "g1", "c2", "p2"].map(|id| agent_hit(&state, id));
    let x = p1.x;
    let s = |glyphs: [&str; 3]| glyphs.map(str::to_string);
    assert_eq!(c1.y, p1.bottom() + 1);
    assert_eq!(tree_columns(&buffer, x, p1.bottom()), s(["│", " ", " "]));
    assert_eq!(tree_columns(&buffer, x, c1.bottom()), s(["│", "│", " "]));
    assert_eq!(tree_columns(&buffer, x, g1.bottom()), s(["│", " ", " "]));
    assert_eq!(tree_columns(&buffer, x, c2.bottom()), s([" ", " ", " "]));
    assert_eq!(p2.y, c2.bottom() + 1);
}

#[test]
fn unpadded_flat_agent_rows_draw_no_tree_lines() {
    let mut config = padded_config(0, 1);
    config.agents.row_gap = 1;
    config.agent_parent_nesting = false;
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(tree_line_snapshot()));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 60).expect("flat frame");
    let buffer = frame.to_ratatui_buffer().expect("frame should reconstruct");
    for (rect, _) in &state.hits.agents {
        for y in [rect.y, rect.bottom() - 1, rect.bottom()] {
            assert!(!row_text(&buffer, *rect, y).contains('│'));
        }
    }
}
