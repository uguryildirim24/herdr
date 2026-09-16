use std::collections::{HashMap, HashSet};

use super::ClientShellSnapshot;

pub(super) struct AgentTreeRow {
    pub(super) pane_id: String,
    /// 0 = top level; true depth, no clamp here
    pub(super) depth: usize,
    /// direct children in the model, hidden or not
    pub(super) child_count: usize,
    /// true only when child_count > 0 and the key is in collapsed_groups
    pub(super) collapsed: bool,
    /// all descendants removed because this row is collapsed; 0 otherwise
    pub(super) hidden_descendants: usize,
    /// Some only when hidden_descendants > 0
    pub(super) worst_hidden_status: Option<crate::api::schema::AgentStatus>,
}

/// Key stored in `collapsed_groups`: `agent:<pane_id>`, or `agent:<machine>:<pane_id>` when `machine` is Some.
pub(super) fn agent_group_key(machine: Option<&str>, pane_id: &str) -> String {
    machine
        .map(|machine| format!("agent:{machine}:{pane_id}"))
        .unwrap_or_else(|| format!("agent:{pane_id}"))
}

/// blocked > working > done > idle > unknown.
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

pub(super) fn nest_agents(
    ordered: &[String],
    snapshot: &ClientShellSnapshot,
    collapsed_groups: &HashSet<String>,
    machine: Option<&str>,
) -> Vec<AgentTreeRow> {
    // One pass over the snapshot keeps this O(n); a scan per ordered pane id would be
    // quadratic in the number of agents on every frame.
    let agents_by_pane_id = snapshot
        .agents
        .iter()
        .map(|agent| (agent.pane_id.as_str(), agent))
        .collect::<HashMap<_, _>>();
    let mut ordered_agents =
        Vec::<Option<&crate::protocol::ClientShellAgent>>::with_capacity(ordered.len());
    let mut order_to_index = HashMap::with_capacity(ordered.len());

    for (index, pane_id) in ordered.iter().enumerate() {
        if let Some(agent) = agents_by_pane_id.get(pane_id.as_str()).copied() {
            order_to_index.insert(pane_id.as_str(), index);
            ordered_agents.push(Some(agent));
        } else {
            ordered_agents.push(None);
        }
    }

    let mut parent_of = vec![None; ordered.len()];
    for (index, pane_id) in ordered.iter().enumerate() {
        let Some(agent) = ordered_agents[index] else {
            continue;
        };
        let Some(parent) = agent
            .tokens
            .iter()
            .rev()
            .find_map(|(key, value)| (key == "parent").then_some(value.as_str()))
        else {
            continue;
        };
        if parent == pane_id {
            continue;
        }
        let Some(parent_index) = order_to_index.get(parent).copied() else {
            continue;
        };
        let Some(parent_agent) = ordered_agents[parent_index] else {
            continue;
        };
        if parent_agent.workspace_id != agent.workspace_id {
            continue;
        }
        parent_of[index] = Some(parent_index);
    }

    let cycle = detect_cycles(&parent_of, &ordered_agents);
    let mut children = vec![Vec::new(); ordered.len()];
    for (index, maybe_parent) in parent_of.iter().copied().enumerate() {
        let Some(parent) = maybe_parent else {
            continue;
        };
        if cycle[index] {
            continue;
        }
        children[parent].push(index);
    }

    let statuses: Vec<Option<crate::api::schema::AgentStatus>> = ordered_agents
        .iter()
        .map(|agent| agent.map(|agent| agent.agent_status))
        .collect();

    let mut subtree_counts: Vec<Option<usize>> = vec![None; ordered.len()];
    let mut subtree_statuses: Vec<Option<crate::api::schema::AgentStatus>> =
        vec![None; ordered.len()];
    for index in 0..ordered.len() {
        aggregate_descendants(
            index,
            &statuses,
            &children,
            &mut subtree_counts,
            &mut subtree_statuses,
        );
    }

    let mut rows = Vec::new();
    for index in 0..ordered.len() {
        let Some(_) = ordered_agents[index] else {
            continue;
        };
        // Cycle members are roots even though they carry a parent edge; every other node
        // with a parent is emitted by its parent's subtree walk.
        if parent_of[index].is_some() && !cycle[index] {
            continue;
        }
        emit_tree_rows(
            index,
            0,
            ordered,
            &children,
            collapsed_groups,
            machine,
            &subtree_counts,
            &subtree_statuses,
            &mut rows,
        );
    }

    rows
}

