//! The background worker that checks pane frames for the composer button.

use std::collections::HashMap;
use std::io;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, Weak};
use std::time::{Duration, Instant};

use super::button::{self, PixelRect, Rows};
use super::debug_dump::DebugDump;
use super::{
    current_epoch, pixel_order, FrameSource, GraphicsDetection, Locator, Observed, UNAVAILABLE,
};

/// Minimum time between two checks of the same pane.
pub(crate) const MIN_CHECK_INTERVAL: Duration = Duration::from_millis(500);
/// Scratch capacity kept between checks: enough for the rows around the button
/// of a wide frame, not for a full search band.
const RETAINED_SCRATCH_BYTES: usize = 2 * 1024 * 1024;

pub(super) struct PendingFrame {
    pub(super) target: Weak<GraphicsDetection>,
    pub(super) frame: Box<dyn FrameSource>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) bgra: bool,
    pub(super) epoch: u64,
}

#[derive(Default)]
pub(super) struct Schedule {
    pending: HashMap<u64, PendingFrame>,
    next_allowed: HashMap<u64, Instant>,
}

pub(super) enum Due {
    Ready(PendingFrame),
    At(Instant),
    Idle,
}

impl Schedule {
    /// Replaces the pane's pending frame. Returns true when the pane had no
    /// pending frame, which is the only case the worker must be woken for.
    pub(super) fn submit(&mut self, slot_id: u64, frame: PendingFrame) -> bool {
        self.pending.insert(slot_id, frame).is_none()
    }

    pub(super) fn take_due(&mut self, now: Instant) -> Due {
        self.pending
            .retain(|_, pending| pending.target.strong_count() > 0);
        let pending = &self.pending;
        self.next_allowed
            .retain(|slot_id, allowed| *allowed > now || pending.contains_key(slot_id));
        let next = self
            .pending
            .keys()
            .map(|slot_id| {
                let due = self
                    .next_allowed
                    .get(slot_id)
                    .copied()
                    .filter(|allowed| *allowed > now)
                    .unwrap_or(now);
                (due, *slot_id)
            })
            .min();
        let Some((due, slot_id)) = next else {
            return Due::Idle;
        };
        if due > now {
            return Due::At(due);
        }
        let Some(frame) = self.pending.remove(&slot_id) else {
            return Due::Idle;
        };
        self.next_allowed.insert(slot_id, now + MIN_CHECK_INTERVAL);
        Due::Ready(frame)
    }
}

/// What one check did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CheckOutcome {
    /// Nothing to do: the feature is off, the agent left, the pane closed, or
    /// the frame could not be read.
    Skipped,
    /// The pixels around the known button position did not change.
    Unchanged,
    Checked {
        /// Whether the whole search band was scanned instead of only the rows
        /// around the known button position.
        searched: bool,
        observed: Observed,
    },
}

fn read_rows(
    frame: &dyn FrameSource,
    frame_width: u32,
    rows: Range<u32>,
    scratch: &mut Vec<u8>,
) -> io::Result<()> {
    let overflow = || io::Error::new(io::ErrorKind::InvalidInput, "frame rows out of range");
    let row_bytes = (frame_width as usize).checked_mul(4).ok_or_else(overflow)?;
    let offset = (rows.start as usize)
        .checked_mul(row_bytes)
        .ok_or_else(overflow)?;
    let len = (rows.end.saturating_sub(rows.start) as usize)
        .checked_mul(row_bytes)
        .ok_or_else(overflow)?;
    if scratch.len() != len {
        scratch.resize(len, 0);
    }
    frame.read_at(offset, scratch)
}

/// A 64-bit digest of the pixels in `rect`, or `None` when `rows` does not
/// hold all of them.
fn crop_digest(rows: &Rows<'_>, rect: PixelRect) -> Option<u64> {
    const MULTIPLIER: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut hash = u64::from(rect.x0) ^ (u64::from(rect.y0) << 32);
    for row in rows.rect_bytes(rect)? {
        let mut chunks = row.chunks_exact(8);
        for chunk in &mut chunks {
            let mut word = [0_u8; 8];
            word.copy_from_slice(chunk);
            hash = (hash ^ u64::from_le_bytes(word))
                .wrapping_mul(MULTIPLIER)
                .rotate_left(29);
        }
        for &byte in chunks.remainder() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(MULTIPLIER);
        }
    }
    Some(hash ^ (hash >> 32))
}

