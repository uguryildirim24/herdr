//! Reads a web chat composer's round action button from raw frame pixels.
//!
//! The button is a filled disk that contrasts strongly with the composer
//! around it, and the rightmost such disk near the bottom of the page. Its
//! glyph tells the state: one solid square is "stop" (a reply is being
//! generated), separated vertical bars are the voice-mode waveform (idle with
//! an empty draft), and an arrow is "send" (idle with a draft). Every test is
//! on luminance contrast, so a light theme (dark disk, light glyph) reads the
//! same as a dark one.
//!
//! The functions here are pure: they only look at the pixel rows handed to
//! them, and the caller decides which rows of a frame to read.

use std::ops::Range;

/// Smallest and largest plausible button radius in frame pixels. A 36 CSS px
/// button is 18 px at 1x and 54 px at 3x device scale.
pub(crate) const MIN_RADIUS: f32 = 10.0;
pub(crate) const MAX_RADIUS: f32 = 64.0;
/// Share of the frame height, counted from the bottom, searched for the button.
const SEARCH_BAND_PERCENT: u32 = 30;
/// Minimum luminance difference between the disk and its surroundings.
const MIN_CONTRAST: i32 = 96;
/// Maximum luminance spread inside one uniform area.
const UNIFORM_TOLERANCE: i32 = 28;
/// Sampling step of the full search, in frame pixels.
const SEARCH_STEP: i32 = 2;
/// Sampled rows a blob may skip and still continue. An anti-aliased
/// horizontal glyph edge leaves one row with no clean disk run.
const MAX_MISSED_ROWS: i32 = 1;
/// Upper bound on blobs tracked at once by the search, bounding its cost on
/// busy pages.
const MAX_ACTIVE_BLOBS: usize = 1024;
/// Widest horizontal extent a blob may grow to: a disk plus slack.
const MAX_BLOB_WIDTH: i32 = (2.0 * MAX_RADIUS) as i32 + 4 * SEARCH_STEP;
const MAX_VERIFIED_CANDIDATES: usize = 6;
const RING_SAMPLES: usize = 16;
/// Ring samples that may disagree before a disk is rejected.
const RING_OUTLIERS: usize = 2;
/// Radius of the ring that must be disk-colored, as a share of the radius.
/// Glyphs stay inside it.
const INNER_RING: f32 = 0.75;
/// Radius of the ring that must contrast with the disk. Close to the edge so a
/// rounded square fails at its corners.
const OUTER_RING: f32 = 1.15;
/// Half side of the square searched for the glyph, as a share of the radius.
const GLYPH_BOX: f32 = 0.62;
/// Largest glyph box side for `MAX_RADIUS`.
const MAX_GLYPH_BOX: usize = 84;
const MAX_GLYPH_GROUPS: usize = 16;
/// Crop margin around the disk, as a multiple of its radius.
const CROP_RADII: f32 = 1.5;

/// Byte order of the four bytes of each frame pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PixelOrder {
    Rgba,
    Bgra,
}

impl PixelOrder {
    /// Luminance of one 4-byte pixel, 0 to 255.
    fn luma(self, pixel: &[u8]) -> u8 {
        let [first, green, third, ..] = *pixel else {
            return 0;
        };
        let (red, blue) = match self {
            Self::Rgba => (first, third),
            Self::Bgra => (third, first),
        };
        ((77 * u32::from(red) + 150 * u32::from(green) + 29 * u32::from(blue)) >> 8) as u8
    }
}

/// Whole rows of a frame, as raw 4-byte pixels.
#[derive(Clone, Copy)]
pub(crate) struct Rows<'a> {
    frame_width: u32,
    first_row: u32,
    row_count: u32,
    bytes: &'a [u8],
    order: PixelOrder,
}

