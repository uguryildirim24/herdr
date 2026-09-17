//! Debug evidence for button detection on live panes.
//!
//! When the server starts with `HERDR_DEBUG_PAGE_BUTTON_DIR` set, every change
//! in what a pane's checks observe writes a PNG of the pixels involved (the
//! crop around the button, or the searched band at quarter resolution when no
//! button was found) and appends one line to `classifications.log` in that
//! directory. Nothing is written while the observation stays the same.

use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use super::button::{GlyphReading, Rows, VerifiedDisk};
use super::Observed;

pub(crate) const DEBUG_DIR_ENV: &str = "HERDR_DEBUG_PAGE_BUTTON_DIR";
/// Downscale factor of the band image written when no button is found.
const NOT_FOUND_DOWNSCALE: u32 = 4;

pub(super) struct DebugDump {
    dir: PathBuf,
}

impl DebugDump {
    pub(super) fn from_env() -> Option<Self> {
        let dir = std::env::var_os(DEBUG_DIR_ENV).filter(|dir| !dir.is_empty())?;
        let dir = PathBuf::from(dir);
        if let Err(err) = fs::create_dir_all(&dir) {
            tracing::warn!(path = %dir.display(), err = %err, "cannot create pane graphics button debug directory");
            return None;
        }
        tracing::info!(path = %dir.display(), "writing pane graphics button debug evidence");
        Some(Self { dir })
    }

    pub(super) fn found(
        &self,
        slot: u64,
        frame: (u32, u32),
        observed: Observed,
        verified: VerifiedDisk,
        reading: GlyphReading,
        rows: &Rows<'_>,
    ) {
        let disk = verified.disk;
        let stats = reading.stats;
        let details = format!(
            "disk=({:.1},{:.1}) r={:.1} disk_luma={} outer_luma={} ink={} glyph_box={}x{} groups={} fill={:.2}",
            disk.cx,
            disk.cy,
            disk.r,
            verified.disk_luma,
            verified.outer_luma,
            stats.ink,
            stats.width,
            stats.height,
            stats.groups,
            stats.fill
        );
        let image = disk.crop(frame.0, frame.1).and_then(|crop| {
            let width = crop.x1 - crop.x0;
            let height = crop.y1 - crop.y0;
            let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
            for y in crop.y0..crop.y1 {
                for x in crop.x0..crop.x1 {
                    pixels.extend_from_slice(&rows.rgba(x, y)?);
                }
            }
            Some((width, height, pixels))
        });
        self.write(slot, frame, observed, &details, image);
    }

    pub(super) fn not_found(&self, slot: u64, frame: (u32, u32), rows: &Rows<'_>) {
        let band = rows.rows();
        let width = frame.0 / NOT_FOUND_DOWNSCALE;
        let height = band.len() as u32 / NOT_FOUND_DOWNSCALE;
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
        let complete = (0..height).all(|row| {
            (0..width).all(|column| {
                let y = band.start + row * NOT_FOUND_DOWNSCALE;
                rows.rgba(column * NOT_FOUND_DOWNSCALE, y)
                    .map(|pixel| pixels.extend_from_slice(&pixel))
                    .is_some()
            })
        });
        let image = (complete && width > 0 && height > 0).then_some((width, height, pixels));
        let details = format!(
            "band_rows={}..{} image_downscale={NOT_FOUND_DOWNSCALE}",
            band.start, band.end
        );
        self.write(slot, frame, Observed::NotFound, &details, image);
    }

    fn write(
        &self,
        slot: u64,
        frame: (u32, u32),
        observed: Observed,
        details: &str,
        image: Option<(u32, u32, Vec<u8>)>,
    ) {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or_default();
        let label = observed.label();
        let image_name = image.map(|(width, height, pixels)| {
            let name = format!("pane-{slot}-{millis}-{label}.png");
            if let Err(err) = write_png(&self.dir.join(&name), width, height, &pixels) {
                tracing::warn!(err = %err, "failed to write pane graphics button debug image");
            }
            name
        });
        let line = format!(
            "{millis} slot={slot} frame={}x{} observed={label} {details} image={}\n",
            frame.0,
            frame.1,
            image_name.as_deref().unwrap_or("-")
        );
        let appended = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("classifications.log"))
            .and_then(|mut log| log.write_all(line.as_bytes()));
        if let Err(err) = appended {
            tracing::warn!(err = %err, "failed to append pane graphics button debug log");
        }
    }
}

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> io::Result<()> {
    let file = BufWriter::new(fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(rgba))
        .map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics_detection::button::{self, PixelOrder};
    use crate::graphics_detection::scene::{Glyph as SceneGlyph, Scene};

    #[test]
    fn writes_the_button_crop_and_a_log_line() {
        let dir =
            std::env::temp_dir().join(format!("herdr-button-debug-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let dump = DebugDump { dir: dir.clone() };

        let scene = Scene::chat(SceneGlyph::Stop);
        let frame = scene.render();
        let band = button::search_rows(scene.height);
        let start = band.start as usize * scene.width as usize * 4;
        let rows =
            Rows::new(scene.width, band.start, &frame[start..], PixelOrder::Rgba).expect("rows");
        let verified = button::locate(&rows).expect("found");
        let reading = button::classify(&rows, verified);
        dump.found(
            7,
            (scene.width, scene.height),
            Observed::Unknown,
            verified,
            reading,
            &rows,
        );
        dump.not_found(7, (scene.width, scene.height), &rows);

        let log = fs::read_to_string(dir.join("classifications.log")).expect("log");
        let lines: Vec<_> = log.lines().collect();
        assert_eq!(lines.len(), 2, "{log}");
        assert!(lines[0].contains("slot=7 frame=2400x1600 observed=unknown disk=("));
        assert!(lines[1].contains("observed=not-found band_rows=1120..1600"));
        let pngs = fs::read_dir(&dir)
            .expect("list")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
            .count();
        assert_eq!(pngs, 2);
        let _ = fs::remove_dir_all(&dir);
    }
}
