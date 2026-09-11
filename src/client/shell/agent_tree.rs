#![allow(dead_code)] // Consumed by lane w2 in parallel render work.

use std::collections::HashSet;

use super::ClientShellSnapshot;

pub(super) struct AgentTreeRow {
    pub(super) pane_id: String,
    pub(super) depth: usize,
    pub(super) child_count: usize,
    pub(super) collapsed: bool,
    pub(super) hidden_descendants: usize,
    pub(super) worst_hidden_status: Option<crate::api::schema::AgentStatus>,
}

/// Key stored in `collapsed_groups`: "agent:<pane_id>", or "agent:<machine>:<pane_id>" when `machine` is Some.
pub(super) fn agent_group_key(machine: Option<&str>, pane_id: &str) -> String {
    machine
        .map(|machine| format!("agent:{machine}:{pane_id}"))
        .unwrap_or_else(|| format!("agent:{pane_id}"))
}

/// Returns the VISIBLE rows in render order. `ordered` is exactly the output of
/// `ordered_agent_pane_ids` (already sorted/filtered). Rows under a collapsed parent are removed.
pub(super) fn nest_agents(
    ordered: &[String],
    _snapshot: &ClientShellSnapshot,
    _collapsed_groups: &HashSet<String>,
    _machine: Option<&str>,
) -> Vec<AgentTreeRow> {
    ordered
        .iter()
        .map(|pane_id| AgentTreeRow {
            pane_id: pane_id.clone(),
            depth: 0,
            child_count: 0,
            collapsed: false,
            hidden_descendants: 0,
            worst_hidden_status: None,
        })
        .collect()
}

/// blocked > working > done > idle > unknown (higher = worse). Used for the roll-up.
pub(super) fn status_severity(status: crate::api::schema::AgentStatus) -> u8 {
    use crate::api::schema::AgentStatus;
    match status {
        AgentStatus::Blocked => 4,
        AgentStatus::Working => 3,
        AgentStatus::Done => 2,
        AgentStatus::Idle => 1,
        AgentStatus::Unknown => 0,
    }
}