impl<'a> Rows<'a> {
    /// `bytes` holds rows `first_row..` of a `frame_width` wide frame. Returns
    /// `None` unless it is a whole number of rows.
    pub(crate) fn new(
        frame_width: u32,
        first_row: u32,
        bytes: &'a [u8],
        order: PixelOrder,
    ) -> Option<Self> {
        let row_bytes = usize::try_from(frame_width).ok()?.checked_mul(4)?;
        if row_bytes == 0 || !bytes.len().is_multiple_of(row_bytes) {
            return None;
        }
        let row_count = u32::try_from(bytes.len() / row_bytes).ok()?;
        first_row.checked_add(row_count)?;
        Some(Self {
            frame_width,
            first_row,
            row_count,
            bytes,
            order,
        })
    }

    pub(crate) fn rows(&self) -> Range<u32> {
        self.first_row..self.first_row + self.row_count
    }

    fn pixel(&self, x: i32, y: i32) -> Option<&'a [u8]> {
        let x = u32::try_from(x).ok()?;
        let y = u32::try_from(y).ok()?;
        if x >= self.frame_width || !self.rows().contains(&y) {
            return None;
        }
        let row = (y - self.first_row) as usize;
        let offset = (row * self.frame_width as usize + x as usize) * 4;
        self.bytes.get(offset..offset + 4)
    }

    fn luma(&self, x: i32, y: i32) -> Option<i32> {
        Some(i32::from(self.order.luma(self.pixel(x, y)?)))
    }

    /// The bytes of whole row `y`.
    fn row(&self, y: u32) -> Option<&'a [u8]> {
        if !self.rows().contains(&y) {
            return None;
        }
        let row_bytes = self.frame_width as usize * 4;
        let start = (y - self.first_row) as usize * row_bytes;
        self.bytes.get(start..start + row_bytes)
    }

    /// The pixel as opaque RGBA, for debug crops. Detection ignores alpha, so
    /// the crops do too.
    pub(crate) fn rgba(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let pixel = self.pixel(i32::try_from(x).ok()?, i32::try_from(y).ok()?)?;
        Some(match self.order {
            PixelOrder::Rgba => [pixel[0], pixel[1], pixel[2], u8::MAX],
            PixelOrder::Bgra => [pixel[2], pixel[1], pixel[0], u8::MAX],
        })
    }

    /// The bytes of `rect`, row by row, when all of its rows are present.
    pub(crate) fn rect_bytes(&self, rect: PixelRect) -> Option<impl Iterator<Item = &'a [u8]>> {
        if rect.x1 > self.frame_width
            || rect.x0 >= rect.x1
            || rect.y0 >= rect.y1
            || rect.y0 < self.first_row
            || rect.y1 > self.first_row + self.row_count
        {
            return None;
        }
        let bytes = self.bytes;
        let row_bytes = self.frame_width as usize * 4;
        let first_row = self.first_row;
        Some((rect.y0..rect.y1).filter_map(move |y| {
            let start = (y - first_row) as usize * row_bytes;
            bytes.get(start + rect.x0 as usize * 4..start + rect.x1 as usize * 4)
        }))
    }
}

/// Rows searched for the button in a frame of `frame_height` rows.
pub(crate) fn search_rows(frame_height: u32) -> Range<u32> {
    let band = (u64::from(frame_height) * u64::from(SEARCH_BAND_PERCENT) / 100) as u32;
    frame_height - band..frame_height
}

/// A pixel rectangle, `x0..x1` by `y0..y1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PixelRect {
    pub(crate) x0: u32,
    pub(crate) y0: u32,
    pub(crate) x1: u32,
    pub(crate) y1: u32,
}

/// A disk in frame pixel coordinates; the center is a pixel-center position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Disk {
    pub(crate) cx: f32,
    pub(crate) cy: f32,
    pub(crate) r: f32,
}

impl Disk {
    /// The area around the disk that verifying and classifying it reads,
    /// clamped to the frame. `None` when the disk lies outside the frame.
    pub(crate) fn crop(&self, frame_width: u32, frame_height: u32) -> Option<PixelRect> {
        let margin = self.r * CROP_RADII;
        let clamp = |value: f32, limit: u32| value.clamp(0.0, limit as f32) as u32;
        let rect = PixelRect {
            x0: clamp((self.cx - margin).floor(), frame_width),
            y0: clamp((self.cy - margin).floor(), frame_height),
            x1: clamp((self.cx + margin).ceil() + 1.0, frame_width),
            y1: clamp((self.cy + margin).ceil() + 1.0, frame_height),
        };
        (rect.x0 < rect.x1 && rect.y0 < rect.y1).then_some(rect)
    }
}

