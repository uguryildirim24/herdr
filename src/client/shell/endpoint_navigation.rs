use super::*;

impl ClientShellState {
    pub(super) fn active_endpoint_workspace_at(&self, point: (u16, u16)) -> Option<String> {
        self.hits
            .workspaces
            .iter()
            .find(|hit| {
                hit.endpoint_id == self.active_endpoint_id && super::contains(hit.rect, point)
            })
            .map(|hit| hit.workspace_id.clone())
    }

    pub(super) fn endpoint_workspace_is_draggable(&self, press: &ClientWorkspacePress) -> bool {
        press.endpoint_id == self.active_endpoint_id
            && self
                .snapshot
                .as_deref()
                .and_then(|snapshot| {
                    snapshot
                        .workspaces
                        .iter()
                        .find(|workspace| workspace.workspace_id == press.workspace_id)
                })
                .is_some_and(|workspace| {
                    !workspace
                        .worktree
                        .as_ref()
                        .is_some_and(|worktree| worktree.is_linked_worktree)
                })
    }

    pub(super) fn finish_endpoint_workspace_press(
        &mut self,
        press: ClientWorkspacePress,
        outcome: &mut ClientShellInput,
    ) {
        self.focus_or_activate(
            press.endpoint_id,
            ClientEndpointFocusTarget::Workspace(press.workspace_id),
            outcome,
        );
    }

    pub(super) fn handle_endpoint_machine_click(
        &mut self,
        point: (u16, u16),
        outcome: &mut ClientShellInput,
    ) -> bool {
        let Some(hit) = self
            .hits
            .machines
            .iter()
            .find(|hit| super::contains(hit.rect, point))
        else {
            return false;
        };
        let endpoint_id = hit.endpoint_id.clone();
        let collapse_toggle = super::contains(hit.collapse_toggle, point);
        if collapse_toggle || endpoint_id == self.active_endpoint_id {
            if !self.collapsed_endpoints.remove(&endpoint_id) {
                self.collapsed_endpoints.insert(endpoint_id.clone());
            }
            outcome.repaint = true;
            if !collapse_toggle && endpoint_id.is_local() {
                self.activate_endpoint(endpoint_id, outcome);
            }
        } else if endpoint_id.is_local() || self.endpoint_is_online(&endpoint_id) {
            outcome.actions.push(ClientShellAction::ActivateEndpoint {
                endpoint_id,
                target: None,
            });
        } else {
            let label = self.endpoint_label(&endpoint_id).to_owned();
            self.receive_endpoint_unavailable(format!("{label} is not ready"));
            outcome.repaint = true;
        }
        true
    }

    pub(super) fn handle_endpoint_agent_click(
        &mut self,
        point: (u16, u16),
        outcome: &mut ClientShellInput,
    ) -> bool {
        let Some((endpoint_id, pane_id)) = self
            .hits
            .endpoint_agents
            .iter()
            .find(|(rect, _, _)| super::contains(*rect, point))
            .map(|(_, endpoint_id, pane_id)| (endpoint_id.clone(), pane_id.clone()))
        else {
            return false;
        };
        self.focus_or_activate(
            endpoint_id,
            ClientEndpointFocusTarget::Pane(pane_id),
            outcome,
        );
        true
    }

