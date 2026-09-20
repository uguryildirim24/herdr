//! Merged "linked space" rows for the multi-machine sidebar.
//!
//! A workspace that carries a custom name is a space identity: the same name on
//! two machines is one project seen through two endpoints. The sidebar draws it
//! once, rolls its status up from the worst part, and marks which machines hold
//! it. Worktree children keep the indentation of the part the row is drawn from.

use super::*;

/// One endpoint's part of a linked space.
pub(super) struct LinkedSpacePart<'a> {
    pub(super) endpoint_id: &'a ClientEndpointId,
    pub(super) endpoint_label: &'a str,
    pub(super) online: bool,
    pub(super) workspace_index: usize,
    pub(super) workspace: &'a ClientShellWorkspace,
    pub(super) snapshot: &'a ClientShellSnapshot,
    pub(super) entry: WorkspaceEntry,
    pub(super) status: crate::api::schema::AgentStatus,
}

/// A merged row: one name, one part per endpoint that has it.
pub(super) struct LinkedSpace<'a> {
    pub(super) parts: Vec<LinkedSpacePart<'a>>,
}

impl LinkedSpace<'_> {
    /// The worst status of the parts, so one busy or blocked part marks the row.
    pub(super) fn worst_status(&self) -> crate::api::schema::AgentStatus {
        self.parts
            .iter()
            .map(|part| part.status)
            .max_by_key(|status| status_priority(*status))
            .unwrap_or(crate::api::schema::AgentStatus::Unknown)
    }

    /// The part a selection acts on: the active endpoint's part when it has one,
    /// else the first online part, else the first part.
    pub(super) fn primary_part_index(&self, active_endpoint_id: &ClientEndpointId) -> usize {
        self.parts
            .iter()
            .position(|part| part.endpoint_id == active_endpoint_id)
            .or_else(|| self.parts.iter().position(|part| part.online))
            .unwrap_or(0)
    }

    /// Label initials of every machine that holds the space, drawn once per
    /// machine when the space spans more than one.
    pub(super) fn machine_marks(&self) -> String {
        if self.parts.len() <= 1 {
            return String::new();
        }
        self.parts
            .iter()
            .filter_map(|part| part.endpoint_label.chars().next())
            .collect()
    }
}

fn collapsed_groups_for<'a>(
    endpoint_id: &ClientEndpointId,
    local: &'a HashSet<String>,
    remote: &'a HashMap<ClientEndpointId, HashSet<String>>,
    empty: &'a HashSet<String>,
) -> &'a HashSet<String> {
    if endpoint_id.is_local() {
        local
    } else {
        remote.get(endpoint_id).unwrap_or(empty)
    }
}

/// Workspaces hidden while their machine has another space: the machine's
/// default home shell with no name, no agent, and only its one shell tab.
pub(super) fn hidden_default_workspaces(snapshot: &ClientShellSnapshot) -> HashSet<usize> {
    if snapshot.workspaces.len() <= 1 {
        return HashSet::new();
    }
    snapshot
        .workspaces
        .iter()
        .enumerate()
        .filter(|(_, workspace)| is_default_shell_space(snapshot, workspace))
        .map(|(index, _)| index)
        .collect()
}

fn is_default_shell_space(
    snapshot: &ClientShellSnapshot,
    workspace: &ClientShellWorkspace,
) -> bool {
    if workspace.custom_label || workspace.label != "~" {
        return false;
    }
    if snapshot
        .agents
        .iter()
        .any(|agent| agent.workspace_id == workspace.workspace_id)
    {
        return false;
    }
    let tabs = snapshot
        .tabs
        .iter()
        .filter(|tab| tab.workspace_id == workspace.workspace_id)
        .collect::<Vec<_>>();
    if tabs.len() != 1 {
        return false;
    }
    snapshot
        .panes
        .iter()
        .filter(|pane| pane.tab_id == tabs[0].tab_id)
        .count()
        == 1
}

/// One tab of a linked space's merged tab strip.
pub(super) struct LinkedTab<'a> {
    pub(super) tab: &'a ClientShellTab,
    pub(super) endpoint_id: &'a ClientEndpointId,
    /// Machine initial when the tab lives on another endpoint.
    pub(super) machine_mark: Option<char>,
    pub(super) active_endpoint: bool,
}

/// The active endpoint's focused workspace, the plain case for the tab row.
fn focused_workspace<'a>(
    endpoints: &'a [ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
) -> Option<(&'a ClientShellEndpoint, &'a ClientShellWorkspace)> {
    let endpoint = endpoints
        .iter()
        .find(|endpoint| &endpoint.endpoint_id == active_endpoint_id)?;
    let snapshot = endpoint.snapshot.as_deref()?;
    let focused_id = snapshot.focused_workspace_id.as_deref()?;
    let workspace = snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == focused_id)?;
    Some((endpoint, workspace))
}