/// A disk confirmed by pixel checks, with the luminance measured inside it and
/// just outside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VerifiedDisk {
    pub(crate) disk: Disk,
    pub(crate) disk_luma: i32,
    pub(crate) outer_luma: i32,
}

/// The glyph drawn inside the button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    /// One solid centered square: stop generating.
    Stop,
    /// Several separated vertical bars: voice mode, empty draft.
    Waveform,
    /// A stem with a chevron on top: send the draft.
    Arrow,
}

/// Measurements of the glyph's ink, kept for debugging classifications.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GlyphStats {
    pub(crate) ink: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) groups: u32,
    pub(crate) fill: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphReading {
    pub(crate) glyph: Option<Glyph>,
    pub(crate) stats: GlyphStats,
}

/// Searches `rows` for the rightmost button disk.
pub(crate) fn locate(rows: &Rows<'_>) -> Option<VerifiedDisk> {
    let mut search = Search::default();
    let step = SEARCH_STEP as usize;
    let max_samples = (2.0 * MAX_RADIUS) as usize / step + 1;
    let mut samples: Vec<u8> = Vec::with_capacity((rows.frame_width as usize).div_ceil(step));
    for y in rows.rows().step_by(step) {
        let row = rows.row(y)?;
        samples.clear();
        let mut pairs = row.chunks_exact(4 * step);
        samples.extend((&mut pairs).map(|pair| rows.order.luma(pair)));
        if pairs.remainder().len() >= 4 {
            samples.push(rows.order.luma(pairs.remainder()));
        }
        let y = i32::try_from(y).ok()?;
        let sample = |index: Option<usize>| index.and_then(|index| samples.get(index)).copied();
        let contrasts = |indices: [Option<usize>; 2], seed: u8| {
            indices
                .into_iter()
                .filter_map(sample)
                .any(|luma| i32::from(luma.abs_diff(seed)) >= MIN_CONTRAST)
        };
        let mut start = 0;
        while let Some(&seed) = samples.get(start) {
            let len = samples[start + 1..]
                .iter()
                .position(|luma| i32::from(luma.abs_diff(seed)) > UNIFORM_TOLERANCE)
                .map_or(samples.len() - start, |uniform| uniform + 1);
            let end = start + len;
            if len <= max_samples
                && contrasts([start.checked_sub(1), start.checked_sub(2)], seed)
                && contrasts([Some(end), Some(end + 1)], seed)
            {
                search.add_run(
                    (start * step) as i32,
                    (end * step) as i32,
                    y,
                    i32::from(seed),
                );
            }
            start = end;
        }
        search.finish_row(y);
    }
    let mut candidates = search.finish();
    candidates.sort_by(|a, b| b.cx.total_cmp(&a.cx).then(b.r.total_cmp(&a.r)));
    candidates
        .into_iter()
        .take(MAX_VERIFIED_CANDIDATES)
        .find_map(|candidate| verify(rows, candidate))
}

#[derive(Debug)]
struct Blob {
    luma: i32,
    x0: i32,
    x1: i32,
    first_row: i32,
    last_row: i32,
    samples: u32,
}

/// Groups the uniform runs of consecutive sampled rows into blobs.
///
/// A run joins a blob of similar luminance whose horizontal extent so far it
/// touches, so a disk stays one blob while its glyph splits some rows into
/// several runs. Runs arrive left to right within a row, and blobs from
/// earlier rows stay sorted by where they start, so a run is compared only
/// with the blobs near it: a cursor skips blobs that end left of earlier runs,
/// and the scan stops at the first blob that starts right of the run.
#[derive(Default)]
struct Search {
    /// Blobs from earlier rows, sorted by `x0` at the start of each row.
    active: Vec<Blob>,
    /// Blobs started in the row being scanned.
    born: Vec<Blob>,
    cursor: usize,
    candidates: Vec<Disk>,
}

