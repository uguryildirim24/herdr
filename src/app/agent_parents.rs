//! Server-side notifications from nested agents to their parent agent.
//!
//! A child agent names its parent with the `parent` pane metadata token. With
//! `[experimental] agent_parent_notify` on, the server prompts the parent with
//! `BLOCKED <child>` once per transition into blocked, and with `GONE <child>`
//! when the child's agent exits or its pane, tab, or workspace closes. Closes
//! remove the child's terminal before their events are emitted, so live links
//! are indexed by terminal id ahead of time.

use crate::api::schema::{AgentPromptParams, AgentStatus, EventData, EventEnvelope};
use crate::terminal::TerminalId;

use super::App;

pub(crate) const PARENT_TOKEN: &str = "parent";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentParentLink {
    /// Public pane id of the parent agent.
    parent: String,
    /// Child agent name, or its public pane id when unnamed.
    label: String,
    blocked_notified: bool,
}

impl App {
    pub(super) fn notify_agent_parents(&mut self, event: &EventEnvelope) {
        if !self.agent_parent_notify {
            return;
        }
        match &event.data {
            EventData::PaneAgentStatusChanged {
                pane_id,
                agent_status,
                ..
            } => {
                let Some(terminal_id) = self.refresh_agent_parent_link(pane_id) else {
                    return;
                };
                self.notify_parent_of_status(&terminal_id, *agent_status);
            }
            EventData::PaneAgentDetected {
                pane_id,
                released: false,
                ..
            } => {
                self.refresh_agent_parent_link(pane_id);
            }
            EventData::PaneUpdated { pane } => {
                self.refresh_agent_parent_link(&pane.pane_id);
            }
            EventData::PaneAgentDetected { released: true, .. }
            | EventData::PaneExited { .. }
            | EventData::PaneClosed { .. }
            | EventData::TabClosed { .. }
            | EventData::WorkspaceClosed { .. } => self.notify_parents_of_gone_children(),
            _ => {}
        }
    }

    /// Re-reads the link for one pane and returns its terminal id when the
    /// pane holds a live child agent with a parent.
    fn refresh_agent_parent_link(&mut self, public_pane_id: &str) -> Option<TerminalId> {
        let (ws_idx, pane_id) = self.parse_pane_id(public_pane_id)?;
        let terminal_id = self
            .state
            .workspaces
            .get(ws_idx)?
            .terminal_id(pane_id)?
            .clone();
        let terminal = self.state.terminals.get(&terminal_id)?;
        let parent = terminal
            .metadata_tokens
            .get(PARENT_TOKEN)
            .filter(|parent| *parent != public_pane_id)
            .map(str::to_string);
        let (Some(parent), Some(_)) = (parent, terminal.effective_known_agent()) else {
            self.agent_parent_links.remove(&terminal_id);
            return None;
        };
        let label = terminal
            .agent_name
            .clone()
            .unwrap_or_else(|| public_pane_id.to_string());
        let blocked_notified = self
            .agent_parent_links
            .get(&terminal_id)
            .is_some_and(|link| link.blocked_notified);
        self.agent_parent_links.insert(
            terminal_id.clone(),
            AgentParentLink {
                parent,
                label,
                blocked_notified,
            },
        );
        Some(terminal_id)
    }

    fn notify_parent_of_status(&mut self, terminal_id: &TerminalId, status: AgentStatus) {
        let Some(link) = self.agent_parent_links.get_mut(terminal_id) else {
            return;
        };
        let blocked = status == AgentStatus::Blocked;
        if blocked == link.blocked_notified {
            return;
        }
        link.blocked_notified = blocked;
        if blocked {
            let (parent, text) = (link.parent.clone(), format!("BLOCKED {}", link.label));
            self.prompt_agent_parent(parent, text);
        }
    }

    fn notify_parents_of_gone_children(&mut self) {
        if self.agent_parent_links.is_empty() {
            return;
        }
        let gone: Vec<TerminalId> = self
            .agent_parent_links
            .keys()
            .filter(|terminal_id| {
                self.state
                    .terminals
                    .get(*terminal_id)
                    .is_none_or(|terminal| terminal.effective_known_agent().is_none())
            })
            .cloned()
            .collect();
        for terminal_id in gone {
            if let Some(link) = self.agent_parent_links.remove(&terminal_id) {
                self.prompt_agent_parent(link.parent, format!("GONE {}", link.label));
            }
        }
    }

