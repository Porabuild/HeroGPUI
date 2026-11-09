//! Headless hover/press interaction sweep across every gallery component
//! example.
//!
//! Each `(page, section)` example is mounted in component-preview mode and
//! the pointer is swept over a grid of candidate targets discovered from
//! the rest frame's painted primitives. Three checks run per target:
//!
//! 1. **Return to rest** — after every target is left and the frame
//!    settles, the scene must composite identically to the mount's rest
//!    frame (guards translucent fills painted twice and stuck highlights).
//! 2. **Monotonic leave** — two frames after leaving mid-fade, the
//!    composite must never sit farther from rest than the frame before it
//!    (guards the snap-to-hovered-colour regression).
//! 3. **Fill over text** — a new solid quad painting after an underline
//!    and covering its centre is a fill painted over a label (the bug that
//!    hit Pagination, Tabs, Calendar, RangeCalendar and ColorField).
//!
//! Two further checks guard the frame settling: one reports an interaction
//! whose redraws never converge (hover feedback that resizes its own
//! hitbox), and one reports a settled frame that still drifts a second
//! later.
//!
//! What this cannot see: glyph pixels, SVG icons and shadows are not
//! observable in test renders, so "over text" means over the underline
//! primitives that stand in for text runs.
//!
//! Checks, by the number a finding reports: 1 return to rest, 2 monotonic
//! leave, 3 fill over text, 4 redraws did not settle, 5 the settled frame
//! drifted over a further second. [`EXPECTED`] lists the findings triaged
//! as legitimate; a matching finding is suppressed and an exemption that
//! matches nothing on an example the shard swept fails as a stale
//! exemption (check 6).

mod scene;

use std::collections::HashSet;
use std::time::{Duration, Instant};

use gpui::{point, px, AppContext, Pixels, Point, TestAppContext, WindowHandle};
use herogpui_theme::ThemeProvider;

use self::scene::Snapshot;
use crate::app::{sweep, Gallery};
use crate::control::{self, sweep_hooks, FrameAck, Request, Step};
use crate::pages::{nav_sections, Page};

const SHARD_COUNT: usize = 8;
/// settle(): simulated milliseconds before a quiet frame counts as rest.
const SETTLE_MIN_MS: u64 = 3_000;
/// settle() gives up past this so perpetual animations cannot hang a shard;
/// the mount's self-check then routes the example to reduced motion.
const SETTLE_CAP_MS: u64 = 8_000;
/// Discovery grid pitch in logical px.
const GRID_STEP: f32 = 16.;
/// Discovery bbox padding in logical px.
const BBOX_PAD: f32 = 32.;
/// An inert point on the preview stage; "leave" always moves here rather
/// than dispatching `MouseExited`.
const EMPTY: (f32, f32) = (4., 4.);
/// Draws allowed inside one `update_window` before the breaker judges the
/// interaction non-converging and parks the pointer. Legitimate updates
/// redraw a handful of times; a hover-feedback oscillation runs to
/// thousands.
const DRAW_BUDGET: usize = 128;
/// Pseudo-check number for the non-convergence finding (the numbered checks
/// are 1-3); reported under the pass it interrupted.
const CHECK_NONCONVERGENCE: u8 = 4;
/// The settled post-hover frame drifted over a further simulated second —
/// reported under its own label rather than as check 1.
const CHECK_CONVERGENCE: u8 = 5;
/// An [`EXPECTED`] entry matched no finding on an example the shard swept.
const CHECK_STALE_EXEMPTION: u8 = 6;

/// Nav sections that are documentation chrome, not component examples.
fn excluded_section(title: &str) -> bool {
    matches!(title, "Overview" | "Versions" | "Getting Started")
}

/// A finding triaged as legitimate and suppressed. `reason` is the evidence
/// for the verdict; it is required so an exemption always says why. An entry
/// whose `(page, heading)` this shard swept but which never matched a finding
/// is itself a failure — exemptions must not outlive the behaviour they
/// excuse.
struct Expected {
    page: &'static str,
    heading: &'static str,
    check: u8,
    pass: &'static str,
    reason: &'static str,
}

const EXPECTED: &[Expected] = &[Expected {
    page: "Tabs",
    heading: "Wrapping Labels",
    check: 3,
    pass: "hover",
    reason: "the covered underlines are lines 4-5 of the previous tab's \
                 wrapped label spilling into the hovered tab; HeroUI's \
                 .tabs__tab is relative + z-index 1, so a later tab paints its \
                 hover fill over earlier overflow in v3 too — legitimate paint \
                 order, not a fill-over-text regression",
}];