impl Search {
    fn add_run(&mut self, x0: i32, x1: i32, y: i32, luma: i32) {
        let samples = ((x1 - x0) / SEARCH_STEP) as u32;
        while self
            .active
            .get(self.cursor)
            .is_some_and(|blob| blob.x1 + SEARCH_STEP < x0)
        {
            self.cursor += 1;
        }
        let joined = self.active[self.cursor..]
            .iter()
            .take_while(|blob| blob.x0 - SEARCH_STEP <= x1)
            .position(|blob| {
                (blob.luma - luma).abs() <= UNIFORM_TOLERANCE
                    && x0 <= blob.x1 + SEARCH_STEP
                    && blob.x1.max(x1) - blob.x0.min(x0) <= MAX_BLOB_WIDTH
            });
        if let Some(blob) = joined.and_then(|index| self.active.get_mut(self.cursor + index)) {
            blob.x0 = blob.x0.min(x0);
            blob.x1 = blob.x1.max(x1);
            blob.last_row = y;
            blob.samples += samples;
        } else if self.active.len() + self.born.len() < MAX_ACTIVE_BLOBS {
            self.born.push(Blob {
                luma,
                x0,
                x1,
                first_row: y,
                last_row: y,
                samples,
            });
        }
    }

    /// Closes blobs that have not continued for more than `MAX_MISSED_ROWS`
    /// sampled rows up to row `y`, turns plausible ones into candidates, and
    /// prepares the blobs for the next row.
    fn finish_row(&mut self, y: i32) {
        let mut index = 0;
        while index < self.active.len() {
            if y - self.active[index].last_row <= MAX_MISSED_ROWS * SEARCH_STEP {
                index += 1;
                continue;
            }
            let blob = self.active.swap_remove(index);
            if let Some(candidate) = blob_candidate(&blob) {
                self.candidates.push(candidate);
            }
        }
        self.active.append(&mut self.born);
        self.active.sort_unstable_by_key(|blob| blob.x0);
        self.cursor = 0;
    }

    fn finish(mut self) -> Vec<Disk> {
        for blob in self.active.drain(..) {
            if let Some(candidate) = blob_candidate(&blob) {
                self.candidates.push(candidate);
            }
        }
        self.candidates
    }
}

fn blob_candidate(blob: &Blob) -> Option<Disk> {
    let width = (blob.x1 - blob.x0) as f32;
    let height = (blob.last_row - blob.first_row + SEARCH_STEP) as f32;
    let diameter = (width + height) / 2.0;
    let r = diameter / 2.0;
    if !(MIN_RADIUS..=MAX_RADIUS).contains(&r)
        || (width - height).abs() > (3 * SEARCH_STEP) as f32 + 0.2 * diameter
    {
        return None;
    }
    // A disk fills pi/4 of its box, less the glyph cut out of it; a solid
    // square fills all of it.
    let fill = (blob.samples * (SEARCH_STEP * SEARCH_STEP) as u32) as f32 / (width * height);
    if !(0.45..=0.9).contains(&fill) {
        return None;
    }
    Some(Disk {
        cx: blob.x0 as f32 + width / 2.0 - 0.5,
        cy: blob.first_row as f32 + height / 2.0 - 0.5,
        r,
    })
}

/// Confirms that a disk of about `approx`'s size sits near its center, and
/// measures it precisely.
pub(crate) fn verify(rows: &Rows<'_>, approx: Disk) -> Option<VerifiedDisk> {
    let (first, _) = measure(rows, approx)?;
    let (disk, disk_luma) = measure(rows, first)?;
    let outer = ring(rows, disk, (disk.r * OUTER_RING).max(disk.r + 2.5));
    let contrasting = outer
        .iter()
        .flatten()
        .filter(|luma| (**luma - disk_luma).abs() >= MIN_CONTRAST)
        .count();
    if contrasting + RING_OUTLIERS < RING_SAMPLES {
        return None;
    }
    let outer_luma = median(outer.iter().flatten().copied())?;
    Some(VerifiedDisk {
        disk,
        disk_luma,
        outer_luma,
    })
}

