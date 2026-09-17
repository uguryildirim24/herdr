//! Agent state read from pane graphics frames.
//!
//! Some agents are drawn entirely as a streamed image, for example a ChatGPT
//! web chat that a terminal browser draws through the pane graphics API.
//! Screen detection reads terminal text, so those panes never get a status.
//! With `experimental.pane_graphics_detection` on, the server hands the newest
//! file frame of an opted-in pane to one background worker. The worker finds
//! the composer's round action button in the frame pixels and reads its glyph
//! (see [`button`]). The result is published as one synthetic detection line
//! that the agent's screen manifest matches, so detection itself stays a pure
//! read of a screen snapshot, explainable with `herdr agent explain` and
//! tunable by manifest hot reload.
//!
//! Cost is bounded independently of the frame rate: submitting a frame is a
//! lease clone and a map insert, each pane keeps only its newest pending frame,
//! a pane is checked at most once per [`worker::MIN_CHECK_INTERVAL`], a check
//! reads only the rows around the last known button position, and a check
//! whose button pixels did not change stops after hashing them. Without new
//! frames the worker sleeps.

pub(crate) mod button;
mod debug_dump;
#[cfg(test)]
pub(crate) mod scene;
mod worker;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::detect::Agent;
pub(crate) use button::Glyph;
use button::{Disk, PixelOrder};
pub(crate) use worker::submit_frame;

/// Share of the pane grid a frame must cover for the pane to count as
/// graphics-only.
const MIN_COVERAGE_PERCENT: u64 = 90;

static ENABLED: AtomicBool = AtomicBool::new(false);
/// Bumped whenever the feature toggles so results from an earlier enabled
/// period are never used again.
static EPOCH: AtomicU64 = AtomicU64::new(0);
/// Set when the worker thread cannot be started.
static UNAVAILABLE: AtomicBool = AtomicBool::new(false);
static NEXT_SLOT_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn set_enabled(enabled: bool) {
    if ENABLED.swap(enabled, Ordering::AcqRel) != enabled {
        EPOCH.fetch_add(1, Ordering::AcqRel);
    }
}

pub(crate) fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire) && !UNAVAILABLE.load(Ordering::Acquire)
}

fn current_epoch() -> u64 {
    EPOCH.load(Ordering::Acquire)
}

/// Whether an agent's state is read from its pane graphics frames. Only agents
/// whose UI is drawn as images opt in, so ordinary panes never pay for it.
pub(crate) fn agent_reads_graphics(agent: Agent) -> bool {
    matches!(agent, Agent::Chatgpt)
}

impl Glyph {
    /// The line detection reads for this glyph; the agent manifest matches it.
    pub(crate) fn detection_line(self) -> &'static str {
        match self {
            Glyph::Stop => "[herdr pane graphics] composer button: stop",
            Glyph::Waveform => "[herdr pane graphics] composer button: voice",
            Glyph::Arrow => "[herdr pane graphics] composer button: send",
        }
    }
}

/// What one check saw at the button position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Observed {
    NotFound,
    /// A button whose glyph is not recognized.
    Unknown,
    Glyph(Glyph),
}

impl Observed {
    fn label(self) -> &'static str {
        match self {
            Observed::NotFound => "not-found",
            Observed::Unknown => "unknown",
            Observed::Glyph(Glyph::Stop) => "stop",
            Observed::Glyph(Glyph::Waveform) => "voice",
            Observed::Glyph(Glyph::Arrow) => "send",
        }
    }
}

/// A frame the worker can sample later, off the thread that received it.
pub(crate) trait FrameSource: Send + 'static {
    /// Reads `data.len()` frame bytes starting at byte `offset`.
    fn read_at(&self, offset: usize, data: &mut [u8]) -> std::io::Result<()>;
}

impl FrameSource for crate::pane_graphics_files::Lease {
    fn read_at(&self, offset: usize, data: &mut [u8]) -> std::io::Result<()> {
        crate::pane_graphics_files::Lease::read_at(self, offset, data)
    }
}

/// Where the button was last found in a pane's frames.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Locator {
    frame: (u32, u32),
    disk: Disk,
    /// Digest of the pixels around the disk, when they were read.
    crop_digest: Option<u64>,
}

/// Per-pane graphics detection state, shared by the pane runtime, its
/// detection task, and the worker.
#[derive(Debug)]
pub(crate) struct GraphicsDetection {
    id: u64,
    detection_content_seq: Arc<AtomicU64>,
    agent_opted_in: AtomicBool,
    /// One more than the epoch of the published glyph, or 0 without one. Lets
    /// the detection task notice a stale glyph without taking the lock.
    published_epoch: AtomicU64,
    state: Mutex<SlotState>,
}

