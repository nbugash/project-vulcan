//! Runs the shell standalone, so the fidelity and budget gates have a subject.
//!
//! `--screenshot <path>` renders and captures without a human watching. Capture
//! runs under the headless Wayland session the pinned environment provides:
//! GPUI presents a Vulkan swapchain, and on X11 that handoff needs DRI3, which a
//! machine without a GPU does not have.

use gpui::{px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use vulcan_adapters::measurement::instrument::{Instrument, Span};
use vulcan_domain::budget::Runner;
use vulcan_ui::palette::Mode;
use vulcan_ui::props::{CompletionStyle, Overlay, PerfReadout, Props, RailTab};
use vulcan_ui::shell::Shell;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let screenshot = args
        .windows(2)
        .find(|pair| pair[0] == "--screenshot")
        .map(|pair| pair[1].clone());
    let smoke = args.iter().any(|arg| arg == "--smoke");
    let measure = args
        .windows(2)
        .find(|pair| pair[0] == "--measure")
        .map(|pair| pair[1].clone());

    if smoke {
        // Construct without a window: proves the layout resolves from tokens
        // on a machine with no display at all.
        let shell = Shell::new(Props::default());
        println!(
            "shell resolves: tool window {}px, dock {}px, code line {}px",
            shell.profile().tool_window_width,
            shell.profile().dock_height,
            shell.profile().code_line_height
        );
        return;
    }

    // The product measures itself: the frame loop knows its own timings
    // exactly, where an outside observer could only sample them.
    let instrument = std::sync::Arc::new(Instrument::new());

    Application::new().run(move |cx: &mut App| {
        // The prototype's typefaces, from the repository. Without these the
        // shell renders in a fallback font and every comparison is meaningless.
        cx.text_system()
            .add_fonts(vulcan_ui::fonts::embedded())
            .expect("vendored typefaces load");

        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let recorder = instrument.clone();
                cx.new(move |_| {
                    let overlay = match std::env::var("VULCAN_OVERLAY").as_deref() {
                        Ok("palette") => Overlay::Palette,
                        Ok("completion") => Overlay::Completion,
                        Ok("runconfig") => Overlay::RunConfig,
                        _ => Overlay::None,
                    };
                    let rail_tab = match std::env::var("VULCAN_RAIL").as_deref() {
                        Ok("structure") => RailTab::Structure,
                        Ok("commit") => RailTab::Commit,
                        Ok("find") => RailTab::Find,
                        Ok("history") => RailTab::History,
                        Ok("packs") => RailTab::Packs,
                        _ => RailTab::Project,
                    };
                    let palette_mode = match std::env::var("VULCAN_PALETTE").as_deref() {
                        Ok("commands") => Mode::Commands,
                        Ok("structural") => Mode::Structural,
                        _ => Mode::Files,
                    };
                    Shell::measured(Props {
                        overlay,
                        completion_open: std::env::var("VULCAN_COMPLETION").as_deref() != Ok("off"),
                        rail_tab,
                        palette_mode,
                        // The prototype's depicted state, unless asked otherwise.
                        completion_style: match std::env::var("VULCAN_COMPLETION").as_deref() {
                            Ok("compact") => CompletionStyle::Compact,
                            _ => CompletionStyle::Detail,
                        },
                        perf_readout: match std::env::var("VULCAN_PERF").as_deref() {
                            Ok("status") => PerfReadout::Status,
                            Ok("off") => PerfReadout::Off,
                            _ => PerfReadout::Hud,
                        },
                        side_collapsed: std::env::var("VULCAN_SIDE").as_deref() == Ok("collapsed"),
                        dock_collapsed: std::env::var("VULCAN_DOCK").as_deref() == Ok("collapsed"),
                        ..Props::default()
                    }, recorder)
                })
            },
        )
        .expect("shell window opens");
        cx.activate(true);
        // Cold start ends when the shell paints its first frame, which the shell
        // itself reports through the recorder. Marking it here would measure
        // this file rather than the product.

        if let Some(path) = measure.clone() {
            let instrument = instrument.clone();
            cx.spawn(async move |cx| {
                // Make the window key before anything is measured.
                //
                // `cx.activate(true)` above activates the application, which is
                // not the same thing. On macOS the draw loop is a CVDisplayLink
                // that GPUI starts from `windowDidBecomeKey` and from a change
                // in occlusion state; a window that never becomes key never
                // starts it. The dirty flag was being set correctly on every one
                // of the 300 inputs below and nothing ever ticked to consume it,
                // so a run on an M3 Pro drew two frames in two and a half
                // seconds and reported no KeystrokeToPaint at all.
                //
                // Linux hid this: there the frame loop runs regardless, so the
                // input-to-paint figures happened to be collected.
                let _ = window.update(cx, |_, win, _| win.activate_window());

                // Let the first frame land, then record a run of frames.
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(3))
                    .await;

                // Drive the shell the way a person does, so the input-to-paint
                // budget has something to measure. Each of these marks an input
                // and causes a frame; the shell times the gap between them from
                // inside its own render path.
                //
                // Without this the run reports no KeystrokeToPaint at all, which
                // is honest but useless: the budget that matters most goes
                // unmeasured on every run.
                for step in 0..300u32 {
                    let _ = window.update(cx, |shell, win, cx| {
                        match step % 4 {
                            0 => shell.select_rail(RailTab::Structure),
                            1 => shell.select_tab((step as usize / 4) % 3),
                            2 => shell.select_rail(RailTab::Project),
                            _ => shell.toggle_hud(),
                        }
                        cx.notify();
                        // Ask the window for a frame as well as marking the view
                        // dirty. `notify` reaches the window only through the
                        // invalidator registered for this entity; `refresh` is
                        // the window's own documented way to say redraw me, and
                        // the two disagreeing is not a difference worth relying
                        // on when the whole point is to count frames.
                        win.refresh();
                    });
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(8))
                        .await;
                }

                // Wait until the shell is actually idle before measuring what
                // idling costs.
                //
                // The input burst above leaves a queue of frames behind it, and
                // measuring straight afterwards charges that backlog to idle. It
                // read 15.32% on a run where quiet runs read 1.5%, which is how
                // this came to be here. Quiescence is a run of frames in which
                // nothing was drawn, not a fixed delay, because how long the
                // backlog takes depends on the machine.
                let mut settled = 0;
                let mut last = instrument.sample_count(Span::UiThreadTask);
                for _ in 0..100 {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(50))
                        .await;
                    let now = instrument.sample_count(Span::UiThreadTask);
                    settled = if now == last { settled + 1 } else { 0 };
                    last = now;
                    if settled >= 4 {
                        break;
                    }
                }



                // The input above must have produced frames, or KeystrokeToPaint
                // is silently absent and the gate refuses without saying why.
                let painted = instrument.sample_count(Span::KeystrokeToPaint);
                // Whether the window was active is printed beside the counts,
                // because it is the first thing worth knowing when they are low
                // and the only way to tell a window that would not draw from a
                // shell that would not respond.
                let active = window.update(cx, |_, win, _| win.is_window_active()).unwrap_or(false);
                eprintln!(
                    "measured: {painted} input-to-paint samples, {last} frames in total, \
                     window active: {active}"
                );
                instrument.sample_memory();

                let report = instrument.to_json(Runner::LinuxCgroup, "unconstrained", 0);
                let _ = std::fs::write(&path, report);
                println!("measurements written to {path}");
                cx.update(|cx| cx.quit()).ok();
            })
            .detach();
        }

        if let Some(path) = screenshot.clone() {
            // The compositor captures the surface once a frame has been
            // presented; the session script drives that, so exit after it.
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(8))
                    .await;
                println!("shell rendered; capture expected at {path}");
                cx.update(|cx| cx.quit()).ok();
            })
            .detach();
        }
    });
}