fn measure(rows: &Rows<'_>, approx: Disk) -> Option<(Disk, i32)> {
    if !(MIN_RADIUS..=MAX_RADIUS).contains(&approx.r) {
        return None;
    }
    let inner = ring(rows, approx, approx.r * INNER_RING);
    let disk_luma = median(inner.iter().flatten().copied())?;
    let uniform = inner
        .iter()
        .flatten()
        .filter(|luma| (**luma - disk_luma).abs() <= UNIFORM_TOLERANCE)
        .count();
    if uniform + RING_OUTLIERS < RING_SAMPLES {
        return None;
    }
    let cx = approx.cx.round() as i32;
    let cy = approx.cy.round() as i32;
    let start = (approx.r * INNER_RING).round() as i32;
    let limit = start + 4;
    let left = edge(rows, (cx - start, cy), (-1, 0), disk_luma, limit)?;
    let right = edge(rows, (cx + start, cy), (1, 0), disk_luma, limit)?;
    let top = edge(rows, (cx, cy - start), (0, -1), disk_luma, limit)?;
    let bottom = edge(rows, (cx, cy + start), (0, 1), disk_luma, limit)?;
    let width = (right - left - 1) as f32;
    let height = (bottom - top - 1) as f32;
    if (width - height).abs() > 2.0_f32.max(0.12 * width.max(height)) {
        return None;
    }
    let disk = Disk {
        cx: (left + right) as f32 / 2.0,
        cy: (top + bottom) as f32 / 2.0,
        r: (width + height) / 4.0 + 0.5,
    };
    (MIN_RADIUS..=MAX_RADIUS)
        .contains(&disk.r)
        .then_some((disk, disk_luma))
}

/// Walks from `start` in `direction` and returns the coordinate of the first
/// pixel that no longer looks like the disk.
fn edge(
    rows: &Rows<'_>,
    start: (i32, i32),
    direction: (i32, i32),
    disk_luma: i32,
    limit: i32,
) -> Option<i32> {
    for step in 0..=limit {
        let x = start.0 + direction.0 * step;
        let y = start.1 + direction.1 * step;
        if (rows.luma(x, y)? - disk_luma).abs() > MIN_CONTRAST / 2 {
            return Some(if direction.0 == 0 { y } else { x });
        }
    }
    None
}

fn ring(rows: &Rows<'_>, disk: Disk, radius: f32) -> [Option<i32>; RING_SAMPLES] {
    let mut samples = [None; RING_SAMPLES];
    for (index, sample) in samples.iter_mut().enumerate() {
        let angle = index as f32 * std::f32::consts::TAU / RING_SAMPLES as f32;
        let x = (disk.cx + radius * angle.cos()).round() as i32;
        let y = (disk.cy + radius * angle.sin()).round() as i32;
        *sample = rows.luma(x, y);
    }
    samples
}

fn median(values: impl Iterator<Item = i32>) -> Option<i32> {
    let mut sorted = [0; RING_SAMPLES];
    let mut count = 0;
    for value in values.take(RING_SAMPLES) {
        sorted[count] = value;
        count += 1;
    }
    let sorted = &mut sorted[..count];
    sorted.sort_unstable();
    sorted.get(count / 2).copied()
}