#[derive(Debug, Default)]
struct SlotState {
    epoch: u64,
    /// The last recognized glyph; kept while the button is missing or unreadable.
    glyph: Option<Glyph>,
    locator: Option<Locator>,
    observed: Option<Observed>,
}

impl GraphicsDetection {
    pub(crate) fn new(detection_content_seq: Arc<AtomicU64>) -> Arc<Self> {
        Arc::new(Self {
            id: NEXT_SLOT_ID.fetch_add(1, Ordering::Relaxed),
            detection_content_seq,
            agent_opted_in: AtomicBool::new(false),
            published_epoch: AtomicU64::new(0),
            state: Mutex::new(SlotState::default()),
        })
    }

    fn lock(&self) -> MutexGuard<'_, SlotState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records the pane's current agent. Called by the detection task every
    /// tick; two atomic reads unless a published glyph went stale because the
    /// agent left or the feature was toggled, in which case detection is
    /// pointed back at terminal text.
    pub(crate) fn observe_agent(&self, agent: Option<Agent>) {
        let opted_in = agent.is_some_and(agent_reads_graphics);
        self.agent_opted_in.store(opted_in, Ordering::Release);
        let published = self.published_epoch.load(Ordering::Acquire);
        if published != 0 && (!self.wants_frames() || published != current_epoch() + 1) {
            self.clear();
        }
    }

    /// Whether new frames of this pane should be checked.
    pub(crate) fn wants_frames(&self) -> bool {
        is_enabled() && self.agent_opted_in.load(Ordering::Acquire)
    }

    /// The synthetic detection screen for this pane, once a glyph was read.
    pub(crate) fn detection_text(&self) -> Option<String> {
        if !self.wants_frames() {
            return None;
        }
        let state = self.lock();
        (state.epoch == current_epoch())
            .then_some(state.glyph)
            .flatten()
            .map(|glyph| glyph.detection_line().to_owned())
    }