struct Finding {
    page: &'static str,
    heading: String,
    pass: &'static str,
    check: u8,
    point: Option<(f32, f32)>,
    detail: String,
}

struct Driver<'a> {
    cx: &'a mut TestAppContext,
    window: WindowHandle<Gallery>,
    /// Window scale factor; painted primitives are in scaled pixels.
    scale: f32,
    /// Window size in logical pixels.
    window_size: gpui::Size<Pixels>,
    shard: usize,
    findings: Vec<Finding>,
    skips: Vec<String>,
    fallbacks: Vec<String>,
    /// (example name, wall ms) for the slowest-10 report.
    timings: Vec<(String, f64)>,
    candidates_total: usize,
    candidates_max: usize,
    examples: usize,
    draws_start: usize,
    /// `HEROGPUI_SWEEP_DEBUG`, read once.
    debug: bool,
    /// Latched when the draw-limit breaker parked the pointer mid-update:
    /// the interaction did not converge and is reported as a finding.
    nonconverged: bool,
    /// Dedup so one example reports its first non-convergence once, then
    /// abandons the remaining passes.
    nonconverged_reported: bool,
    /// One slot per [`EXPECTED`] entry, set when a finding matched it.
    expected_hits: Vec<bool>,
    /// `(page, heading)` of every example this shard actually swept, for the
    /// stale-exemption check at finish.
    swept_examples: HashSet<(String, String)>,
    /// Examples that already logged a reduced-motion fallback (each mounts
    /// twice — hover pass and press pass — but reports once).
    fallback_reported: HashSet<String>,
    sim_ms: u64,
    started: Instant,
    page_filter: Option<String>,
    section_filter: Option<String>,
}

macro_rules! sweep_shard {
    ($name:ident, $shard:expr) => {
        #[gpui::test]
        fn $name(cx: &mut TestAppContext) {
            run_shard($shard, cx);
        }
    };
}

sweep_shard!(sweep_shard_0, 0);
sweep_shard!(sweep_shard_1, 1);
sweep_shard!(sweep_shard_2, 2);
sweep_shard!(sweep_shard_3, 3);
sweep_shard!(sweep_shard_4, 4);
sweep_shard!(sweep_shard_5, 5);
sweep_shard!(sweep_shard_6, 6);
sweep_shard!(sweep_shard_7, 7);

fn run_shard(shard: usize, cx: &mut TestAppContext) {
    sweep_hooks::set_exact(None);
    sweep::set_underline_text(true);
    let window = open_window(cx);
    // Where the draw-limit breaker parks the pointer: the same inert point
    // every "leave" uses, in the window-pixel space `dispatch_event` takes.
    sweep::set_park_point(point(px(EMPTY.0), px(EMPTY.1)));
    let (scale, window_size) = cx
        .update_window(window.into(), |_, window, _| {
            (window.scale_factor(), window.viewport_size())
        })
        .expect("window alive");
    let mut driver = Driver {
        cx,
        window,
        scale,
        window_size,
        shard,
        findings: Vec::new(),
        skips: Vec::new(),
        fallbacks: Vec::new(),
        timings: Vec::new(),
        candidates_total: 0,
        candidates_max: 0,
        examples: 0,
        draws_start: sweep::draws(),
        debug: std::env::var_os("HEROGPUI_SWEEP_DEBUG").is_some(),
        nonconverged: false,
        nonconverged_reported: false,
        expected_hits: vec![false; EXPECTED.len()],
        swept_examples: HashSet::new(),
        fallback_reported: HashSet::new(),
        sim_ms: 0,
        started: Instant::now(),
        page_filter: std::env::var("HEROGPUI_SWEEP_PAGE").ok(),
        section_filter: std::env::var("HEROGPUI_SWEEP_SECTION").ok(),
    };

    let mut global_index = 0usize;
    for section in nav_sections() {
        if excluded_section(section.title) {
            continue;
        }
        for page in section.items {
            // A page filter is for focused runs: skip recording every other
            // page's headings so a single-example probe stays cheap. Shard
            // assignment is unaffected — filtered runs are debugging aids,
            // not the sharded sweep, and every shard applies the same filter.
            if driver
                .page_filter
                .as_deref()
                .is_some_and(|filter| page.title() != filter)
            {
                continue;
            }
            let headings = driver.record_headings(page);
            for heading in headings {
                if global_index % SHARD_COUNT == shard {
                    driver.sweep_example(page, heading);
                }
                global_index += 1;
            }
        }
    }
    driver.finish();
}