/// Classifies the glyph inside a verified disk.
pub(crate) fn classify(rows: &Rows<'_>, verified: VerifiedDisk) -> GlyphReading {
    let unknown = |stats| GlyphReading { glyph: None, stats };
    let disk = verified.disk;
    let half = ((disk.r * GLYPH_BOX).floor() as usize).min((MAX_GLYPH_BOX - 1) / 2);
    let size = 2 * half + 1;
    let left = disk.cx.round() as i32 - half as i32;
    let top = disk.cy.round() as i32 - half as i32;
    let threshold = ((verified.disk_luma - verified.outer_luma).abs() / 2).max(48);

    let mut columns = [0_u32; MAX_GLYPH_BOX];
    let mut row_min = [usize::MAX; MAX_GLYPH_BOX];
    let mut row_max = [0_usize; MAX_GLYPH_BOX];
    let mut ink = 0_u32;
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (usize::MAX, 0, usize::MAX, 0);
    for j in 0..size {
        for (i, column) in columns.iter_mut().enumerate().take(size) {
            let Some(luma) = rows.luma(left + i as i32, top + j as i32) else {
                return unknown(GlyphStats::default());
            };
            if (luma - verified.disk_luma).abs() < threshold {
                continue;
            }
            ink += 1;
            *column += 1;
            row_min[j] = row_min[j].min(i);
            row_max[j] = row_max[j].max(i);
            min_x = min_x.min(i);
            max_x = max_x.max(i);
            min_y = min_y.min(j);
            max_y = max_y.max(j);
        }
    }
    if ink < 4 || (ink as usize) * 100 < size * size {
        return unknown(GlyphStats {
            ink,
            ..GlyphStats::default()
        });
    }

    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let mut group_widths = [0_usize; MAX_GLYPH_GROUPS];
    let mut group_peaks = [0_u32; MAX_GLYPH_GROUPS];
    let mut groups: usize = 0;
    let mut in_group = false;
    for column in &columns[min_x..=max_x] {
        if *column == 0 {
            in_group = false;
            continue;
        }
        if !in_group {
            in_group = true;
            groups += 1;
        }
        if let Some(slot) = groups
            .checked_sub(1)
            .filter(|slot| *slot < MAX_GLYPH_GROUPS)
        {
            group_widths[slot] += 1;
            group_peaks[slot] = group_peaks[slot].max(*column);
        }
    }
    let fill = ink as f32 / (width * height) as f32;
    let stats = GlyphStats {
        ink,
        width: width as u32,
        height: height as u32,
        groups: groups as u32,
        fill,
    };

    let r = disk.r;
    let center = half as f32;
    let bbox_cx = (min_x + max_x) as f32 / 2.0;
    let bbox_cy = (min_y + max_y) as f32 / 2.0;
    if (bbox_cx - center).abs() > 0.25 * r || (bbox_cy - center).abs() > 0.25 * r {
        return unknown(stats);
    }
    let (short, long) = (width.min(height) as f32, width.max(height) as f32);

    let glyph = if groups == 1 {
        if fill >= 0.75 && long <= 1.3 * short && short >= 3.0_f32.max(0.2 * r) {
            Some(Glyph::Stop)
        } else if is_arrow(
            &columns[min_x..=max_x],
            &row_min,
            &row_max,
            min_y..max_y + 1,
            stats,
            r,
        ) {
            Some(Glyph::Arrow)
        } else {
            None
        }
    } else if (3..=9).contains(&groups) {
        let groups = &group_widths[..groups.min(MAX_GLYPH_GROUPS)];
        let narrow = groups
            .iter()
            .all(|group_width| *group_width as f32 <= 3.0_f32.max(0.3 * width as f32));
        let tall = groups
            .iter()
            .zip(group_peaks)
            .filter(|(group_width, peak)| *peak as f32 >= 1.2 * **group_width as f32)
            .count();
        (narrow && tall >= 3 && height as f32 >= 0.25 * r).then_some(Glyph::Waveform)
    } else {
        None
    };
    GlyphReading { glyph, stats }
}