    /// Forgets everything read from this pane's frames, so detection falls
    /// back to terminal text.
    pub(crate) fn clear(&self) {
        let mut state = self.lock();
        let had_glyph = state.glyph.is_some();
        *state = SlotState::default();
        self.published_epoch.store(0, Ordering::Release);
        drop(state);
        if had_glyph {
            self.detection_content_seq.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn cached_locator(&self, epoch: u64, frame: (u32, u32)) -> Option<Locator> {
        let state = self.lock();
        state
            .locator
            .filter(|locator| state.epoch == epoch && locator.frame == frame)
    }

    /// Stores one check's result. Unknown and missing buttons keep the last
    /// glyph. Returns whether the observation differs from the previous check.
    fn record(&self, epoch: u64, observed: Observed, locator: Option<Locator>) -> bool {
        let mut state = self.lock();
        let before = state.glyph.filter(|_| state.epoch == epoch);
        if state.epoch != epoch {
            *state = SlotState {
                epoch,
                ..SlotState::default()
            };
        }
        state.locator = locator;
        let observation_changed = state.observed != Some(observed);
        state.observed = Some(observed);
        if let Observed::Glyph(glyph) = observed {
            state.glyph = Some(glyph);
        }
        let glyph_changed = state.glyph != before;
        self.published_epoch.store(
            state.glyph.map_or(0, |_| epoch.wrapping_add(1)),
            Ordering::Release,
        );
        drop(state);
        if glyph_changed {
            self.detection_content_seq.fetch_add(1, Ordering::Relaxed);
        }
        observation_changed
    }
}

/// Whether an image placed at the given cell rectangle covers enough of a
/// `rows` x `cols` pane for the pane to be graphics-only. Zero grid sizes mean
/// "fill the pane", matching how placements are rendered.
pub(crate) fn placement_covers_pane(
    viewport_col: i32,
    viewport_row: i32,
    grid_cols: u32,
    grid_rows: u32,
    rows: u16,
    cols: u16,
) -> bool {
    fn covered(start: i32, len: u32, pane_len: u16) -> u64 {
        let pane_len = i64::from(pane_len);
        let len = if len == 0 { pane_len } else { i64::from(len) };
        let start = i64::from(start);
        let end = start.saturating_add(len).min(pane_len);
        u64::try_from(end.saturating_sub(start.max(0))).unwrap_or(0)
    }
    let total = u64::from(rows) * u64::from(cols);
    if total == 0 {
        return false;
    }
    let covered_cells =
        covered(viewport_col, grid_cols, cols) * covered(viewport_row, grid_rows, rows);
    covered_cells * 100 >= total * MIN_COVERAGE_PERCENT
}

fn pixel_order(bgra: bool) -> PixelOrder {
    if bgra {
        PixelOrder::Bgra
    } else {
        PixelOrder::Rgba
    }
}

#[cfg(test)]
pub(crate) fn test_guard() -> MutexGuard<'static, ()> {
    static GUARD: Mutex<()> = Mutex::new(());
    GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn enabled_slot() -> (Arc<GraphicsDetection>, Arc<AtomicU64>) {
        set_enabled(true);
        let seq = Arc::new(AtomicU64::new(0));
        let slot = GraphicsDetection::new(seq.clone());
        slot.observe_agent(Some(Agent::Chatgpt));
        (slot, seq)
    }

    #[test]
    fn only_graphics_drawn_agents_opt_in() {
        assert!(agent_reads_graphics(Agent::Chatgpt));
        for agent in Agent::ALL
            .into_iter()
            .filter(|agent| *agent != Agent::Chatgpt)
        {
            assert!(!agent_reads_graphics(agent), "{agent:?}");
        }
    }

    #[test]
    fn coverage_requires_the_image_to_fill_most_of_the_pane() {
        assert!(placement_covers_pane(0, 0, 0, 0, 40, 120));
        assert!(placement_covers_pane(0, 0, 120, 40, 40, 120));
        assert!(placement_covers_pane(0, 0, 200, 80, 40, 120));
        assert!(placement_covers_pane(0, 1, 120, 39, 40, 120));
        assert!(!placement_covers_pane(0, 0, 60, 40, 40, 120));
        assert!(!placement_covers_pane(0, 20, 120, 40, 40, 120));
        assert!(!placement_covers_pane(-100, 0, 120, 40, 40, 120));
        assert!(!placement_covers_pane(0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn unknown_and_missing_buttons_keep_the_last_glyph() {
        let _guard = test_guard();
        let (slot, seq) = enabled_slot();
        let epoch = current_epoch();
        assert!(slot.record(epoch, Observed::Glyph(Glyph::Stop), None));
        assert_eq!(
            slot.detection_text().as_deref(),
            Some(Glyph::Stop.detection_line())
        );
        assert_eq!(seq.load(Ordering::Relaxed), 1);

        assert!(slot.record(epoch, Observed::Unknown, None));
        assert!(slot.record(epoch, Observed::NotFound, None));
        assert!(!slot.record(epoch, Observed::NotFound, None));
        assert_eq!(
            slot.detection_text().as_deref(),
            Some(Glyph::Stop.detection_line())
        );
        assert_eq!(
            seq.load(Ordering::Relaxed),
            1,
            "no rescan without a new glyph"
        );

        assert!(slot.record(epoch, Observed::Glyph(Glyph::Waveform), None));
        assert_eq!(
            slot.detection_text().as_deref(),
            Some(Glyph::Waveform.detection_line())
        );
        assert_eq!(seq.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn state_is_dropped_when_agent_leaves_or_feature_toggles() {
        let _guard = test_guard();
        let (slot, seq) = enabled_slot();
        slot.record(current_epoch(), Observed::Glyph(Glyph::Arrow), None);
        assert_eq!(seq.load(Ordering::Relaxed), 1);

        set_enabled(false);
        assert_eq!(slot.detection_text(), None);
        assert!(!slot.wants_frames());
        slot.observe_agent(Some(Agent::Chatgpt));
        assert_eq!(
            seq.load(Ordering::Relaxed),
            2,
            "turning the feature off rescans terminal text"
        );
        slot.observe_agent(Some(Agent::Chatgpt));
        assert_eq!(seq.load(Ordering::Relaxed), 2);

        set_enabled(true);
        slot.record(current_epoch(), Observed::Glyph(Glyph::Arrow), None);
        set_enabled(false);
        set_enabled(true);
        assert_eq!(
            slot.detection_text(),
            None,
            "a glyph from an earlier period is stale"
        );
        slot.observe_agent(Some(Agent::Chatgpt));
        assert_eq!(seq.load(Ordering::Relaxed), 4, "a stale glyph is dropped");

        slot.record(current_epoch(), Observed::Glyph(Glyph::Arrow), None);
        assert!(slot.detection_text().is_some());
        slot.observe_agent(Some(Agent::Codex));
        assert!(!slot.wants_frames());
        assert_eq!(slot.detection_text(), None);
        assert_eq!(seq.load(Ordering::Relaxed), 6);

        slot.observe_agent(Some(Agent::Chatgpt));
        slot.clear();
        assert_eq!(seq.load(Ordering::Relaxed), 6, "clearing nothing is silent");
    }

    #[test]
    fn detection_lines_are_distinct_and_single_line() {
        let lines = [Glyph::Stop, Glyph::Waveform, Glyph::Arrow].map(Glyph::detection_line);
        assert_ne!(lines[0], lines[1]);
        assert_ne!(lines[1], lines[2]);
        assert!(lines.iter().all(|line| !line.contains('\n')));
    }
}