fn open_window(cx: &mut TestAppContext) -> WindowHandle<Gallery> {
    cx.update(|cx| {
        ThemeProvider::init(cx);
        control::set_section_filter("", cx);
        control::set_specimen_filter(None, cx);
        control::set_preview_only(true, cx);
        cx.set_reduce_motion(false);
    });
    cx.open_window(gpui::size(px(1280.), px(1000.)), |_, cx| {
        let mut gallery = Gallery::new(cx);
        gallery.set_initial_page(Page::Button);
        gallery
    })
}

impl Driver<'_> {
    /// Runs `f` inside `update_window` with the draw-limit breaker armed, so
    /// a non-converging interaction (hover feedback that resizes its own
    /// hitbox and oscillates `on_hover` forever) parks the pointer instead of
    /// spinning `flush_effects`; the latch folds into `self.nonconverged`.
    fn window_update<T>(
        &mut self,
        f: impl FnOnce(gpui::AnyView, &mut gpui::Window, &mut gpui::App) -> T,
    ) -> T {
        sweep::arm_draw_limit(sweep::draws() + DRAW_BUDGET);
        let result = self
            .cx
            .update_window(self.window.into(), f)
            .expect("window alive");
        sweep::disarm_draw_limit();
        if sweep::take_nonconverged() {
            self.nonconverged = true;
        }
        result
    }

    /// Advances the executor clock, runs parked work, then delivers one
    /// frame's worth of `on_next_frame` callbacks. Returns how many ran.
    fn tick(&mut self, ms: u64) -> usize {
        if ms > 0 {
            self.cx.executor().advance_clock(Duration::from_millis(ms));
            self.sim_ms += ms;
        }
        self.cx.run_until_parked();
        let ran = self.window_update(|_, window, cx| window.simulate_next_frame(cx));
        self.cx.run_until_parked();
        ran
    }

    /// Drives frames until the scene has been quiet for [`SETTLE_MIN_MS`]
    /// simulated time, or animations prove perpetual (capped). Returns
    /// `true` when the last simulated frames ran no callbacks — a periodic
    /// animation never does, so this is the deterministic "still animating"
    /// signal the mount self-check needs (frame sampling can coincide with
    /// an animation's period).
    fn settle(&mut self) -> bool {
        let start = self.sim_ms;
        loop {
            // Animation progress is clock-driven, so a large step lands on the
            // same endpoint as small ones; only the callback count matters
            // here, and it is independent of the step size.
            let last = self.tick(250);
            let elapsed = self.sim_ms - start;
            if elapsed >= SETTLE_MIN_MS && last == 0 {
                return true;
            }
            if elapsed >= SETTLE_CAP_MS {
                return false;
            }
        }
    }

    fn step(&mut self, step: &Step) {
        self.window_update(|_, window, cx| control::dispatch_step(step, window, cx))
            .expect("input step dispatches");
    }

    fn at(&self, target: (f32, f32)) -> Point<Pixels> {
        point(px(target.0), px(target.1))
    }

    fn move_to(&mut self, point: (f32, f32)) {
        self.step(&Step::Move(self.at(point)));
    }

    fn down(&mut self, point: (f32, f32)) {
        self.step(&Step::Down(self.at(point)));
    }

    fn up(&mut self, point: (f32, f32)) {
        self.step(&Step::Up(self.at(point)));
    }

    fn click(&mut self, point: (f32, f32)) {
        self.step(&Step::Click(self.at(point)));
    }

    fn snapshot(&mut self) -> Snapshot {
        let scale = self.scale;
        self.window_update(|_, window, _| Snapshot::capture(window, scale))
    }

    /// Blurs the window and hides focus rings so carets and focus-visible
    /// highlights cannot blink through the sweep.
    fn blur_quiet(&mut self) {
        self.window_update(|_, window, cx| window.blur(cx));
        self.cx
            .update(|cx| herogpui_components::extend::set_focus_visible(false, cx));
    }

    /// Records every section heading `page` renders in preview mode. The
    /// recording hook makes `include_section` collect headings and reject
    /// them all, so the page draws once with an empty body.
    fn record_headings(&mut self, page: Page) -> Vec<String> {
        sweep_hooks::set_exact(None);
        sweep_hooks::start_recording();
        let request = Request::parse(&format!(
            "seq=sweep-record\npage={}\nreset=1\npreview=component\ntheme=light\noverlays=0",
            page.title()
        ))
        .expect("record request parses");
        let window = self.window;
        let applied = self
            .cx
            .update(|cx| control::apply(request, window, FrameAck::default(), cx));
        let headings = sweep_hooks::take_headings();
        assert!(applied, "apply failed while recording {}", page.title());
        assert!(
            !headings.is_empty(),
            "page {} yielded no component sections",
            page.title()
        );
        let mut seen = HashSet::new();
        for heading in &headings {
            assert!(
                seen.insert(heading.as_str()),
                "page {} records heading {heading:?} twice",
                page.title()
            );
        }
        headings
    }

    /// Mounts one example: exact-heading preview, pointer parked, blurred,
    /// reduced-motion per `reduce`. Returns `None` when the heading did not
    /// mount (already reported).
    fn try_mount(&mut self, page: Page, heading: &str, reduce: bool) -> Option<()> {
        sweep_hooks::set_exact(Some(heading));
        let motion = if reduce { "reduce" } else { "full" };
        let request = Request::parse(&format!(
            "seq=sweep\npage={}\nreset=1\npreview=component\nmotion={motion}\ntheme=light\noverlays=0",
            page.title()
        ))
        .expect("mount request parses");
        let window = self.window;
        let applied = self
            .cx
            .update(|cx| control::apply(request, window, FrameAck::default(), cx));
        if !applied || !sweep_hooks::exact_hit() {
            eprintln!(
                "sweep: {}/{} did not mount (apply={}, exact_hit={})",
                page.title(),
                heading,
                applied,
                sweep_hooks::exact_hit()
            );
            return None;
        }
        self.move_to(EMPTY);
        self.blur_quiet();
        Some(())
    }

    /// Mounts an example and proves its rest frame is stable: the mount
    /// settle and a `tick(1000) + settle` self-check must both end quiet and
    /// composite identically. Full motion is tried first; an unstable rest
    /// retries under reduced motion (recorded as a fallback), and a
    /// still-unstable rest skips the example.
    fn mount_example(&mut self, page: Page, heading: &str) -> Option<(Snapshot, bool)> {
        for reduce in [false, true] {
            if self.try_mount(page, heading, reduce).is_none() {
                self.skips.push(format!(
                    "{}/{}: heading did not mount",
                    page.title(),
                    heading
                ));
                return None;
            }
            let quiet_mount = self.settle();
            let rest = self.snapshot();
            self.tick(1_000);
            let quiet_later = self.settle();
            let later = self.snapshot();
            let stable = quiet_mount
                && quiet_later
                && scene::equivalent(&rest, &later, self.scale).is_equal();
            if stable {
                if reduce {
                    // Each example mounts twice (hover pass, then press pass);
                    // report the fallback once.
                    let key = format!("{}/{}", page.title(), heading);
                    if self.fallback_reported.insert(key.clone()) {
                        self.fallbacks.push(format!(
                            "{key}: unstable under full motion; swept with reduce"
                        ));
                    }
                    return Some((rest, true));
                }
                return Some((rest, false));
            }
        }
        self.skips.push(format!(
            "{}/{}: rest state never settles",
            page.title(),
            heading
        ));
        None
    }

    /// The union of the rest frame's quads (minus window-filling and
    /// dot-sized marks) and underlines, padded and clamped to the window —
    /// the region the discovery grid sweeps, in logical pixels.
    fn bbox(&self, rest: &Snapshot) -> (f32, f32, f32, f32) {
        let scale = self.scale;
        let win_w = f32::from(self.window_size.width) * scale;
        let win_h = f32::from(self.window_size.height) * scale;
        let win_area = win_w * win_h;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut any = false;
        let mut grow = |bounds: &gpui::Bounds<gpui::ScaledPixels>| {
            min_x = min_x.min(bounds.left().as_f32());
            min_y = min_y.min(bounds.top().as_f32());
            max_x = max_x.max(bounds.right().as_f32());
            max_y = max_y.max(bounds.bottom().as_f32());
            any = true;
        };
        for quad in &rest.quads {
            let area = quad.bounds.size.width.as_f32() * quad.bounds.size.height.as_f32();
            if area >= 0.9 * win_area {
                continue;
            }
            grow(&quad.bounds);
        }
        for underline in &rest.underlines {
            grow(&underline.bounds);
        }
        if !any {
            return (
                0.,
                0.,
                f32::from(self.window_size.width),
                f32::from(self.window_size.height),
            );
        }
        let pad = BBOX_PAD * scale;
        let left = (min_x - pad).max(0.) / scale;
        let top = (min_y - pad).max(0.) / scale;
        let right = (max_x + pad).min(win_w) / scale;
        let bottom = (max_y + pad).min(win_h) / scale;
        (left, top, right, bottom)
    }

    /// Sweeps a 16px grid over the bbox with the clock frozen. A grid point
    /// becomes a candidate when the move repaints and the scene's structural
    /// diff has a signature (changed primitive bounds, sorted) not seen
    /// before on this example.
    fn discover(&mut self, rest: &Snapshot) -> Vec<(f32, f32)> {
        let bbox = self.bbox(rest);
        let mut candidates = Vec::new();
        let mut signatures = HashSet::new();
        let mut previous = rest.clone();
        let debug = self.debug;
        let mut y = bbox.1;
        'rows: while y <= bbox.3 {
            let mut x = bbox.0;
            while x <= bbox.2 {
                let target = (x, y);
                let before = sweep::draws();
                self.move_to(target);
                if self.nonconverged {
                    // The breaker parked the pointer: this example's
                    // interaction never settles, so stop probing it.
                    break 'rows;
                }
                let drew = sweep::draws() != before;
                if debug {
                    eprintln!(
                        "  grid ({x:.0},{y:.0}) drew={} draws_delta={}",
                        drew,
                        sweep::draws() - before
                    );
                }
                if drew {
                    // A fade at delta 0 paints its from-colour, which can be
                    // field-identical to rest; advancing ~50ms makes every
                    // hover fill visibly different so no target is missed.
                    self.tick(50);
                    let snapshot = self.snapshot();
                    // Order-inclusive: a hover's element→fill move can swap
                    // two field-identical quads, invisible to the visual
                    // diff but still proof the point is interactive.
                    let difference = scene::diff_ordered(&previous, &snapshot);
                    if !difference.is_empty() {
                        let signature = signature(&difference, self.scale);
                        if signatures.insert(signature) {
                            candidates.push(target);
                        }
                    }
                    previous = snapshot;
                }
                x += GRID_STEP;
            }
            y += GRID_STEP;
        }
        candidates
    }

    /// Check (1): `a` must equal `b`; on failure, records a finding with up
    /// to five mismatching points and the differing quads' bounds.
    #[allow(clippy::too_many_arguments)] // the two snapshots plus the finding's page/pass/point context
    fn check_equivalent(
        &mut self,
        a: &Snapshot,
        b: &Snapshot,
        page: Page,
        heading: &str,
        pass: &'static str,
        check: u8,
        point: Option<(f32, f32)>,
    ) {
        if let scene::Equivalence::Different { points } = scene::equivalent(a, b, self.scale) {
            let difference = scene::diff(a, b);
            let quads = difference
                .quads_a
                .iter()
                .chain(difference.quads_b.iter())
                .take(3)
                .map(|quad| format!("{:?}", scene::logical_bounds(quad, self.scale)))
                .collect::<Vec<_>>()
                .join(", ");
            let samples = points
                .iter()
                .map(|m| format!("({:.0},{:.0}): {} vs {}", m.point.0, m.point.1, m.a, m.b))
                .collect::<Vec<_>>()
                .join("; ");
            self.finding(
                page,
                heading,
                pass,
                check,
                point,
                format!(
                    "{} point(s) differ; quads [{}]; {}",
                    points.len(),
                    quads,
                    samples
                ),
            );
        }
    }

    /// Check (2): leaving must never increase a point's distance from the
    /// rest composite. `prev` is the frame before the leave step, `next` the
    /// frame after. Aggregates to one finding per (target, leg) — the count
    /// of failing points and the worst one.
    #[allow(clippy::too_many_arguments)] // the three snapshots plus the finding's page/pass/leg/point context
    fn check_monotonic(
        &mut self,
        rest: &Snapshot,
        prev: &Snapshot,
        next: &Snapshot,
        page: Page,
        heading: &str,
        pass: &'static str,
        leg: &'static str,
        point: Option<(f32, f32)>,
    ) {
        let Some(target) = point else { return };
        let violations = scene::monotonic_leave_violations(rest, prev, next, target, self.scale);
        if violations.is_empty() {
            return;
        }
        let worst = violations
            .iter()
            .max_by(|a, b| (a.after - a.before).total_cmp(&(b.after - b.before)))
            .expect("non-empty");
        self.finding(
            page,
            heading,
            pass,
            2,
            point,
            format!(
                "{leg}: {} point(s) jumped away from rest; worst at ({:.0},{:.0}): {:.3} -> {:.3}",
                violations.len(),
                worst.point.0 / self.scale,
                worst.point.1 / self.scale,
                worst.before,
                worst.after
            ),
        );
    }

    /// Records a finding unless an [`EXPECTED`] entry excuses it; a match is
    /// remembered so `finish` can fail on exemptions that never fired.
    fn finding(
        &mut self,
        page: Page,
        heading: &str,
        pass: &'static str,
        check: u8,
        point: Option<(f32, f32)>,
        detail: String,
    ) {
        for (index, expected) in EXPECTED.iter().enumerate() {
            if expected.page == page.title()
                && expected.heading == heading
                && expected.check == check
                && expected.pass == pass
            {
                self.expected_hits[index] = true;
                return;
            }
        }
        self.findings.push(Finding {
            page: page.title(),
            heading: heading.to_owned(),
            pass,
            check,
            point,
            detail,
        });
    }

    /// Records a latched non-convergence once per example and reports
    /// whether the breaker is currently tripped, so callers abandon the
    /// phase. The pointer was already parked by the breaker; this is the
    /// finding that the interaction's feedback never settled.
    fn report_nonconverged(&mut self, page: Page, heading: &str, pass: &'static str) -> bool {
        if !self.nonconverged {
            return false;
        }
        self.nonconverged = false;
        if !self.nonconverged_reported {
            self.nonconverged_reported = true;
            self.finding(
                page,
                heading,
                pass,
                CHECK_NONCONVERGENCE,
                None,
                format!(
                    "redraws did not settle within {DRAW_BUDGET} draws of one update; \
                     the pointer was parked to break the loop"
                ),
            );
        }
        true
    }

    /// The hover pass: per candidate, hover mid-fade, check fill-over-text,
    /// leave and check monotonicity twice, without settling between targets.
    /// Afterwards the frame must return to rest and stay converged.
    fn hover_pass(
        &mut self,
        rest: &Snapshot,
        rest_keys: &HashSet<String>,
        candidates: &[(f32, f32)],
        page: Page,
        heading: &str,
    ) {
        for &target in candidates {
            self.move_to(target);
            if self.nonconverged {
                break;
            }
            self.tick(0);
            self.tick(40);
            let mid = self.snapshot();
            for (quad, underlines) in scene::fill_over_text(&mid, rest_keys, target, self.scale) {
                let covered = underlines
                    .iter()
                    .map(|underline| {
                        format!(
                            "{:?}",
                            scene::logical_bounds_from_bounds(&underline.bounds, self.scale)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                self.finding(
                    page,
                    heading,
                    "hover",
                    3,
                    Some(target),
                    format!(
                        "quad {:?} bg {:?} order {} paints over {} underline(s) [{}]",
                        scene::logical_bounds(&quad, self.scale),
                        quad.background,
                        quad.order,
                        underlines.len(),
                        covered
                    ),
                );
            }
            self.move_to(EMPTY);
            self.tick(0);
            let left0 = self.snapshot();
            self.check_monotonic(
                rest,
                &mid,
                &left0,
                page,
                heading,
                "hover",
                "leave0",
                Some(target),
            );
            self.tick(16);
            let left1 = self.snapshot();
            self.check_monotonic(
                rest,
                &left0,
                &left1,
                page,
                heading,
                "hover",
                "leave1",
                Some(target),
            );
        }
        if self.nonconverged {
            // Parked mid-pass: the rest/convergence checks below would read a
            // broken frame, so leave them to the non-convergence finding.
            return;
        }
        self.settle();
        let final_frame = self.snapshot();
        self.check_equivalent(rest, &final_frame, page, heading, "hover", 1, None);
        self.tick(1_000);
        self.settle();
        let converged = self.snapshot();
        self.check_equivalent(
            &final_frame,
            &converged,
            page,
            heading,
            "hover",
            CHECK_CONVERGENCE,
            None,
        );
    }

    /// The press pass on a fresh mount: per candidate, settle the hover,
    /// press mid-fade, check fill-over-text, release at EMPTY without a
    /// click, then clean up.
    fn press_pass(
        &mut self,
        rest_keys: &HashSet<String>,
        candidates: &[(f32, f32)],
        page: Page,
        heading: &str,
    ) {
        for &target in candidates {
            self.move_to(target);
            if self.nonconverged {
                break;
            }
            self.tick(0);
            self.tick(150);
            self.down(target);
            self.tick(0);
            self.tick(40);
            let pressed = self.snapshot();
            for (quad, underlines) in scene::fill_over_text(&pressed, rest_keys, target, self.scale)
            {
                let covered = underlines
                    .iter()
                    .map(|underline| {
                        format!(
                            "{:?}",
                            scene::logical_bounds_from_bounds(&underline.bounds, self.scale)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                self.finding(
                    page,
                    heading,
                    "press",
                    3,
                    Some(target),
                    format!(
                        "quad {:?} bg {:?} order {} paints over {} underline(s) [{}]",
                        scene::logical_bounds(&quad, self.scale),
                        quad.background,
                        quad.order,
                        underlines.len(),
                        covered
                    ),
                );
            }
            // Release over EMPTY with no move in between: no click, no drag.
            self.up(EMPTY);
            self.move_to(EMPTY);
            self.tick(0);
            // Dismiss anything the press opened, then let release fades run.
            self.click(EMPTY);
            self.blur_quiet();
            self.tick(0);
            self.tick(400);
        }
    }

    /// One example end to end: mount, discover, residue check, hover pass,
    /// press pass on a fresh mount.
    fn sweep_example(&mut self, page: Page, heading: String) {
        if let Some(filter) = &self.page_filter {
            if page.title() != filter {
                return;
            }
        }
        if let Some(filter) = &self.section_filter {
            if heading != *filter {
                return;
            }
        }
        let started = Instant::now();
        let name = format!("{}/{}", page.title(), heading);
        self.examples += 1;
        self.nonconverged = false;
        self.nonconverged_reported = false;
        if self.debug {
            eprintln!("sweep {name}");
        }

        let debug = self.debug;
        let Some((mut rest, _)) = self.mount_example(page, &heading) else {
            self.timings
                .push((name, started.elapsed().as_secs_f64() * 1000.));
            return;
        };
        self.swept_examples
            .insert((page.title().to_owned(), heading.clone()));
        if debug {
            eprintln!("  mounted {name}");
        }

        let candidates = self.discover(&rest);
        if debug {
            eprintln!("  discovered {name} ({} candidates)", candidates.len());
        }
        if self.report_nonconverged(page, &heading, "discovery") {
            self.timings
                .push((name, started.elapsed().as_secs_f64() * 1000.));
            return;
        }

        // Discovery residue: leaving every probed point and settling must
        // return to the same rest frame.
        self.move_to(EMPTY);
        self.settle();
        let after_discovery = self.snapshot();
        if !scene::equivalent(&rest, &after_discovery, self.scale).is_equal() {
            self.check_equivalent(
                &rest,
                &after_discovery,
                page,
                &heading,
                "discovery",
                1,
                None,
            );
            match self.mount_example(page, &heading) {
                Some((remounted, _)) => rest = remounted,
                None => {
                    self.timings
                        .push((name, started.elapsed().as_secs_f64() * 1000.));
                    return;
                }
            }
        }
        if self.report_nonconverged(page, &heading, "discovery") {
            self.timings
                .push((name, started.elapsed().as_secs_f64() * 1000.));
            return;
        }

        self.candidates_total += candidates.len();
        self.candidates_max = self.candidates_max.max(candidates.len());

        let rest_keys = scene::quad_keys(&rest);
        self.hover_pass(&rest, &rest_keys, &candidates, page, &heading);
        if debug {
            eprintln!("  hovered {name}");
        }
        if self.report_nonconverged(page, &heading, "hover") {
            self.timings
                .push((name, started.elapsed().as_secs_f64() * 1000.));
            return;
        }

        // The press pass starts from a fresh mount so hover-pass state
        // cannot leak into it.
        if let Some((press_rest, _)) = self.mount_example(page, &heading) {
            let press_keys = scene::quad_keys(&press_rest);
            self.press_pass(&press_keys, &candidates, page, &heading);
            self.report_nonconverged(page, &heading, "press");
        }
        if debug {
            eprintln!("  pressed {name}");
        }

        if self.debug {
            eprintln!(
                "  done {name} {:.0}ms",
                started.elapsed().as_secs_f64() * 1000.
            );
        }
        self.timings
            .push((name, started.elapsed().as_secs_f64() * 1000.));
    }

    /// Per-shard summary on stderr, then one panic carrying every finding.
    fn finish(mut self) {
        let wall = self.started.elapsed();
        let draws = sweep::draws() - self.draws_start;
        let avg_candidates = if self.examples == 0 {
            0.
        } else {
            self.candidates_total as f64 / self.examples as f64
        };
        eprintln!(
            "sweep shard {}: {} examples, {:.1}s wall, {} draws, {} candidates (avg {:.1}, max {})",
            self.shard,
            self.examples,
            wall.as_secs_f64(),
            draws,
            self.candidates_total,
            avg_candidates,
            self.candidates_max
        );
        let mut timings = self.timings;
        timings.sort_by(|a, b| b.1.total_cmp(&a.1));
        eprintln!("sweep shard {}: slowest examples:", self.shard);
        for (name, ms) in timings.iter().take(10) {
            eprintln!("  {ms:9.1}ms  {name}");
        }
        for skip in &self.skips {
            eprintln!("sweep skip: {skip}");
        }
        for fallback in &self.fallbacks {
            eprintln!("sweep reduced-motion fallback: {fallback}");
        }
        // An exemption that matched nothing on an example this shard swept
        // has outlived its behaviour and must be removed.
        for (index, expected) in EXPECTED.iter().enumerate() {
            if self.expected_hits[index]
                || !self
                    .swept_examples
                    .contains(&(expected.page.to_owned(), expected.heading.to_owned()))
            {
                continue;
            }
            self.findings.push(Finding {
                page: expected.page,
                heading: expected.heading.to_owned(),
                pass: expected.pass,
                check: CHECK_STALE_EXEMPTION,
                point: None,
                detail: format!("stale exemption ({}): {}", expected.check, expected.reason),
            });
        }
        if let Some(dir) = std::env::var_os("HEROGPUI_SWEEP_REPORT_DIR") {
            let mut swept: Vec<_> = self.swept_examples.iter().collect();
            swept.sort();
            let mut report = String::new();
            for (page, heading) in swept {
                let checks: Vec<String> = self
                    .findings
                    .iter()
                    .filter(|finding| finding.page == page.as_str() && finding.heading == *heading)
                    .map(|finding| finding.check.to_string())
                    .collect();
                report.push_str(&format!("{page}\t{heading}\t{}\n", checks.join(",")));
            }
            let path = std::path::Path::new(&dir).join(format!("shard-{}.tsv", self.shard));
            std::fs::write(&path, report).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        }
        if !self.findings.is_empty() {
            let mut counts: std::collections::BTreeMap<(u8, &'static str), usize> =
                std::collections::BTreeMap::new();
            for finding in &self.findings {
                *counts.entry((finding.check, finding.pass)).or_default() += 1;
            }
            let mut message = format!(
                "interaction sweep shard {}: {} findings\n",
                self.shard,
                self.findings.len()
            );
            for ((check, pass), count) in &counts {
                message.push_str(&format!("  check {check} ({pass}): {count}\n"));
            }
            for finding in self.findings.iter().take(40) {
                let point = finding
                    .point
                    .map(|p| format!(" @({:.0},{:.0})", p.0, p.1))
                    .unwrap_or_default();
                message.push_str(&format!(
                    "  {}/{} {} check{}{}: {}\n",
                    finding.page,
                    finding.heading,
                    finding.pass,
                    finding.check,
                    point,
                    finding.detail
                ));
            }
            if self.findings.len() > 40 {
                message.push_str(&format!("  … and {} more\n", self.findings.len() - 40));
            }
            panic!("{message}");
        }
    }
}

/// The discovery signature: sorted bounds (1px-rounded, logical) of every
/// primitive in the diff, tagged by kind so a quad and an underline at the
/// same bounds do not collide.
fn signature(difference: &scene::SceneDiff, scale: f32) -> Vec<(u8, i32, i32, i32, i32)> {
    let mut entries = Vec::new();
    for quad in difference.quads_a.iter().chain(difference.quads_b.iter()) {
        let bounds = &quad.bounds;
        entries.push((
            0,
            (bounds.left().as_f32() / scale).round() as i32,
            (bounds.top().as_f32() / scale).round() as i32,
            (bounds.size.width.as_f32() / scale).round() as i32,
            (bounds.size.height.as_f32() / scale).round() as i32,
        ));
    }
    for underline in difference
        .underlines_a
        .iter()
        .chain(difference.underlines_b.iter())
    {
        let bounds = &underline.bounds;
        entries.push((
            1,
            (bounds.left().as_f32() / scale).round() as i32,
            (bounds.top().as_f32() / scale).round() as i32,
            (bounds.size.width.as_f32() / scale).round() as i32,
            (bounds.size.height.as_f32() / scale).round() as i32,
        ));
    }
    entries.sort_unstable();
    entries
}