/// For each visible row, whether it is the last child of its parent among the visible rows.
/// Top-level rows are never a last child. Shared by the expanded and collapsed sidebars so
/// both draw the same tree marks.
pub(super) fn last_child_flags(rows: &[AgentTreeRow]) -> Vec<bool> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            row.depth > 0
                && rows[index + 1..]
                    .iter()
                    .find(|next| next.depth <= row.depth)
                    .is_none_or(|next| next.depth < row.depth)
        })
        .collect()
}

fn detect_cycles(
    parent_of: &[Option<usize>],
    ordered_agents: &[Option<&crate::protocol::ClientShellAgent>],
) -> Vec<bool> {
    let mut state = vec![0u8; parent_of.len()];
    let mut cycle = vec![false; parent_of.len()];

    for start in 0..parent_of.len() {
        if ordered_agents[start].is_none() || state[start] != 0 {
            continue;
        }

        let mut cursor = start;
        let mut path = Vec::new();
        loop {
            if state[cursor] == 0 {
                state[cursor] = 1;
                path.push(cursor);
                let Some(parent) = parent_of[cursor] else {
                    break;
                };
                if ordered_agents[parent].is_none() {
                    break;
                }
                cursor = parent;
                continue;
            }
            if state[cursor] == 1 {
                if let Some(cycle_start) = path.iter().position(|&node| node == cursor) {
                    for node in &path[cycle_start..] {
                        cycle[*node] = true;
                    }
                }
            }
            break;
        }

        for node in path {
            state[node] = 2;
        }
    }

    cycle
}

fn aggregate_descendants(
    root: usize,
    statuses: &[Option<crate::api::schema::AgentStatus>],
    children: &[Vec<usize>],
    subtree_counts: &mut [Option<usize>],
    subtree_statuses: &mut [Option<crate::api::schema::AgentStatus>],
) {
    if subtree_counts[root].is_some() {
        return;
    }

    let Some(_) = statuses[root] else {
        subtree_counts[root] = Some(0);
        return;
    };

    let mut count = 0usize;
    let mut hidden = None;
    for &child in &children[root] {
        aggregate_descendants(child, statuses, children, subtree_counts, subtree_statuses);
        if let Some(child_status) = statuses[child] {
            count += 1;
            count += subtree_counts[child].unwrap_or(0);
            hidden = worse_status(hidden, Some(child_status));
            hidden = worse_status(hidden, subtree_statuses[child]);
        }
    }

    subtree_counts[root] = Some(count);
    subtree_statuses[root] = hidden;
}

fn worse_status(
    left: Option<crate::api::schema::AgentStatus>,
    right: Option<crate::api::schema::AgentStatus>,
) -> Option<crate::api::schema::AgentStatus> {
    match (left, right) {
        (None, status) => status,
        (Some(left), None) => Some(left),
        (Some(left), Some(right)) => {
            if status_severity(right) > status_severity(left) {
                Some(right)
            } else {
                Some(left)
            }
        }
    }
}