/// Checks one frame into its pane slot.
pub(super) fn check_frame(
    job: PendingFrame,
    scratch: &mut Vec<u8>,
    dump: Option<&DebugDump>,
) -> CheckOutcome {
    let Some(target) = job.target.upgrade() else {
        return CheckOutcome::Skipped;
    };
    if !target.wants_frames() || job.epoch != current_epoch() {
        return CheckOutcome::Skipped;
    }
    let frame = (job.width, job.height);
    let order = pixel_order(job.bgra);
    let skipped = |err: io::Error| {
        // The producer may already have replaced the frame; a newer frame is
        // submitted with its own lease.
        tracing::debug!(err = %err, "skipping pane graphics frame for button detection");
        CheckOutcome::Skipped
    };

    if let Some(locator) = target.cached_locator(job.epoch, frame) {
        if let Some(crop) = locator.disk.crop(job.width, job.height) {
            if let Err(err) = read_rows(&*job.frame, job.width, crop.y0..crop.y1, scratch) {
                return skipped(err);
            }
            if let Some(rows) = Rows::new(job.width, crop.y0, scratch, order) {
                let digest = crop_digest(&rows, crop);
                if digest.is_some() && digest == locator.crop_digest {
                    return CheckOutcome::Unchanged;
                }
                if let Some(verified) = button::verify(&rows, locator.disk) {
                    let reading = button::classify(&rows, verified);
                    let observed = observed_glyph(reading.glyph);
                    let locator = Locator {
                        frame,
                        disk: verified.disk,
                        crop_digest: verified
                            .disk
                            .crop(job.width, job.height)
                            .and_then(|crop| crop_digest(&rows, crop)),
                    };
                    if target.record(job.epoch, observed, Some(locator)) {
                        if let Some(dump) = dump {
                            dump.found(target.id, frame, observed, verified, reading, &rows);
                        }
                    }
                    return CheckOutcome::Checked {
                        searched: false,
                        observed,
                    };
                }
            }
        }
    }

    let band = button::search_rows(job.height);
    if let Err(err) = read_rows(&*job.frame, job.width, band.clone(), scratch) {
        return skipped(err);
    }
    let outcome = match Rows::new(job.width, band.start, scratch, order) {
        Some(rows) => {
            let (observed, locator, found) = match button::locate(&rows) {
                Some(verified) => {
                    let reading = button::classify(&rows, verified);
                    let locator = Locator {
                        frame,
                        disk: verified.disk,
                        crop_digest: verified
                            .disk
                            .crop(job.width, job.height)
                            .and_then(|crop| crop_digest(&rows, crop)),
                    };
                    (
                        observed_glyph(reading.glyph),
                        Some(locator),
                        Some((verified, reading)),
                    )
                }
                None => (Observed::NotFound, None, None),
            };
            if target.record(job.epoch, observed, locator) {
                if let Some(dump) = dump {
                    match found {
                        Some((verified, reading)) => {
                            dump.found(target.id, frame, observed, verified, reading, &rows)
                        }
                        None => dump.not_found(target.id, frame, &rows),
                    }
                }
            }
            CheckOutcome::Checked {
                searched: true,
                observed,
            }
        }
        None => CheckOutcome::Skipped,
    };
    if scratch.capacity() > RETAINED_SCRATCH_BYTES {
        *scratch = Vec::new();
    }
    outcome
}

fn observed_glyph(glyph: Option<button::Glyph>) -> Observed {
    glyph.map_or(Observed::Unknown, Observed::Glyph)
}

struct Worker {
    schedule: Mutex<Schedule>,
    wake: Condvar,
    started: AtomicBool,
}

fn worker() -> &'static Worker {
    static WORKER: OnceLock<Worker> = OnceLock::new();
    WORKER.get_or_init(|| Worker {
        schedule: Mutex::new(Schedule::default()),
        wake: Condvar::new(),
        started: AtomicBool::new(false),
    })
}

