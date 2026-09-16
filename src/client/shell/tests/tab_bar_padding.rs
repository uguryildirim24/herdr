use super::*;

fn padded_state(padding_y: u16, position: crate::config::TabBarPositionConfig) -> ClientShellState {
    let mut config = ClientShellConfig::from_config(&Config::default());
    config.tab_bar_padding_y = padding_y;
    config.tab_bar_position = position;
    let mut projected = snapshot();
    for index in 2..=3 {
        let mut tab = projected.tabs[0].clone();
        tab.tab_id = format!("tab_{index}");
        tab.number = index;
        tab.label = index.to_string();
        tab.focused = false;
        projected.tabs.push(tab);
    }
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(surface());
    state
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> RawInputEvent {
    RawInputEvent::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::empty(),
    })
}

fn click_focuses(state: &mut ClientShellState, column: u16, row: u16, tab_id: &str) -> bool {
    state.handle_raw_events(vec![mouse(
        MouseEventKind::Down(MouseButton::Left),
        column,
        row,
    )]);
    let up = state.handle_raw_events(vec![mouse(
        MouseEventKind::Up(MouseButton::Left),
        column,
        row,
    )]);
    matches!(
        &up.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                crate::api::schema::Method::TabFocus(target) if target.tab_id == tab_id
            )
    )
}

#[test]
fn padded_tab_bar_reserves_three_rows_top_and_bottom() {
    use crate::config::TabBarPositionConfig::{Bottom, Top};

    let top = padded_state(1, Top);
    let layout = top.layout(106, 30);
    assert_eq!(layout.tab_bar.y, 0);
    assert_eq!(layout.tab_bar.height, 3);
    assert_eq!(layout.pane_surface.y, 3);
    assert_eq!(layout.pane_surface.height, 27);

    let bottom = padded_state(1, Bottom);
    let layout = bottom.layout(106, 30);
    assert_eq!(layout.tab_bar.y, 27);
    assert_eq!(layout.tab_bar.height, 3);
    assert_eq!(layout.pane_surface.y, 0);
    assert_eq!(layout.pane_surface.height, 27);

    // Terminals too short for padding keep the one-row bar and at least one pane row.
    for rows in [2, 3] {
        let layout = top.layout(106, rows);
        assert_eq!(layout.tab_bar.height, 1);
        assert_eq!(layout.pane_surface.height, rows - 1);
    }
    assert_eq!(top.layout(106, 4).tab_bar.height, 3);
    assert_eq!(top.layout(106, 1).tab_bar.height, 0);

    // Mobile layout ignores the desktop tab bar padding.
    let mobile = top.layout(44, 30);
    assert!(mobile.tab_bar.is_empty());
    assert_eq!(mobile.pane_surface.height, 28);

    let unpadded = padded_state(0, Top).layout(106, 30);
    assert_eq!(unpadded.tab_bar.height, 1);
    assert_eq!(unpadded.pane_surface.height, 29);
}

#[test]
fn padded_tabs_are_solid_blocks_clickable_on_every_row() {
    let mut state = padded_state(1, crate::config::TabBarPositionConfig::Top);
    let frame = state.compose(106, 30).expect("padded tab bar");
    let buffer = frame.to_ratatui_buffer().expect("padded tab buffer");
    let active = state.hits.tabs[0].0;
    let inactive = state.hits.tabs[1].0;
    assert_eq!(active.height, 3);
    assert_eq!(inactive.height, 3);
    assert_eq!(state.hits.new_tab.height, 3);
    let accent = state.config.palette.accent;
    let surface0 = state.config.palette.surface0;
    for y in active.top()..active.bottom() {
        for x in active.left()..active.right() {
            assert_eq!(buffer[(x, y)].bg, accent, "active tab cell ({x}, {y})");
        }
        for x in inactive.left()..inactive.right() {
            assert_eq!(buffer[(x, y)].bg, surface0, "inactive tab cell ({x}, {y})");
        }
    }
    let label_row = |y: u16| {
        (active.left()..active.right())
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
    };
    assert_eq!(label_row(0).trim(), "");
    assert_eq!(label_row(1).trim(), "1");
    assert_eq!(label_row(2).trim(), "");

    assert!(click_focuses(&mut state, inactive.x + 1, 0, "tab_2"));
    let third = state.hits.tabs[2].0;
    assert!(click_focuses(&mut state, third.x + 1, 2, "tab_3"));
}

