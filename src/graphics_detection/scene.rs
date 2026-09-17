//! Synthetic chat page frames for classifier tests, modeled on the ChatGPT web
//! composer: a rounded bar with "+", placeholder text, a model label, a mic
//! icon, and a 36 CSS px round action button at its bottom-right corner.

/// Colors of one page theme.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Theme {
    page: [u8; 3],
    composer: [u8; 3],
    border: [u8; 3],
    pub(crate) text: [u8; 3],
    muted: [u8; 3],
    disk: [u8; 3],
    glyph: [u8; 3],
}

impl Theme {
    pub(crate) const DARK: Theme = Theme {
        page: [33, 33, 33],
        composer: [48, 48, 48],
        border: [60, 60, 60],
        text: [236, 236, 236],
        muted: [160, 160, 160],
        disk: [255, 255, 255],
        glyph: [0, 0, 0],
    };
    pub(crate) const LIGHT: Theme = Theme {
        page: [255, 255, 255],
        composer: [255, 255, 255],
        border: [226, 226, 226],
        text: [13, 13, 13],
        muted: [110, 110, 110],
        disk: [0, 0, 0],
        glyph: [255, 255, 255],
    };
}

/// What the scene draws at the button position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    Waveform,
    Stop,
    Arrow,
    /// A disk with no glyph.
    Empty,
    /// A disk with an "x".
    Cross,
    /// A disk with a ring outline.
    Ring,
    /// No button at all.
    NoButton,
    /// A rounded square instead of a disk.
    SquareButton,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Scene {
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Device pixels per CSS pixel.
    pub(crate) scale: f32,
    pub(crate) theme: Theme,
    pub(crate) glyph: Glyph,
    pub(crate) draft_lines: u32,
}

const BUTTON_CSS: f32 = 36.0;

type Bounds = (f32, f32, f32, f32);

impl Scene {
    pub(crate) fn chat(glyph: Glyph) -> Self {
        Self {
            width: 2400,
            height: 1600,
            scale: 2.0,
            theme: Theme::DARK,
            glyph,
            draft_lines: 1,
        }
    }