/// A stem that spans the glyph's height, narrow at the bottom, with a wide
/// symmetric chevron at the top.
fn is_arrow(
    columns: &[u32],
    row_min: &[usize],
    row_max: &[usize],
    rows: Range<usize>,
    stats: GlyphStats,
    r: f32,
) -> bool {
    let width = stats.width as f32;
    let height = stats.height as f32;
    if height < 0.4 * r || stats.fill > 0.65 {
        return false;
    }
    let Some((peak_index, peak)) = columns
        .iter()
        .copied()
        .enumerate()
        .max_by_key(|(_, count)| *count)
    else {
        return false;
    };
    let centered = (peak_index as f32 - (width - 1.0) / 2.0).abs() <= 1.0 + 0.2 * width;
    if !centered || (peak as f32) < 0.7 * height {
        return false;
    }
    let mirrored: u32 = (0..columns.len())
        .map(|index| columns[index].abs_diff(columns[columns.len() - 1 - index]))
        .sum();
    if mirrored as f32 > 0.35 * stats.ink as f32 {
        return false;
    }
    let row_width =
        |row: usize| (row_min[row] != usize::MAX).then(|| (row_max[row] - row_min[row] + 1) as f32);
    let len = rows.len();
    let stem_rows = rows.start + len * 13 / 20..rows.end;
    let head_rows = rows.start..rows.start + len / 2;
    let stem_narrow = stem_rows
        .filter_map(row_width)
        .all(|row| row <= 3.0_f32.max(0.4 * width));
    let head_wide = head_rows
        .filter_map(row_width)
        .any(|row| row >= 0.6 * width);
    stem_narrow && head_wide
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics_detection::scene::{Glyph as SceneGlyph, Scene, Theme};

    fn read(scene: &Scene) -> (Option<VerifiedDisk>, Option<GlyphReading>) {
        let frame = scene.render();
        let band = search_rows(scene.height);
        let start = band.start as usize * scene.width as usize * 4;
        let rows = Rows::new(scene.width, band.start, &frame[start..], PixelOrder::Rgba)
            .expect("whole rows");
        let found = locate(&rows);
        let reading = found.map(|verified| classify(&rows, verified));
        (found, reading)
    }

    fn assert_reads(scene: Scene, expected: Glyph) {
        let (found, reading) = read(&scene);
        let found = found.unwrap_or_else(|| panic!("button not found in {scene:?}"));
        let (cx, cy, r) = scene.button();
        assert!(
            (found.disk.cx - cx).abs() <= 1.5
                && (found.disk.cy - cy).abs() <= 1.5
                && (found.disk.r - r).abs() <= 2.0,
            "{found:?} vs {:?}",
            (cx, cy, r)
        );
        let reading = reading.expect("reading");
        assert_eq!(reading.glyph, Some(expected), "{reading:?} in {scene:?}");
    }

    #[test]
    fn reads_the_dark_theme_voice_stop_and_send_glyphs() {
        assert_reads(Scene::chat(SceneGlyph::Waveform), Glyph::Waveform);
        assert_reads(Scene::chat(SceneGlyph::Stop), Glyph::Stop);
        assert_reads(Scene::chat(SceneGlyph::Arrow), Glyph::Arrow);
    }

    #[test]
    fn reads_the_light_theme_by_contrast() {
        for (glyph, expected) in [
            (SceneGlyph::Waveform, Glyph::Waveform),
            (SceneGlyph::Stop, Glyph::Stop),
            (SceneGlyph::Arrow, Glyph::Arrow),
        ] {
            assert_reads(
                Scene {
                    theme: Theme::LIGHT,
                    ..Scene::chat(glyph)
                },
                expected,
            );
        }
    }

    #[test]
    fn follows_the_button_after_a_resize_and_a_taller_composer() {
        let resized = Scene::chat(SceneGlyph::Stop).resized(1800, 1180);
        assert_reads(resized, Glyph::Stop);
        let tall = Scene::chat(SceneGlyph::Waveform).with_draft_lines(4);
        assert_reads(tall, Glyph::Waveform);
        assert_reads(
            Scene::chat(SceneGlyph::Arrow)
                .resized(1500, 1000)
                .with_draft_lines(3),
            Glyph::Arrow,
        );
    }

    #[test]
    fn reads_other_device_scales() {
        for scale in [1.0, 1.25, 1.75, 2.5, 3.0] {
            assert_reads(Scene::chat(SceneGlyph::Stop).scaled(scale), Glyph::Stop);
            assert_reads(
                Scene::chat(SceneGlyph::Waveform).scaled(scale),
                Glyph::Waveform,
            );
            assert_reads(Scene::chat(SceneGlyph::Arrow).scaled(scale), Glyph::Arrow);
        }
    }

    #[test]
    fn finds_the_button_below_dense_text() {
        for theme in [Theme::DARK, Theme::LIGHT] {
            let scene = Scene {
                theme,
                ..Scene::chat(SceneGlyph::Waveform)
            };
            let mut frame = scene.render();
            let width = scene.width as usize;
            let (_, composer_top, _, _) = scene.composer();
            // 2 px text strokes every 6 px, in 22 px lines across most of the page.
            for y in search_rows(scene.height).start as usize..composer_top as usize - 8 {
                if y % 56 >= 22 {
                    continue;
                }
                for x in (width / 5..width * 4 / 5).filter(|x| x % 6 < 2) {
                    let offset = (y * width + x) * 4;
                    frame[offset..offset + 3].copy_from_slice(&theme.text);
                }
            }
            let band = search_rows(scene.height);
            let start = band.start as usize * width * 4;
            let rows = Rows::new(scene.width, band.start, &frame[start..], PixelOrder::Rgba)
                .expect("rows");
            let verified = locate(&rows).expect("button found below dense text");
            assert_eq!(classify(&rows, verified).glyph, Some(Glyph::Waveform));
        }
    }

    #[test]
    fn finds_no_button_without_a_disk() {
        let (found, _) = read(&Scene::chat(SceneGlyph::NoButton));
        assert_eq!(found, None);
        let (found, _) = read(&Scene::chat(SceneGlyph::SquareButton));
        assert_eq!(found, None, "a rounded square is not the button");
    }

    #[test]
    fn leaves_unrecognized_glyphs_unknown() {
        for glyph in [SceneGlyph::Empty, SceneGlyph::Cross, SceneGlyph::Ring] {
            let scene = Scene::chat(glyph);
            let (found, reading) = read(&scene);
            assert!(found.is_some(), "disk should still be found for {glyph:?}");
            assert_eq!(reading.and_then(|reading| reading.glyph), None, "{glyph:?}");
        }
    }

    #[test]
    fn verifies_a_cached_disk_from_its_crop_only() {
        let scene = Scene::chat(SceneGlyph::Stop);
        let frame = scene.render();
        let (cx, cy, r) = scene.button();
        let crop = Disk { cx, cy, r }
            .crop(scene.width, scene.height)
            .expect("crop");
        let start = crop.y0 as usize * scene.width as usize * 4;
        let end = crop.y1 as usize * scene.width as usize * 4;
        let rows =
            Rows::new(scene.width, crop.y0, &frame[start..end], PixelOrder::Rgba).expect("rows");
        let verified = verify(
            &rows,
            Disk {
                cx: cx + 2.0,
                cy: cy - 1.0,
                r: r - 1.0,
            },
        )
        .expect("verified from a slightly stale cache");
        assert_eq!(classify(&rows, verified).glyph, Some(Glyph::Stop));
        assert_eq!(
            verify(
                &rows,
                Disk {
                    cx: cx - 3.0 * r,
                    cy,
                    r
                }
            ),
            None
        );
    }

    #[test]
    fn bgra_frames_read_like_rgba_frames() {
        let scene = Scene::chat(SceneGlyph::Arrow);
        let mut frame = scene.render();
        for pixel in frame.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let band = search_rows(scene.height);
        let start = band.start as usize * scene.width as usize * 4;
        let rows =
            Rows::new(scene.width, band.start, &frame[start..], PixelOrder::Bgra).expect("rows");
        let verified = locate(&rows).expect("found");
        assert_eq!(classify(&rows, verified).glyph, Some(Glyph::Arrow));
    }

    #[test]
    fn rows_reject_partial_rows_and_out_of_range_rects() {
        assert!(Rows::new(2, 0, &[0; 7], PixelOrder::Rgba).is_none());
        assert!(Rows::new(0, 0, &[], PixelOrder::Rgba).is_none());
        let bytes = [0_u8; 16];
        let rows = Rows::new(2, 5, &bytes, PixelOrder::Rgba).expect("two rows");
        assert_eq!(rows.rows(), 5..7);
        let rect = |x0, y0, x1, y1| PixelRect { x0, y0, x1, y1 };
        assert!(rows.rect_bytes(rect(0, 5, 2, 7)).is_some());
        assert!(rows.rect_bytes(rect(0, 4, 2, 7)).is_none());
        assert!(rows.rect_bytes(rect(0, 5, 3, 7)).is_none());
        assert_eq!(search_rows(1000), 700..1000);
    }
}
