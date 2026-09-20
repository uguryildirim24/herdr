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

/// Tabs of every part of the active space, active endpoint first. When the
/// focused workspace has no custom name the active endpoint's own tabs are all
/// that can link, so the strip is unchanged.
pub(super) fn linked_tabs<'a>(
    endpoints: &'a [ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
) -> Vec<LinkedTab<'a>> {
    let Some(active_endpoint) = endpoints
        .iter()
        .find(|endpoint| &endpoint.endpoint_id == active_endpoint_id)
    else {
        return Vec::new();
    };
    let Some(active_snapshot) = active_endpoint.snapshot.as_deref() else {
        return Vec::new();
    };
    let Some(focused_id) = active_snapshot.focused_workspace_id.as_deref() else {
        return Vec::new();
    };
    let Some(active_workspace) = active_snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == focused_id)
    else {
        return Vec::new();
    };
    let mut parts = vec![(active_endpoint, active_workspace)];
    if active_workspace.custom_label {
        for endpoint in endpoints {
            if &endpoint.endpoint_id == active_endpoint_id {
                continue;
            }
            let Some(snapshot) = endpoint.snapshot.as_deref() else {
                continue;
            };
            if let Some(workspace) = snapshot.workspaces.iter().find(|workspace| {
                workspace.custom_label && workspace.label == active_workspace.label
            }) {
                parts.push((endpoint, workspace));
            }
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
    let mut index_by_name = HashMap::<String, usize>::new();
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
            let existing = linkable
                .then(|| index_by_name.get(&workspace.label).copied())
                .flatten();
            if let Some(index) = existing {
                spaces[index].parts.push(part);
            } else {
                if linkable {
                    index_by_name.insert(workspace.label.clone(), spaces.len());
                }
                spaces.push(LinkedSpace { parts: vec![part] });
            }
        }
    }
    spaces
}