/// Resolve the endpoint a `parent` token's machine label names. A bare token stays on
/// `own`; an ambiguous label resolves to nothing, the way the agent tree refuses to guess.
fn parent_endpoint<'a>(
    endpoints: &'a [ClientShellEndpoint],
    own: &'a ClientShellEndpoint,
    label: Option<&str>,
) -> Option<&'a ClientShellEndpoint> {
    match label {
        None => Some(own),
        Some(label) => {
            let mut matches = endpoints.iter().filter(|endpoint| endpoint.label == label);
            let first = matches.next()?;
            matches.next().is_none().then_some(first)
        }
    }
}

/// The workspace whose tab row the bar draws. Normally that is the focused workspace; a lane
/// hangs under a coordinator, so entering one keeps its coordinator's row and the next/previous
/// tab walk can cross the whole row instead of stranding on the lane.
fn tab_anchor<'a>(
    endpoints: &'a [ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
) -> Option<(&'a ClientShellEndpoint, &'a ClientShellWorkspace)> {
    let (endpoint, focused) = focused_workspace(endpoints, active_endpoint_id)?;
    let snapshot = endpoint.snapshot.as_deref()?;
    for agent in snapshot
        .agents
        .iter()
        .filter(|agent| agent.workspace_id == focused.workspace_id)
    {
        let Some(token) = super::agent_tree::parent_token(agent) else {
            continue;
        };
        let Some((label, pane_id)) = super::aggregate_navigation::split_machine_parent(token)
        else {
            continue;
        };
        let Some(parent_endpoint) = parent_endpoint(endpoints, endpoint, label) else {
            continue;
        };
        let Some(parent_snapshot) = parent_endpoint.snapshot.as_deref() else {
            continue;
        };
        let Some(parent_workspace) = parent_snapshot.workspaces.iter().find(|workspace| {
            parent_snapshot
                .panes
                .iter()
                .any(|pane| pane.workspace_id == workspace.workspace_id && pane.pane_id == pane_id)
        }) else {
            continue;
        };
        // A parent in the same workspace is not a lane, so the focused workspace stays the row.
        if parent_endpoint.endpoint_id == endpoint.endpoint_id
            && parent_workspace.workspace_id == focused.workspace_id
        {
            continue;
        }
        return Some((parent_endpoint, parent_workspace));
    }
    Some((endpoint, focused))
}

/// Workspaces whose agents hang under a pane of the anchor workspace, on any endpoint. A lane
/// runs in its own workspace, often on another machine, and its tab belongs in its coordinator's
/// row even though the workspace name differs.
fn lane_spaces<'a>(
    endpoints: &'a [ClientShellEndpoint],
    anchor_endpoint: &'a ClientShellEndpoint,
    anchor_workspace: &'a ClientShellWorkspace,
) -> Vec<(&'a ClientShellEndpoint, &'a ClientShellWorkspace)> {
    let Some(anchor_snapshot) = anchor_endpoint.snapshot.as_deref() else {
        return Vec::new();
    };
    let anchor_panes = anchor_snapshot
        .panes
        .iter()
        .filter(|pane| pane.workspace_id == anchor_workspace.workspace_id)
        .map(|pane| pane.pane_id.as_str())
        .collect::<HashSet<_>>();
    if anchor_panes.is_empty() {
        return Vec::new();
    }
    let mut lanes = Vec::new();
    for endpoint in endpoints {
        let Some(snapshot) = endpoint.snapshot.as_deref() else {
            continue;
        };
        for workspace in &snapshot.workspaces {
            if endpoint.endpoint_id == anchor_endpoint.endpoint_id
                && workspace.workspace_id == anchor_workspace.workspace_id
            {
                continue;
            }
            let has_lane = snapshot.agents.iter().any(|agent| {
                if agent.workspace_id != workspace.workspace_id {
                    return false;
                }
                let Some(token) = super::agent_tree::parent_token(agent) else {
                    return false;
                };
                let Some((label, pane_id)) =
                    super::aggregate_navigation::split_machine_parent(token)
                else {
                    return false;
                };
                let Some(parent_endpoint) = parent_endpoint(endpoints, endpoint, label) else {
                    return false;
                };
                parent_endpoint.endpoint_id == anchor_endpoint.endpoint_id
                    && anchor_panes.contains(pane_id)
            });
            if has_lane {
                lanes.push((endpoint, workspace));
            }
        }
    }
    lanes
}