#[test]
fn padded_tab_drag_drops_from_padding_rows() {
    let mut state = padded_state(1, crate::config::TabBarPositionConfig::Top);
    state.compose(106, 30).expect("padded tabs");
    let first = state.hits.tabs[0].0;
    let third = state.hits.tabs[2].0;
    state.handle_raw_events(vec![mouse(
        MouseEventKind::Down(MouseButton::Left),
        first.x + 1,
        0,
    )]);
    state.handle_raw_events(vec![mouse(
        MouseEventKind::Drag(MouseButton::Left),
        third.right().saturating_sub(1),
        2,
    )]);
    assert!(matches!(
        state.chrome_drag,
        Some(ClientChromeDrag::Tab {
            insert_index: Some(3),
            ..
        })
    ));
    let frame = state.compose(106, 30).expect("padded drop indicator");
    let width = usize::from(frame.width);
    for row in 0..3 {
        assert!(frame.cells[row * width..(row + 1) * width]
            .iter()
            .any(|cell| cell.symbol == "│"));
    }

    state.handle_raw_events(vec![mouse(
        MouseEventKind::Drag(MouseButton::Left),
        third.right().saturating_sub(1),
        3,
    )]);
    assert!(matches!(
        state.chrome_drag,
        Some(ClientChromeDrag::Tab {
            insert_index: None,
            ..
        })
    ));
    state.handle_raw_events(vec![mouse(
        MouseEventKind::Drag(MouseButton::Left),
        third.right().saturating_sub(1),
        1,
    )]);
    let release = state.handle_raw_events(vec![mouse(
        MouseEventKind::Up(MouseButton::Left),
        third.right().saturating_sub(1),
        1,
    )]);
    assert!(matches!(
        &release.actions[..],
        [ClientShellAction::Endpoint { request, .. }]
            if matches!(
                &request.method,
                crate::api::schema::Method::TabMove(params)
                    if params.tab_id == "tab_1" && params.insert_index == 3
            )
    ));
}

#[test]
fn bottom_mode_bar_covers_every_padded_tab_row() {
    for mode in [ClientShellMode::Navigate, ClientShellMode::Prefix] {
        let mut state = padded_state(1, crate::config::TabBarPositionConfig::Bottom);
        state.compose(106, 30).expect("bottom padded tabs");
        let active = state.hits.tabs[0].0;
        assert_eq!((active.y, active.height), (27, 3));

        state.mode = mode;
        let frame = state.compose(106, 30).expect("bottom mode bar");
        let buffer = frame.to_ratatui_buffer().expect("bottom mode bar buffer");
        let accent = state.config.palette.accent;
        for y in 27..29 {
            for x in active.left()..active.right() {
                assert_eq!(buffer[(x, y)].symbol(), " ", "{mode:?} cell ({x}, {y})");
                assert_ne!(buffer[(x, y)].bg, accent, "{mode:?} cell ({x}, {y})");
            }
        }
        assert!(state.hits.tabs.is_empty());
        assert!(state.hits.new_tab.is_empty());
        assert!(state.hits.tab_scroll_left.is_empty());
        assert!(state.hits.tab_scroll_right.is_empty());
        let up = state.handle_raw_events(vec![
            mouse(MouseEventKind::Down(MouseButton::Left), active.x + 1, 27),
            mouse(MouseEventKind::Up(MouseButton::Left), active.x + 1, 27),
        ]);
        assert!(!up.actions.iter().any(|action| matches!(
            action,
            ClientShellAction::Endpoint { request, .. }
                if matches!(&request.method, crate::api::schema::Method::TabFocus(_))
        )));
    }
}

#[test]
fn horizontal_padding_sets_tab_width() {
    let tab_width = |padding_x: u16| {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.tab_bar_padding_x = padding_x;
        let mut projected = snapshot();
        projected.tabs[0].label = "abcdefgh".into();
        let mut state = ClientShellState::new(config);
        state.set_snapshot(Box::new(projected));
        state.set_pane_surface(surface());
        state.compose(106, 30).expect("tab width");
        state.hits.tabs[0].0.width
    };
    assert_eq!(tab_width(2), 12);
    assert_eq!(tab_width(3), 14);
    assert_eq!(tab_width(0), 8);

    let short_label = |padding_x: u16| {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.tab_bar_padding_x = padding_x;
        let mut state = ClientShellState::new(config);
        state.set_snapshot(Box::new(snapshot()));
        state.set_pane_surface(surface());
        state.compose(106, 30).expect("short tab width");
        state.hits.tabs[0].0.width
    };
    assert_eq!(short_label(2), 8);
    assert_eq!(short_label(4), 12);
}

#[test]
fn oversized_horizontal_padding_is_clamped_with_diagnostic() {
    let mut config = Config::default();
    config.ui.tab_bar_padding_x = u16::MAX;
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&config));
    assert_eq!(state.config.tab_bar_padding_x, 8);
    let mut projected = snapshot();
    projected.tabs[0].label = "abcdefgh".into();
    state.set_snapshot(Box::new(projected));
    state.set_pane_surface(surface());
    state.compose(106, 30).expect("clamped tab width");
    // label 8 + 2 * 8 padding columns
    assert_eq!(state.hits.tabs[0].0.width, 24);

    let mut next = Config::default();
    next.ui.tab_bar_padding_x = 9;
    let diagnostics = state.config.apply_live_config(&next, &[], &[]);
    assert_eq!(state.config.tab_bar_padding_x, 8);
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("ui.tab_bar_padding_x")
            && diagnostic.contains("got 9")));
}