    pub(crate) fn resized(self, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..self
        }
    }

    pub(crate) fn scaled(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    pub(crate) fn with_draft_lines(self, draft_lines: u32) -> Self {
        Self {
            draft_lines,
            ..self
        }
    }

    /// Composer rectangle in device pixels: left, top, right, bottom.
    pub(crate) fn composer(&self) -> Bounds {
        let s = self.scale;
        let view_width = self.width as f32 / s;
        let view_height = self.height as f32 / s;
        let width = 768.0_f32.min(view_width - 32.0);
        let left = (view_width - width) / 2.0;
        let bottom = view_height - 36.0;
        let height = 56.0 + 24.0 * self.draft_lines.saturating_sub(1) as f32;
        (
            left * s,
            (bottom - height) * s,
            (left + width) * s,
            bottom * s,
        )
    }

    /// Button center (pixel-center coordinates) and radius in device pixels.
    pub(crate) fn button(&self) -> (f32, f32, f32) {
        let (x, y, r) = self.button_geometry();
        (x - 0.5, y - 0.5, r)
    }

    fn button_geometry(&self) -> (f32, f32, f32) {
        let s = self.scale;
        let (_, _, right, bottom) = self.composer();
        let r = BUTTON_CSS / 2.0 * s;
        (right - 10.0 * s - r, bottom - 10.0 * s - r, r)
    }

    pub(crate) fn render(&self) -> Vec<u8> {
        let theme = self.theme;
        let mut canvas = Canvas::new(self.width, self.height, theme.page);
        let s = self.scale;
        let (left, top, right, bottom) = self.composer();

        // Earlier chat text, partly inside the searched bottom band.
        let mut line_top = top - 40.0 * s;
        while line_top > self.height as f32 * 0.55 {
            canvas.words(
                (
                    left + 20.0 * s,
                    line_top,
                    left + 720.0 * s,
                    line_top + 11.0 * s,
                ),
                s,
                theme.text,
            );
            line_top -= 28.0 * s;
        }
        // Scroll-to-bottom button: a page-colored circle with a border.
        let (scroll_x, scroll_y) = ((left + right) / 2.0, top - 24.0 * s);
        canvas.fill(
            theme.border,
            around(scroll_x, scroll_y, 17.0 * s),
            |x, y| {
                let d = ((x - scroll_x).powi(2) + (y - scroll_y).powi(2)).sqrt();
                (15.0 * s..=16.0 * s).contains(&d)
            },
        );

        // Composer bar with a border.
        let radius = if self.draft_lines > 1 { 24.0 } else { 28.0 } * s;
        let outer = (left, top, right, bottom);
        canvas.fill(theme.border, outer, |x, y| {
            rounded_rect(x, y, outer, radius)
        });
        let inner = (left + s, top + s, right - s, bottom - s);
        canvas.fill(theme.composer, inner, |x, y| {
            rounded_rect(x, y, inner, radius - s)
        });

        // "+" in the last row, placeholder text, the model label, and a mic.
        let (bx, by, br) = self.button_geometry();
        let plus_x = left + 26.0 * s;
        canvas.segment(
            theme.text,
            (plus_x - 7.0 * s, by),
            (plus_x + 7.0 * s, by),
            s,
        );
        canvas.segment(
            theme.text,
            (plus_x, by - 7.0 * s),
            (plus_x, by + 7.0 * s),
            s,
        );
        canvas.words(
            (
                left + 52.0 * s,
                by - 6.0 * s,
                left + 162.0 * s,
                by + 6.0 * s,
            ),
            s,
            theme.muted,
        );
        canvas.words(
            (bx - 120.0 * s, by - 5.0 * s, bx - 80.0 * s, by + 5.0 * s),
            s,
            theme.muted,
        );
        let mic_x = bx - 46.0 * s;
        let capsule = (mic_x - 4.0 * s, by - 8.0 * s, mic_x + 4.0 * s, by + 3.0 * s);
        let hollow = (mic_x - 2.5 * s, by - 6.5 * s, mic_x + 2.5 * s, by + 1.5 * s);
        canvas.fill(theme.muted, capsule, |x, y| {
            rounded_rect(x, y, capsule, 4.0 * s) && !rounded_rect(x, y, hollow, 2.5 * s)
        });
        canvas.segment(
            theme.muted,
            (mic_x, by + 4.0 * s),
            (mic_x, by + 8.0 * s),
            0.75 * s,
        );
        // The note under the composer.
        let note_left = (self.width as f32 - 320.0 * s) / 2.0;
        canvas.words(
            (
                note_left,
                bottom + 14.0 * s,
                note_left + 320.0 * s,
                bottom + 23.0 * s,
            ),
            s,
            theme.muted,
        );

        let disk_bounds = around(bx, by, br + 1.0);
        match self.glyph {
            Glyph::NoButton => {}
            Glyph::SquareButton => {
                let square = (bx - br, by - br, bx + br, by + br);
                canvas.fill(theme.disk, disk_bounds, |x, y| {
                    rounded_rect(x, y, square, 6.0 * s)
                });
                self.waveform(&mut canvas, bx, by);
            }
            glyph => {
                canvas.fill(theme.disk, disk_bounds, |x, y| {
                    (x - bx).powi(2) + (y - by).powi(2) <= br * br
                });
                match glyph {
                    Glyph::Waveform => self.waveform(&mut canvas, bx, by),
                    Glyph::Stop => {
                        let square = around(bx, by, 5.0 * s);
                        canvas.fill(theme.glyph, square, |x, y| {
                            rounded_rect(x, y, square, 2.0 * s)
                        });
                    }
                    Glyph::Arrow => {
                        let tip = (bx, by - 7.0 * s);
                        canvas.segment(theme.glyph, (bx, by + 7.0 * s), tip, s);
                        canvas.segment(theme.glyph, (bx - 6.0 * s, by - 1.0 * s), tip, s);
                        canvas.segment(theme.glyph, (bx + 6.0 * s, by - 1.0 * s), tip, s);
                    }
                    Glyph::Cross => {
                        let d = 6.0 * s;
                        canvas.segment(theme.glyph, (bx - d, by - d), (bx + d, by + d), s);
                        canvas.segment(theme.glyph, (bx - d, by + d), (bx + d, by - d), s);
                    }
                    Glyph::Ring => canvas.fill(theme.glyph, around(bx, by, 9.0 * s), |x, y| {
                        let d = ((x - bx).powi(2) + (y - by).powi(2)).sqrt();
                        (6.0 * s..=8.0 * s).contains(&d)
                    }),
                    _ => {}
                }
            }
        }
        canvas.pixels
    }

    fn waveform(&self, canvas: &mut Canvas, bx: f32, by: f32) {
        let s = self.scale;
        for (offset, height) in [
            (-8.0, 6.0),
            (-4.0, 10.0),
            (0.0, 14.0),
            (4.0, 10.0),
            (8.0, 6.0),
        ] {
            let x = bx + offset * s;
            let half = height / 2.0 * s;
            let bar = (x - s, by - half, x + s, by + half);
            canvas.fill(self.theme.glyph, bar, |px, py| rounded_rect(px, py, bar, s));
        }
    }
}