/// Tabs of the anchor space, its same-named parts on other machines, and its lanes.
/// `parent_nesting` is the experimental gate the agent panel uses; with it off the strip is the
/// focused workspace's own tabs exactly as before.
pub(super) fn linked_tabs<'a>(
    endpoints: &'a [ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    parent_nesting: bool,
) -> Vec<LinkedTab<'a>> {
    let anchor = if parent_nesting {
        tab_anchor(endpoints, active_endpoint_id)
    } else {
        focused_workspace(endpoints, active_endpoint_id)
    };
    let Some((anchor_endpoint, anchor_workspace)) = anchor else {
        return Vec::new();
    };
    let Some(anchor_snapshot) = anchor_endpoint.snapshot.as_deref() else {
        return Vec::new();
    };
    let mut parts = vec![(anchor_endpoint, anchor_workspace)];
    if anchor_workspace.custom_label {
        let occurrence = anchor_snapshot
            .workspaces
            .iter()
            .filter(|workspace| workspace.custom_label && workspace.label == anchor_workspace.label)
            .position(|workspace| workspace.workspace_id == anchor_workspace.workspace_id)
            .unwrap_or(0);
        for endpoint in endpoints {
            if endpoint.endpoint_id == anchor_endpoint.endpoint_id {
                continue;
            }
            let Some(snapshot) = endpoint.snapshot.as_deref() else {
                continue;
            };
            if let Some(workspace) = snapshot
                .workspaces
                .iter()
                .filter(|workspace| {
                    workspace.custom_label && workspace.label == anchor_workspace.label
                })
                .nth(occurrence)
            {
                parts.push((endpoint, workspace));
            }
        }
    }
    if parent_nesting {
        for (endpoint, workspace) in lane_spaces(endpoints, anchor_endpoint, anchor_workspace) {
            if parts.iter().any(|(part_endpoint, part_workspace)| {
                part_endpoint.endpoint_id == endpoint.endpoint_id
                    && part_workspace.workspace_id == workspace.workspace_id
            }) {
                continue;
            }
            parts.push((endpoint, workspace));
        }
    }
    let mut tabs = Vec::new();
    for (endpoint, workspace) in parts {
        let Some(snapshot) = endpoint.snapshot.as_deref() else {
            continue;
        };
        let active_endpoint = &endpoint.endpoint_id == active_endpoint_id;
        let machine_mark = (!active_endpoint)
            .then(|| endpoint.label.chars().next())
            .flatten();
        for tab in snapshot
            .tabs
            .iter()
            .filter(|tab| tab.workspace_id == workspace.workspace_id)
        {
            tabs.push(LinkedTab {
                tab,
                endpoint_id: &endpoint.endpoint_id,
                machine_mark,
                active_endpoint,
            });
        }
    }
    tabs
}

/// Build the merged list. Endpoint order sets row order; a name already seen is
/// appended as another part instead of a second row. A custom name is the space
/// identity, so auto-labelled workspaces never merge across machines.
pub(super) fn linked_spaces<'a>(
    endpoints: &'a [ClientShellEndpoint],
    collapsed_endpoints: &HashSet<ClientEndpointId>,
    local_collapsed_groups: &HashSet<String>,
    remote_collapsed_groups: &HashMap<ClientEndpointId, HashSet<String>>,
) -> Vec<LinkedSpace<'a>> {
    let empty = HashSet::new();
    let mut spaces = Vec::<LinkedSpace<'a>>::new();
    let mut indices_by_name = HashMap::<String, Vec<usize>>::new();
    for endpoint in endpoints.iter() {
        if collapsed_endpoints.contains(&endpoint.endpoint_id) {
            continue;
        }
        let Some(snapshot) = endpoint.snapshot.as_deref() else {
            continue;
        };
        let collapsed = collapsed_groups_for(
            &endpoint.endpoint_id,
            local_collapsed_groups,
            remote_collapsed_groups,
            &empty,
        );
        let hidden = hidden_default_workspaces(snapshot);
        let online = endpoint.status == ClientEndpointStatus::Online;
        for entry in super::sidebar::workspace_entries(snapshot, collapsed) {
            if hidden.contains(&entry.index) {
                continue;
            }
            let workspace = &snapshot.workspaces[entry.index];
            let status = super::sidebar::displayed_workspace_status(snapshot, workspace, collapsed);
            let part = LinkedSpacePart {
                endpoint_id: &endpoint.endpoint_id,
                endpoint_label: &endpoint.label,
                online,
                workspace_index: entry.index,
                workspace,
                snapshot,
                entry,
                status,
            };
            let linkable = workspace.custom_label;
            let existing = linkable.then(|| {
                indices_by_name.get(&workspace.label).and_then(|indices| {
                    indices.iter().copied().find(|index| {
                        spaces[*index]
                            .parts
                            .iter()
                            .all(|candidate| candidate.endpoint_id != &endpoint.endpoint_id)
                    })
                })
            });
            if let Some(index) = existing.flatten() {
                spaces[index].parts.push(part);
            } else {
                let index = spaces.len();
                spaces.push(LinkedSpace { parts: vec![part] });
                if linkable {
                    indices_by_name
                        .entry(workspace.label.clone())
                        .or_default()
                        .push(index);
                }
            }
        }
    }
    spaces
}