    pub(super) fn handle_endpoint_navigation(
        &mut self,
        action: crate::input::KeybindAction,
        outcome: &mut ClientShellInput,
    ) -> bool {
        use crate::input::KeybindAction;
        if !self.multi_endpoint_active() {
            return false;
        }
        if matches!(
            action,
            KeybindAction::PreviousWorkspace | KeybindAction::NextWorkspace
        ) {
            // Walk the merged rows once: a name on two machines is one step.
            let spaces = super::linked_spaces::linked_spaces(
                &self.endpoints,
                &self.collapsed_endpoints,
                &self.collapsed_groups,
                &self.remote_collapsed_groups,
            );
            if spaces.is_empty() {
                return true;
            }
            let focused = self
                .snapshot
                .as_deref()
                .and_then(|snapshot| snapshot.focused_workspace_id.as_deref());
            let current = spaces.iter().position(|space| {
                space.parts.iter().any(|part| {
                    part.endpoint_id == &self.active_endpoint_id
                        && Some(part.workspace.workspace_id.as_str()) == focused
                })
            });
            let next = match (current, action) {
                (Some(index), KeybindAction::PreviousWorkspace) => {
                    (index + spaces.len() - 1) % spaces.len()
                }
                (Some(index), KeybindAction::NextWorkspace) => (index + 1) % spaces.len(),
                (None, KeybindAction::PreviousWorkspace) => spaces.len() - 1,
                (None, KeybindAction::NextWorkspace) => 0,
                _ => unreachable!("endpoint workspace navigation"),
            };
            let space = &spaces[next];
            let part = &space.parts[space.primary_part_index(&self.active_endpoint_id)];
            self.focus_or_activate(
                part.endpoint_id.clone(),
                ClientEndpointFocusTarget::Workspace(part.workspace.workspace_id.clone()),
                outcome,
            );
            return true;
        }
        if matches!(action, KeybindAction::PreviousTab | KeybindAction::NextTab) {
            // Walk the merged tab strip of a linked space, crossing machines.
            let tabs = super::linked_spaces::linked_tabs(&self.endpoints, &self.active_endpoint_id);
            if tabs.iter().any(|entry| !entry.active_endpoint) {
                let focused = self
                    .snapshot
                    .as_deref()
                    .and_then(|snapshot| snapshot.focused_tab_id.as_deref());
                let current = tabs.iter().position(|entry| {
                    entry.active_endpoint && Some(entry.tab.tab_id.as_str()) == focused
                });
                let next = match (current, action) {
                    (Some(index), KeybindAction::PreviousTab) => {
                        (index + tabs.len() - 1) % tabs.len()
                    }
                    (Some(index), KeybindAction::NextTab) => (index + 1) % tabs.len(),
                    (None, KeybindAction::PreviousTab) => tabs.len() - 1,
                    _ => 0,
                };
                let entry = &tabs[next];
                if entry.active_endpoint {
                    self.push_endpoint_method(
                        crate::api::schema::Method::TabFocus(crate::api::schema::TabTarget {
                            tab_id: entry.tab.tab_id.clone(),
                        }),
                        outcome,
                    );
                } else {
                    self.focus_or_activate(
                        entry.endpoint_id.clone(),
                        ClientEndpointFocusTarget::Tab(entry.tab.tab_id.clone()),
                        outcome,
                    );
                }
                return true;
            }
        }
        if matches!(
            action,
            KeybindAction::PreviousAgent | KeybindAction::NextAgent | KeybindAction::FocusAgent(_)
        ) {
            let agents = super::aggregate_navigation::online_agent_targets(
                &self.endpoints,
                &self.active_endpoint_id,
                &self.config,
                &self.collapsed_groups,
            );
            if agents.is_empty() {
                return true;
            }
            let next = match action {
                KeybindAction::FocusAgent(index) => {
                    if index >= agents.len() {
                        return true;
                    }
                    index
                }
                KeybindAction::PreviousAgent | KeybindAction::NextAgent => {
                    let focused = self
                        .snapshot
                        .as_deref()
                        .and_then(|snapshot| snapshot.focused_pane_id.as_deref());
                    let current = agents.iter().position(|target| {
                        target.endpoint_id == self.active_endpoint_id
                            && Some(target.pane_id.as_str()) == focused
                    });
                    match (current, action) {
                        (Some(index), KeybindAction::PreviousAgent) => {
                            (index + agents.len() - 1) % agents.len()
                        }
                        (Some(index), KeybindAction::NextAgent) => (index + 1) % agents.len(),
                        (None, KeybindAction::PreviousAgent) => agents.len() - 1,
                        _ => 0,
                    }
                }
                _ => unreachable!("endpoint agent navigation"),
            };
            let target = &agents[next];
            self.focus_or_activate(
                target.endpoint_id.clone(),
                ClientEndpointFocusTarget::Pane(target.pane_id.clone()),
                outcome,
            );
            return true;
        }
        false
    }

    pub(super) fn activate_endpoint(
        &mut self,
        endpoint_id: ClientEndpointId,
        outcome: &mut ClientShellInput,
    ) -> bool {
        let online = self.endpoint_is_online(&endpoint_id);
        if !online && !endpoint_id.is_local() {
            let label = self.endpoint_label(&endpoint_id).to_owned();
            self.receive_endpoint_unavailable(format!("{label} is not ready"));
            outcome.repaint = true;
            return false;
        }
        if (endpoint_id.is_local() && (self.multi_endpoint_active() || !online))
            || endpoint_id != self.active_endpoint_id
        {
            outcome.actions.push(ClientShellAction::ActivateEndpoint {
                endpoint_id,
                target: None,
            });
        }
        true
    }

    pub(super) fn focus_or_activate(
        &mut self,
        endpoint_id: ClientEndpointId,
        target: ClientEndpointFocusTarget,
        outcome: &mut ClientShellInput,
    ) -> bool {
        let online = self.endpoint_is_online(&endpoint_id);
        if !online && !endpoint_id.is_local() {
            let label = self.endpoint_label(&endpoint_id).to_owned();
            self.receive_endpoint_unavailable(format!("{label} is not ready"));
            outcome.repaint = true;
            return false;
        }
        // Local can still be displayed while a remote activation is pending.
        // Route explicit selections through the runtime so they can cancel that handoff.
        if endpoint_id == self.active_endpoint_id
            && !(endpoint_id.is_local() && (self.multi_endpoint_active() || !online))
        {
            let method = match target {
                ClientEndpointFocusTarget::Workspace(workspace_id) => {
                    crate::api::schema::Method::WorkspaceFocus(
                        crate::api::schema::WorkspaceTarget { workspace_id },
                    )
                }
                ClientEndpointFocusTarget::Tab(tab_id) => {
                    crate::api::schema::Method::TabFocus(crate::api::schema::TabTarget { tab_id })
                }
                ClientEndpointFocusTarget::Pane(pane_id) => {
                    crate::api::schema::Method::PaneFocus(crate::api::schema::PaneTarget {
                        pane_id,
                    })
                }
            };
            self.push_endpoint_method(method, outcome);
        } else {
            outcome.actions.push(ClientShellAction::ActivateEndpoint {
                endpoint_id,
                target: Some(target),
            });
        }
        true
    }
}