impl Worker {
    fn lock(&self) -> MutexGuard<'_, Schedule> {
        self.schedule
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn ensure_thread(&'static self) {
        if self.started.swap(true, Ordering::AcqRel) {
            return;
        }
        let spawned = std::thread::Builder::new()
            .name("herdr-graphics-detect".into())
            .spawn(move || self.run());
        if let Err(err) = spawned {
            tracing::warn!(err = %err, "failed to start pane graphics detection worker");
            UNAVAILABLE.store(true, Ordering::Release);
            self.lock().pending.clear();
        }
    }

    fn run(&self) {
        crate::platform::lower_current_thread_priority();
        let dump = DebugDump::from_env();
        let mut scratch = Vec::new();
        loop {
            let job = {
                let mut schedule = self.lock();
                loop {
                    match schedule.take_due(Instant::now()) {
                        Due::Ready(job) => break job,
                        Due::At(when) => {
                            let timeout = when.saturating_duration_since(Instant::now());
                            schedule = self
                                .wake
                                .wait_timeout(schedule, timeout)
                                .map(|(guard, _)| guard)
                                .unwrap_or_else(|poisoned| poisoned.into_inner().0);
                        }
                        Due::Idle => {
                            schedule = self
                                .wake
                                .wait(schedule)
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                        }
                    }
                }
            };
            check_frame(job, &mut scratch, dump.as_ref());
        }
    }
}

