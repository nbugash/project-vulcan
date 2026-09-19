//! Runs the shell standalone, so the fidelity and budget gates have a subject.
//!
//! `--screenshot <path>` renders and captures without a human watching. Capture
//! runs under the headless Wayland session the pinned environment provides:
//! GPUI presents a Vulkan swapchain, and on X11 that handoff needs DRI3, which a
//! machine without a GPU does not have.

use gpui::{px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};
use vulcan_adapters::measurement::instrument::Instrument;
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
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| {
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
                    Shell::new(Props {
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
                    })
                })
            },
        )
        .expect("shell window opens");
        cx.activate(true);
        // Cold start ends when the window exists, not when the harness finishes
        // waiting; marking it later would measure this file instead of the shell.
        instrument.mark_first_frame();

        if let Some(path) = measure.clone() {
            let instrument = instrument.clone();
            cx.spawn(async move |cx| {
                // Let the first frame land, then record a run of frames.
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(3))
                    .await;

                // Idle behaviour is real: the shell is up and doing nothing, and
                // this samples what that costs.
                // Overshoot past the requested sleep is work the shell did while
                // idle: scheduling, compositing, anything on a timer.
                let requested = std::time::Duration::from_millis(8);
                for _ in 0..120 {
                    let started = std::time::Instant::now();
                    cx.background_executor().timer(requested).await;
                    let window = started.elapsed();
                    instrument.observe_idle(window.saturating_sub(requested), window);
                }
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