    fn prompt_agent_parent(&mut self, parent: String, text: String) {
        let params = AgentPromptParams {
            target: parent.clone(),
            text: text.clone(),
            wait: None,
        };
        // A parent that is blocked, gone, or not an agent misses the notice;
        // nothing waits on the submission.
        if let Err(response) = self.queue_agent_prompt("agent-parent-notify".into(), params) {
            tracing::debug!(%parent, %text, %response, "agent parent notification not delivered");
        }
    }

    /// Rebuild parent links after cold restore or live handoff. Terminal ids
    /// change, so the in-memory map cannot be reused. A child that stays
    /// blocked across the restore does not re-fire BLOCKED.
    pub(crate) fn reindex_after_restore(&mut self) {
        self.agent_parent_links.clear();
        let mut pane_ids = Vec::new();
        for (ws_idx, workspace) in self.state.workspaces.iter().enumerate() {
            for tab in &workspace.tabs {
                for pane_id in tab.layout.pane_ids() {
                    if let Some(public_id) = self.public_pane_id(ws_idx, pane_id) {
                        pane_ids.push(public_id);
                    }
                }
            }
        }
        for public_id in pane_ids {
            self.index_restored_agent_parent_link(&public_id);
        }
    }

    fn index_restored_agent_parent_link(&mut self, public_pane_id: &str) {
        let Some(terminal_id) = self.refresh_agent_parent_link(public_pane_id) else {
            return;
        };
        let blocked = self
            .state
            .terminals
            .get(&terminal_id)
            .is_some_and(|terminal| terminal.state == crate::detect::AgentState::Blocked);
        if let Some(link) = self.agent_parent_links.get_mut(&terminal_id) {
            link.blocked_notified = blocked;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use bytes::Bytes;

    use super::*;
    use crate::{
        app::Mode,
        config::Config,
        detect::{Agent, AgentState},
        workspace::Workspace,
    };

    struct Harness {
        app: App,
        parent_rx: tokio::sync::mpsc::Receiver<Bytes>,
        child_pane: crate::layout::PaneId,
        child_public: String,
    }

    fn harness(notify: bool, parent_token: bool) -> Harness {
        let mut config = Config::default();
        config.experimental.agent_parent_notify = notify;
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(
            &config,
            crate::app::AppPolicy::TEST,
            None,
            api_rx,
            crate::api::EventHub::default(),
        );
        app.state.workspaces = vec![Workspace::test_new("coord"), Workspace::test_new("lane")];
        app.state.ensure_test_terminals();
        app.state.active = Some(0);
        app.state.selected = 0;
        app.state.mode = Mode::Terminal;

        let parent_pane = app.state.workspaces[0].tabs[0].root_pane;
        let parent_public = app.public_pane_id(0, parent_pane).unwrap();
        let parent = terminal_mut(&mut app, 0, parent_pane);
        parent.set_agent_name("coordinator".into());
        parent.set_detected_state(Some(Agent::Claude), AgentState::Idle);
        let (runtime, parent_rx) = crate::terminal::TerminalRuntime::test_with_channel(80, 24);
        app.state.insert_test_runtime(parent_pane, runtime);

        let child_pane = app.state.workspaces[1].tabs[0].root_pane;
        let child_public = app.public_pane_id(1, child_pane).unwrap();
        let child = terminal_mut(&mut app, 1, child_pane);
        child.set_agent_name("w1".into());
        if parent_token {
            child.metadata_tokens.patch(
                std::collections::HashMap::from([(PARENT_TOKEN.to_string(), Some(parent_public))]),
                None,
                Instant::now(),
            );
        }
        Harness {
            app,
            parent_rx,
            child_pane,
            child_public,
        }
    }

    fn terminal_mut(
        app: &mut App,
        ws_idx: usize,
        pane_id: crate::layout::PaneId,
    ) -> &mut crate::terminal::TerminalState {
        let terminal_id = app.state.workspaces[ws_idx]
            .terminal_id(pane_id)
            .unwrap()
            .clone();
        app.state.terminals.get_mut(&terminal_id).unwrap()
    }

    impl Harness {
        fn child_status(&mut self, state: AgentState, status: AgentStatus) {
            terminal_mut(&mut self.app, 1, self.child_pane)
                .set_detected_state(Some(Agent::Claude), state);
            let workspace_id = self.app.public_workspace_id(1);
            self.app.emit_event(EventEnvelope {
                event: crate::api::schema::EventKind::PaneAgentStatusChanged,
                data: EventData::PaneAgentStatusChanged {
                    pane_id: self.child_public.clone(),
                    workspace_id,
                    agent_status: status,
                    agent: Some("claude".into()),
                    title: None,
                    display_agent: None,
                    state_labels: std::collections::HashMap::new(),
                },
            });
        }

        /// Collects what was typed into the parent until it goes quiet.
        fn parent_input(&mut self) -> String {
            let mut typed = Vec::new();
            let mut quiet_since = Instant::now();
            while quiet_since.elapsed() < Duration::from_millis(600) {
                match self.parent_rx.try_recv() {
                    Ok(bytes) => {
                        typed.extend_from_slice(&bytes);
                        quiet_since = Instant::now();
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(20)),
                }
            }
            String::from_utf8(typed).unwrap()
        }
    }

    #[tokio::test]
    async fn blocked_child_prompts_parent_once_per_transition() {
        let mut h = harness(true, true);

        h.child_status(AgentState::Blocked, AgentStatus::Blocked);
        assert_eq!(h.parent_input(), "BLOCKED w1\r");

        // A presentation-only change while still blocked does not repeat it.
        h.child_status(AgentState::Blocked, AgentStatus::Blocked);
        h.child_status(AgentState::Working, AgentStatus::Working);
        assert_eq!(h.parent_input(), "");

        h.child_status(AgentState::Blocked, AgentStatus::Blocked);
        assert_eq!(h.parent_input(), "BLOCKED w1\r");
    }

    #[tokio::test]
    async fn closing_child_workspace_prompts_parent_gone() {
        let mut h = harness(true, true);
        h.child_status(AgentState::Working, AgentStatus::Working);
        let workspace_id = h.app.public_workspace_id(1);

        let response = h.app.handle_api_request(crate::api::schema::Request {
            id: "close".into(),
            method: crate::api::schema::Method::WorkspaceClose(
                crate::api::schema::WorkspaceCloseParams {
                    workspace_id,
                    close_group: false,
                },
            ),
        });
        let response: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["result"]["type"], "ok");

        assert_eq!(h.parent_input(), "GONE w1\r");
        assert!(h.app.agent_parent_links.is_empty());
    }

    #[tokio::test]
    async fn child_without_parent_or_with_notify_off_stays_silent() {
        for (notify, parent_token) in [(false, true), (true, false)] {
            let mut h = harness(notify, parent_token);
            h.child_status(AgentState::Blocked, AgentStatus::Blocked);
            assert_eq!(
                h.parent_input(),
                "",
                "notify={notify} parent={parent_token}"
            );
            assert!(h.app.agent_parent_links.is_empty());
        }
    }

    #[test]
    fn agent_parent_notify_is_opt_in() {
        assert!(!Config::default().experimental.agent_parent_notify);
        let config: Config =
            toml::from_str("[experimental]\nagent_parent_notify = true\n").unwrap();
        assert!(config.experimental.agent_parent_notify);
    }

    #[tokio::test]
    async fn reindex_after_restore_seeds_blocked_notified() {
        let mut h = harness(true, true);
        terminal_mut(&mut h.app, 1, h.child_pane)
            .set_detected_state(Some(Agent::Claude), AgentState::Blocked);
        h.app.reindex_after_restore();

        h.child_status(AgentState::Blocked, AgentStatus::Blocked);
        assert_eq!(h.parent_input(), "");

        h.child_status(AgentState::Working, AgentStatus::Working);
        h.child_status(AgentState::Blocked, AgentStatus::Blocked);
        assert_eq!(h.parent_input(), "BLOCKED w1\r");
    }
}