/// Queues the newest frame of a pane for a button check, replacing any frame
/// of the same pane that was not checked yet. Cheap enough to call for every
/// frame the pane receives; does nothing unless the pane wants frames.
pub(crate) fn submit_frame(
    target: &Arc<GraphicsDetection>,
    frame: impl FrameSource,
    width: u32,
    height: u32,
    bgra: bool,
) {
    if !target.wants_frames() {
        return;
    }
    let worker = worker();
    let wake = worker.lock().submit(
        target.id,
        PendingFrame {
            target: Arc::downgrade(target),
            frame: Box::new(frame),
            width,
            height,
            bgra,
            epoch: current_epoch(),
        },
    );
    worker.ensure_thread();
    if wake {
        worker.wake.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::Agent;
    use crate::graphics_detection::scene::{Glyph as SceneGlyph, Scene};
    use crate::graphics_detection::{set_enabled, test_guard, Glyph};
    use std::sync::atomic::{AtomicU64, AtomicUsize};

    /// An in-memory frame that counts the bytes read from it.
    #[derive(Clone)]
    struct CountingFrame {
        pixels: Arc<Vec<u8>>,
        bytes_read: Arc<AtomicUsize>,
    }

    impl CountingFrame {
        fn new(pixels: Vec<u8>) -> Self {
            Self {
                pixels: Arc::new(pixels),
                bytes_read: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl FrameSource for CountingFrame {
        fn read_at(&self, offset: usize, data: &mut [u8]) -> io::Result<()> {
            let source = self
                .pixels
                .get(offset..offset + data.len())
                .ok_or_else(|| io::Error::other("out of range"))?;
            data.copy_from_slice(source);
            self.bytes_read.fetch_add(data.len(), Ordering::Relaxed);
            Ok(())
        }
    }

    struct FailingFrame;

    impl FrameSource for FailingFrame {
        fn read_at(&self, _offset: usize, _data: &mut [u8]) -> io::Result<()> {
            Err(io::Error::other("replaced"))
        }
    }

    fn enabled_slot() -> (Arc<GraphicsDetection>, Arc<AtomicU64>) {
        set_enabled(true);
        let seq = Arc::new(AtomicU64::new(0));
        let slot = GraphicsDetection::new(seq.clone());
        slot.observe_agent(Some(Agent::Chatgpt));
        (slot, seq)
    }

    fn job(slot: &Arc<GraphicsDetection>, frame: impl FrameSource, scene: &Scene) -> PendingFrame {
        PendingFrame {
            target: Arc::downgrade(slot),
            frame: Box::new(frame),
            width: scene.width,
            height: scene.height,
            bgra: false,
            epoch: current_epoch(),
        }
    }

    fn check(slot: &Arc<GraphicsDetection>, scene: &Scene) -> (CheckOutcome, usize) {
        let frame = CountingFrame::new(scene.render());
        let bytes_read = frame.bytes_read.clone();
        let outcome = check_frame(job(slot, frame, scene), &mut Vec::new(), None);
        (outcome, bytes_read.load(Ordering::Relaxed))
    }

    fn line(slot: &GraphicsDetection) -> Option<String> {
        slot.detection_text()
    }

    #[test]
    fn a_located_button_is_rechecked_from_its_crop_and_searched_again_when_it_moves() {
        let _guard = test_guard();
        let (slot, seq) = enabled_slot();
        let stop = Scene::chat(SceneGlyph::Stop);
        let band_bytes = button::search_rows(stop.height).len() * stop.width as usize * 4;

        let (outcome, read) = check(&slot, &stop);
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: true,
                observed: Observed::Glyph(Glyph::Stop)
            }
        );
        assert_eq!(read, band_bytes);
        assert_eq!(line(&slot).as_deref(), Some(Glyph::Stop.detection_line()));
        assert_eq!(seq.load(Ordering::Relaxed), 1);

        // The reply finished: same position, new glyph, read from the crop only.
        let voice = Scene::chat(SceneGlyph::Waveform);
        let (outcome, read) = check(&slot, &voice);
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: false,
                observed: Observed::Glyph(Glyph::Waveform)
            }
        );
        assert!(
            read * 3 < band_bytes,
            "crop read {read} vs band {band_bytes}"
        );
        assert_eq!(
            line(&slot).as_deref(),
            Some(Glyph::Waveform.detection_line())
        );

        // A multi-line draft grows the composer upward; the button keeps its
        // bottom-right position and still verifies from the crop.
        let draft = Scene::chat(SceneGlyph::Arrow).with_draft_lines(5);
        let (outcome, _) = check(&slot, &draft);
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: false,
                observed: Observed::Glyph(Glyph::Arrow)
            }
        );
        assert_eq!(line(&slot).as_deref(), Some(Glyph::Arrow.detection_line()));

        // Zooming the page moves and resizes the button within the same frame
        // size: the crop no longer verifies, so the band is searched again.
        let zoomed = Scene::chat(SceneGlyph::Stop).scaled(1.5);
        let (outcome, read) = check(&slot, &zoomed);
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: true,
                observed: Observed::Glyph(Glyph::Stop)
            }
        );
        assert!(read > band_bytes);
        assert_eq!(line(&slot).as_deref(), Some(Glyph::Stop.detection_line()));
        assert_eq!(seq.load(Ordering::Relaxed), 4);
    }

    #[test]
    fn a_frame_with_unchanged_button_pixels_is_not_classified_again() {
        let _guard = test_guard();
        let (slot, seq) = enabled_slot();
        let scene = Scene::chat(SceneGlyph::Waveform);
        check(&slot, &scene);
        let (outcome, read) = check(&slot, &scene);
        assert_eq!(outcome, CheckOutcome::Unchanged);
        assert!(read < 2 * 1024 * 1024);
        assert_eq!(seq.load(Ordering::Relaxed), 1);

        // A new frame size invalidates the cached position.
        let resized = scene.resized(2000, 1400);
        let (outcome, _) = check(&slot, &resized);
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: true,
                observed: Observed::Glyph(Glyph::Waveform)
            }
        );
        assert_eq!(seq.load(Ordering::Relaxed), 1, "same glyph, no rescan");
    }

    #[test]
    fn a_missing_or_unreadable_button_keeps_the_last_state() {
        let _guard = test_guard();
        let (slot, seq) = enabled_slot();
        check(&slot, &Scene::chat(SceneGlyph::Stop));
        let (outcome, _) = check(&slot, &Scene::chat(SceneGlyph::Cross));
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: false,
                observed: Observed::Unknown
            }
        );
        let (outcome, _) = check(&slot, &Scene::chat(SceneGlyph::NoButton));
        assert_eq!(
            outcome,
            CheckOutcome::Checked {
                searched: true,
                observed: Observed::NotFound
            }
        );
        let scene = Scene::chat(SceneGlyph::Stop);
        assert_eq!(
            check_frame(job(&slot, FailingFrame, &scene), &mut Vec::new(), None),
            CheckOutcome::Skipped
        );
        assert_eq!(line(&slot).as_deref(), Some(Glyph::Stop.detection_line()));
        assert_eq!(seq.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn nothing_is_read_while_the_feature_is_off_or_the_agent_does_not_opt_in() {
        let _guard = test_guard();
        let (slot, seq) = enabled_slot();
        let scene = Scene::chat(SceneGlyph::Stop);

        slot.observe_agent(Some(Agent::Codex));
        let frame = CountingFrame::new(Vec::new());
        let bytes_read = frame.bytes_read.clone();
        assert_eq!(
            check_frame(job(&slot, frame.clone(), &scene), &mut Vec::new(), None),
            CheckOutcome::Skipped
        );
        submit_frame(&slot, frame.clone(), scene.width, scene.height, false);

        slot.observe_agent(Some(Agent::Chatgpt));
        let stale = job(&slot, frame.clone(), &scene);
        set_enabled(false);
        submit_frame(&slot, frame.clone(), scene.width, scene.height, false);
        assert!(!worker().lock().pending.contains_key(&slot.id));
        set_enabled(true);
        assert_eq!(
            check_frame(stale, &mut Vec::new(), None),
            CheckOutcome::Skipped,
            "a frame queued before a toggle is stale"
        );
        assert_eq!(bytes_read.load(Ordering::Relaxed), 0);
        assert_eq!(seq.load(Ordering::Relaxed), 0);
        assert!(!worker().lock().pending.contains_key(&slot.id));
    }

    #[test]
    fn schedule_keeps_only_the_newest_frame_per_pane() {
        let _guard = test_guard();
        let (slot, _) = enabled_slot();
        let scene = Scene::chat(SceneGlyph::Stop);
        let mut schedule = Schedule::default();
        let now = Instant::now();
        let first = CountingFrame::new(vec![1; 4]);
        let second = CountingFrame::new(vec![2; 4]);
        assert!(schedule.submit(slot.id, job(&slot, first, &scene)));
        assert!(!schedule.submit(slot.id, job(&slot, second, &scene)));
        let Due::Ready(frame) = schedule.take_due(now) else {
            panic!("frame should be due");
        };
        let mut data = [0; 4];
        frame.frame.read_at(0, &mut data).expect("read");
        assert_eq!(data, [2; 4]);
        assert!(matches!(schedule.take_due(now), Due::Idle));
    }

    #[test]
    fn schedule_limits_each_pane_to_two_checks_per_second() {
        let _guard = test_guard();
        let (slot, _) = enabled_slot();
        let (other, _) = enabled_slot();
        let scene = Scene::chat(SceneGlyph::Stop);
        let frame = || CountingFrame::new(vec![0; 4]);
        let mut schedule = Schedule::default();
        let now = Instant::now();
        schedule.submit(slot.id, job(&slot, frame(), &scene));
        assert!(matches!(schedule.take_due(now), Due::Ready(_)));

        schedule.submit(slot.id, job(&slot, frame(), &scene));
        match schedule.take_due(now + Duration::from_millis(200)) {
            Due::At(when) => assert_eq!(when, now + MIN_CHECK_INTERVAL),
            _ => panic!("second frame must wait for the interval"),
        }
        assert_eq!(MIN_CHECK_INTERVAL, Duration::from_millis(500));

        // Another pane is not held back by the first pane's interval.
        schedule.submit(other.id, job(&other, frame(), &scene));
        let Due::Ready(ready) = schedule.take_due(now + Duration::from_millis(200)) else {
            panic!("other pane should be due");
        };
        assert!(Weak::ptr_eq(&ready.target, &Arc::downgrade(&other)));

        assert!(matches!(
            schedule.take_due(now + MIN_CHECK_INTERVAL),
            Due::Ready(_)
        ));
    }

    #[test]
    fn schedule_drops_frames_of_closed_panes() {
        let _guard = test_guard();
        let (slot, _) = enabled_slot();
        let scene = Scene::chat(SceneGlyph::Stop);
        let mut schedule = Schedule::default();
        schedule.submit(slot.id, job(&slot, CountingFrame::new(Vec::new()), &scene));
        drop(slot);
        assert!(matches!(schedule.take_due(Instant::now()), Due::Idle));
        assert!(schedule.pending.is_empty());
    }

    /// Per-check cost on a 2400x1600 frame read through a real file, for the
    /// cached-position path, the unchanged path, and a full search.
    #[cfg(unix)]
    #[test]
    #[ignore = "benchmark: run with --run-ignored only --no-capture"]
    fn bench_check_cost_on_a_2400x1600_file_frame() {
        use std::io::Write;
        use std::os::unix::fs::FileExt;

        struct FileFrame(std::fs::File);
        impl FrameSource for FileFrame {
            fn read_at(&self, offset: usize, data: &mut [u8]) -> io::Result<()> {
                self.0.read_exact_at(data, offset as u64)
            }
        }
        fn thread_cpu() -> Duration {
            let mut ts = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            // SAFETY: clock_gettime writes into the provided timespec.
            unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) };
            Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
        }
        let write = |scene: &Scene, name: &str| {
            let path = std::env::temp_dir()
                .join(format!("herdr-button-bench-{}-{name}", std::process::id()));
            let mut file = std::fs::File::create(&path).expect("create");
            file.write_all(&scene.render()).expect("write");
            drop(file);
            let file = std::fs::File::open(&path).expect("open");
            let _ = std::fs::remove_file(&path);
            file
        };

        let _guard = test_guard();
        let stop = Scene::chat(SceneGlyph::Stop);
        let voice = Scene::chat(SceneGlyph::Waveform);
        let stop_file = Arc::new(write(&stop, "stop"));
        let voice_file = Arc::new(write(&voice, "voice"));
        let empty_file = Arc::new(write(&Scene::chat(SceneGlyph::NoButton), "empty"));
        let open = |file: &Arc<std::fs::File>| FileFrame(file.try_clone().expect("clone"));
        let iterations = 200;
        let mut scratch = Vec::new();
        let (slot, _) = enabled_slot();
        check_frame(job(&slot, open(&stop_file), &stop), &mut scratch, None);

        let mut measure =
            |label: &str,
             slot: &Arc<GraphicsDetection>,
             frames: &dyn Fn(usize) -> (FileFrame, CheckOutcome)| {
                let mut cpu = Vec::with_capacity(iterations);
                let mut wall = Vec::with_capacity(iterations);
                for iteration in 0..iterations {
                    let (frame, expected) = frames(iteration);
                    let (wall_start, cpu_start) = (Instant::now(), thread_cpu());
                    let outcome = check_frame(job(slot, frame, &stop), &mut scratch, None);
                    cpu.push(thread_cpu() - cpu_start);
                    wall.push(wall_start.elapsed());
                    assert_eq!(outcome, expected, "{label}");
                }
                cpu.sort();
                wall.sort();
                eprintln!(
                    "{label}: thread cpu median {:?} p90 {:?}; wall median {:?} p90 {:?}",
                    cpu[iterations / 2],
                    cpu[iterations * 9 / 10],
                    wall[iterations / 2],
                    wall[iterations * 9 / 10]
                );
            };

        // Cached position, glyph alternating every check.
        measure("cached position, glyph changed", &slot, &|iteration| {
            if iteration % 2 == 0 {
                (
                    open(&voice_file),
                    CheckOutcome::Checked {
                        searched: false,
                        observed: Observed::Glyph(Glyph::Waveform),
                    },
                )
            } else {
                (
                    open(&stop_file),
                    CheckOutcome::Checked {
                        searched: false,
                        observed: Observed::Glyph(Glyph::Stop),
                    },
                )
            }
        });
        measure("cached position, button unchanged", &slot, &|_| {
            (open(&stop_file), CheckOutcome::Unchanged)
        });
        measure("full search (no cached position)", &slot, &|_| {
            slot_reset(&slot);
            (
                open(&stop_file),
                CheckOutcome::Checked {
                    searched: true,
                    observed: Observed::Glyph(Glyph::Stop),
                },
            )
        });
        measure("full search, no button on the page", &slot, &|_| {
            slot_reset(&slot);
            (
                open(&empty_file),
                CheckOutcome::Checked {
                    searched: true,
                    observed: Observed::NotFound,
                },
            )
        });
    }

    fn slot_reset(slot: &GraphicsDetection) {
        slot.lock().locator = None;
    }
}