fn around(x: f32, y: f32, half: f32) -> Bounds {
    (x - half, y - half, x + half, y + half)
}

fn rounded_rect(x: f32, y: f32, (left, top, right, bottom): Bounds, radius: f32) -> bool {
    let cx = (left + right) / 2.0;
    let cy = (top + bottom) / 2.0;
    let radius = radius.min((right - left) / 2.0).min((bottom - top) / 2.0);
    let qx = (x - cx).abs() - ((right - left) / 2.0 - radius);
    let qy = (y - cy).abs() - ((bottom - top) / 2.0 - radius);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) <= radius
}

struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(width: u32, height: u32, color: [u8; 3]) -> Self {
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
        for _ in 0..width as usize * height as usize {
            pixels.extend_from_slice(&[color[0], color[1], color[2], 255]);
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Blends `color` over the pixels of `bounds` by 4x4 supersampled coverage
    /// of `inside`.
    fn fill(&mut self, color: [u8; 3], bounds: Bounds, inside: impl Fn(f32, f32) -> bool) {
        let clamp = |value: f32, limit: u32| value.clamp(0.0, limit as f32) as u32;
        let (x0, x1) = (
            clamp(bounds.0.floor() - 1.0, self.width),
            clamp(bounds.2.ceil() + 1.0, self.width),
        );
        let (y0, y1) = (
            clamp(bounds.1.floor() - 1.0, self.height),
            clamp(bounds.3.ceil() + 1.0, self.height),
        );
        for py in y0..y1 {
            for px in x0..x1 {
                let mut covered = 0;
                for j in 0..4 {
                    for i in 0..4 {
                        let x = px as f32 + (i as f32 + 0.5) / 4.0;
                        let y = py as f32 + (j as f32 + 0.5) / 4.0;
                        if inside(x, y) {
                            covered += 1;
                        }
                    }
                }
                if covered == 0 {
                    continue;
                }
                let alpha = covered as f32 / 16.0;
                let offset = (py as usize * self.width as usize + px as usize) * 4;
                for (target, new) in self.pixels[offset..offset + 3].iter_mut().zip(color) {
                    let old = f32::from(*target);
                    *target = (old + (f32::from(new) - old) * alpha).round() as u8;
                }
            }
        }
    }

    fn segment(&mut self, color: [u8; 3], a: (f32, f32), b: (f32, f32), half_width: f32) {
        let bounds = (
            a.0.min(b.0) - half_width,
            a.1.min(b.1) - half_width,
            a.0.max(b.0) + half_width,
            a.1.max(b.1) + half_width,
        );
        self.fill(color, bounds, |x, y| {
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let len = dx * dx + dy * dy;
            let t = if len == 0.0 {
                0.0
            } else {
                (((x - a.0) * dx + (y - a.1) * dy) / len).clamp(0.0, 1.0)
            };
            let (nx, ny) = (a.0 + t * dx - x, a.1 + t * dy - y);
            nx * nx + ny * ny <= half_width * half_width
        });
    }

    /// Letter-shaped blocks filling one text line: 6 CSS px letters with 2 px
    /// gaps, and every fifth letter slot left empty as a word gap.
    fn words(&mut self, bounds: Bounds, s: f32, color: [u8; 3]) {
        let (left, top, right, bottom) = bounds;
        self.fill(color, bounds, move |x, y| {
            if x < left || x >= right || y < top || y >= bottom {
                return false;
            }
            let column = ((x - left) / s) as u32;
            column % 8 < 6 && (column / 8) % 5 != 4
        });
    }
}