fn emit_tree_rows(
    index: usize,
    depth: usize,
    ordered: &[String],
    children: &[Vec<usize>],
    collapsed_groups: &HashSet<String>,
    machine: Option<&str>,
    subtree_counts: &[Option<usize>],
    subtree_statuses: &[Option<crate::api::schema::AgentStatus>],
    rows: &mut Vec<AgentTreeRow>,
) {
    let mut stack = Vec::new();
    stack.push((index, depth));

    while let Some((row_index, row_depth)) = stack.pop() {
        let child_count = children[row_index].len();
        let key = agent_group_key(machine, ordered[row_index].as_str());
        let collapsed = child_count > 0 && collapsed_groups.contains(&key);
        let hidden_descendants = if collapsed {
            subtree_counts[row_index].unwrap_or_default()
        } else {
            0
        };
        let worst_hidden_status = if collapsed {
            subtree_statuses[row_index].filter(|_| hidden_descendants > 0)
        } else {
            None
        };

        rows.push(AgentTreeRow {
            pane_id: ordered[row_index].clone(),
            depth: row_depth,
            child_count,
            collapsed,
            hidden_descendants,
            worst_hidden_status,
        });

        if collapsed {
            continue;
        }

        for &child in children[row_index].iter().rev() {
            stack.push((child, row_depth.saturating_add(1)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::schema::AgentStatus;
    use crate::protocol::ClientShellAgent;
    use std::collections::HashSet;

    fn ordered(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_string()).collect()
    }

    fn agent(
        pane_id: &str,
        workspace_id: &str,
        status: AgentStatus,
        tokens: &[(&str, &str)],
    ) -> ClientShellAgent {
        ClientShellAgent {
            pane_id: pane_id.into(),
            workspace_id: workspace_id.into(),
            tab_id: "tab".into(),
            name: None,
            display_agent: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: status,
            state_change_seq: 0,
            state_labels: Vec::new(),
            tokens: tokens
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
            focused: false,
        }
    }

    fn snapshot(agents: Vec<ClientShellAgent>) -> crate::protocol::ClientShellSnapshot {
        let mut snapshot = crate::client::shell::tests::snapshot();
        snapshot.agents = agents;
        snapshot
    }

    #[test]
    fn no_parent_tokens_keep_order_top_level() {
        let ordered = ordered(&["p1", "p2", "p3"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("p1", "ws_1", AgentStatus::Idle, &[]),
                agent("p2", "ws_1", AgentStatus::Working, &[]),
                agent("p3", "ws_1", AgentStatus::Blocked, &[]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].pane_id, "p1");
        assert_eq!(rows[1].pane_id, "p2");
        assert_eq!(rows[2].pane_id, "p3");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 0);
        assert_eq!(rows[2].depth, 0);
        assert_eq!(rows[0].child_count, 0);
        assert_eq!(rows[1].child_count, 0);
        assert_eq!(rows[2].child_count, 0);
    }

    #[test]
    fn parent_with_children_before_and_after_stays_adjacent() {
        let ordered = ordered(&["before", "parent", "after"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("before", "ws_1", AgentStatus::Idle, &[("parent", "parent")]),
                agent("parent", "ws_1", AgentStatus::Working, &[]),
                agent("after", "ws_1", AgentStatus::Done, &[("parent", "parent")]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].pane_id, "parent");
        assert_eq!(rows[1].pane_id, "before");
        assert_eq!(rows[2].pane_id, "after");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 1);
        assert_eq!(rows[0].child_count, 2);
    }

    #[test]
    fn grandchild_is_depth_first() {
        let ordered = ordered(&["parent", "child", "grandchild"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("parent", "ws_1", AgentStatus::Idle, &[]),
                agent(
                    "child",
                    "ws_1",
                    AgentStatus::Working,
                    &[("parent", "parent")],
                ),
                agent(
                    "grandchild",
                    "ws_1",
                    AgentStatus::Done,
                    &[("parent", "child")],
                ),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].pane_id, "parent");
        assert_eq!(rows[1].pane_id, "child");
        assert_eq!(rows[2].pane_id, "grandchild");
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 2);
    }

    #[test]
    fn parent_missing_from_order_makes_child_top_level() {
        let ordered = ordered(&["child", "parent"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("child", "ws_1", AgentStatus::Idle, &[("parent", "missing")]),
                agent("parent", "ws_1", AgentStatus::Working, &[]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pane_id, "child");
        assert_eq!(rows[1].pane_id, "parent");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 0);
    }

    #[test]
    fn parent_in_different_workspace_makes_child_top_level() {
        let ordered = ordered(&["p1", "p2"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("p1", "ws_1", AgentStatus::Idle, &[]),
                agent("p2", "ws_2", AgentStatus::Working, &[("parent", "p1")]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pane_id, "p1");
        assert_eq!(rows[1].pane_id, "p2");
        assert_eq!(rows[1].depth, 0);
    }

    #[test]
    fn self_parenting_is_top_level() {
        let ordered = ordered(&["p1", "p2"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("p1", "ws_1", AgentStatus::Idle, &[]),
                agent("p2", "ws_1", AgentStatus::Working, &[("parent", "p2")]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pane_id, "p1");
        assert_eq!(rows[1].pane_id, "p2");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 0);
    }

    #[test]
    fn two_node_cycle_is_flat() {
        let ordered = ordered(&["p1", "p2"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("p1", "ws_1", AgentStatus::Idle, &[("parent", "p2")]),
                agent("p2", "ws_1", AgentStatus::Done, &[("parent", "p1")]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pane_id, "p1");
        assert_eq!(rows[1].pane_id, "p2");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 0);
    }

    #[test]
    fn three_node_cycle_is_flat() {
        let ordered = ordered(&["p1", "p2", "p3"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("p1", "ws_1", AgentStatus::Working, &[("parent", "p3")]),
                agent("p2", "ws_1", AgentStatus::Blocked, &[("parent", "p1")]),
                agent("p3", "ws_1", AgentStatus::Idle, &[("parent", "p2")]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].pane_id, "p1");
        assert_eq!(rows[1].pane_id, "p2");
        assert_eq!(rows[2].pane_id, "p3");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 0);
        assert_eq!(rows[2].depth, 0);
    }

    #[test]
    fn child_of_cycle_member_still_nests_under_it() {
        // Spec 8 item 7: the cycle members are top level, and a node whose parent chain
        // reaches the cycle without being in it still nests under its parent.
        let ordered = ordered(&["cyc1", "cyc2", "child", "grand"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("cyc1", "ws_1", AgentStatus::Idle, &[("parent", "cyc2")]),
                agent("cyc2", "ws_1", AgentStatus::Idle, &[("parent", "cyc1")]),
                agent("child", "ws_1", AgentStatus::Working, &[("parent", "cyc1")]),
                agent(
                    "grand",
                    "ws_1",
                    AgentStatus::Blocked,
                    &[("parent", "child")],
                ),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(
            rows.iter()
                .map(|row| row.pane_id.as_str())
                .collect::<Vec<_>>(),
            vec!["cyc1", "child", "grand", "cyc2"]
        );
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[0].child_count, 1);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].depth, 2);
        assert_eq!(rows[3].depth, 0);
        assert_eq!(rows[3].child_count, 0);
    }

    #[test]
    fn collapsed_cycle_member_rolls_up_its_own_descendants() {
        let ordered = ordered(&["cyc1", "cyc2", "child"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("cyc1", "ws_1", AgentStatus::Idle, &[("parent", "cyc2")]),
                agent("cyc2", "ws_1", AgentStatus::Idle, &[("parent", "cyc1")]),
                agent("child", "ws_1", AgentStatus::Blocked, &[("parent", "cyc1")]),
            ]),
            &[agent_group_key(None, "cyc1")].into_iter().collect(),
            None,
        );

        assert_eq!(
            rows.iter()
                .map(|row| row.pane_id.as_str())
                .collect::<Vec<_>>(),
            vec!["cyc1", "cyc2"]
        );
        assert!(rows[0].collapsed);
        assert_eq!(rows[0].hidden_descendants, 1);
        assert_eq!(rows[0].worst_hidden_status, Some(AgentStatus::Blocked));
    }

    #[test]
    fn worst_hidden_status_spans_every_hidden_branch() {
        let ordered = ordered(&["parent", "a", "a_child", "b"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("parent", "ws_1", AgentStatus::Blocked, &[]),
                agent("a", "ws_1", AgentStatus::Idle, &[("parent", "parent")]),
                agent("a_child", "ws_1", AgentStatus::Working, &[("parent", "a")]),
                agent("b", "ws_1", AgentStatus::Done, &[("parent", "parent")]),
            ]),
            &[agent_group_key(None, "parent")].into_iter().collect(),
            None,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].child_count, 2);
        assert_eq!(rows[0].hidden_descendants, 3);
        // Working beats Done and Idle, and the parent's own Blocked status is not rolled up.
        assert_eq!(rows[0].worst_hidden_status, Some(AgentStatus::Working));
    }

    #[test]
    fn children_keep_relative_order_and_parent_keeps_its_index() {
        let ordered = ordered(&["first", "c2", "parent", "c1", "last"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("first", "ws_1", AgentStatus::Idle, &[]),
                agent("c2", "ws_1", AgentStatus::Idle, &[("parent", "parent")]),
                agent("parent", "ws_1", AgentStatus::Idle, &[]),
                agent("c1", "ws_1", AgentStatus::Idle, &[("parent", "parent")]),
                agent("last", "ws_1", AgentStatus::Idle, &[]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(
            rows.iter()
                .map(|row| row.pane_id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "parent", "c2", "c1", "last"]
        );
        assert_eq!(rows[2].depth, 1);
        assert_eq!(rows[3].depth, 1);
    }

    #[test]
    fn same_tab_is_not_required() {
        let ordered = ordered(&["parent", "child"]);
        let mut child = agent("child", "ws_1", AgentStatus::Idle, &[("parent", "parent")]);
        child.tab_id = "other_tab".into();
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![agent("parent", "ws_1", AgentStatus::Idle, &[]), child]),
            &HashSet::new(),
            None,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].pane_id, "child");
        assert_eq!(rows[1].depth, 1);
    }

    #[test]
    fn collapsed_parent_hides_all_levels_and_tracks_worst_status() {
        let ordered = ordered(&["parent", "child", "grand"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("parent", "ws_1", AgentStatus::Done, &[]),
                agent(
                    "child",
                    "ws_1",
                    AgentStatus::Working,
                    &[("parent", "parent")],
                ),
                agent(
                    "grand",
                    "ws_1",
                    AgentStatus::Blocked,
                    &[("parent", "child")],
                ),
            ]),
            &[agent_group_key(None, "parent")].into_iter().collect(),
            None,
        );

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pane_id, "parent");
        assert!(rows[0].collapsed);
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[0].child_count, 1);
        assert_eq!(rows[0].hidden_descendants, 2);
        assert_eq!(rows[0].worst_hidden_status, Some(AgentStatus::Blocked));
    }

    #[test]
    fn collapsed_grandchild_only_hides_subtree() {
        let ordered = ordered(&["parent", "child", "grand"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("parent", "ws_1", AgentStatus::Idle, &[]),
                agent(
                    "child",
                    "ws_1",
                    AgentStatus::Working,
                    &[("parent", "parent")],
                ),
                agent(
                    "grand",
                    "ws_1",
                    AgentStatus::Blocked,
                    &[("parent", "child")],
                ),
            ]),
            &[agent_group_key(None, "child")].into_iter().collect(),
            None,
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pane_id, "parent");
        assert_eq!(rows[1].pane_id, "child");
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
        assert!(rows[1].collapsed);
        assert_eq!(rows[1].child_count, 1);
        assert_eq!(rows[1].hidden_descendants, 1);
        assert_eq!(rows[1].worst_hidden_status, Some(AgentStatus::Blocked));
    }

    #[test]
    fn last_child_flags_follow_siblings_at_each_depth() {
        let ordered = ordered(&["p1", "c1", "g1", "c2", "top"]);
        let rows = nest_agents(
            &ordered,
            &snapshot(vec![
                agent("p1", "ws_1", AgentStatus::Idle, &[]),
                agent("c1", "ws_1", AgentStatus::Idle, &[("parent", "p1")]),
                agent("g1", "ws_1", AgentStatus::Idle, &[("parent", "c1")]),
                agent("c2", "ws_1", AgentStatus::Idle, &[("parent", "p1")]),
                agent("top", "ws_1", AgentStatus::Idle, &[]),
            ]),
            &HashSet::new(),
            None,
        );

        assert_eq!(
            last_child_flags(&rows),
            vec![false, false, true, true, false]
        );
        assert!(last_child_flags(&[]).is_empty());
    }

    #[test]
    fn agent_group_key_uses_machine_when_present() {
        assert_eq!(agent_group_key(None, "p1"), "agent:p1");
        assert_eq!(agent_group_key(Some("m1"), "p1"), "agent:m1:p1");
    }

    #[test]
    fn status_severity_orders_blocked_working_done_idle_unknown() {
        assert!(status_severity(AgentStatus::Blocked) > status_severity(AgentStatus::Working));
        assert!(status_severity(AgentStatus::Working) > status_severity(AgentStatus::Done));
        assert!(status_severity(AgentStatus::Done) > status_severity(AgentStatus::Idle));
        assert!(status_severity(AgentStatus::Idle) > status_severity(AgentStatus::Unknown));
    }

    #[test]
    fn config_default_false_and_parses_true() {
        assert!(
            !crate::config::Config::default()
                .experimental
                .agent_parent_nesting
        );
        let config: crate::config::Config =
            toml::from_str("[experimental]\nagent_parent_nesting = true\n").unwrap();
        assert!(config.experimental.agent_parent_nesting);
    }
}
