//! T119: how much processor time does an idle GPUI window cost by itself?
//!
//! The shell idles at about 1.5% of a core against a 1% budget, and it draws no
//! frames while doing it. This opens a window containing nothing and measures
//! the same way, to separate what the framework costs from what the shell adds.
//!
//! Measured 2026-09-19 on Linux with software rendering: 1.34%, 1.19%, 1.10%
//! for an empty window, against 1.14% to 1.35% for the whole shell. The shell
//! adds nothing measurable. The floor is GPUI's.
//!
//! Kept so the question can be asked again after a framework upgrade, which is
//! the only thing likely to move the number.

use gpui::{div, px, size, App, AppContext, Application, Bounds, Context, IntoElement,
           ParentElement, Render, Styled, Window, WindowBounds, WindowOptions};
use vulcan_adapters::measurement::instrument::Instrument;
use vulcan_domain::budget::Runner;

struct Empty;

impl Render for Empty {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child("")
    }
}

fn main() {
    let instrument = std::sync::Arc::new(Instrument::new());

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            |_, cx| cx.new(|_| Empty),
        )
        .expect("window opens");
        cx.activate(true);

        let instrument = instrument.clone();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(std::time::Duration::from_secs(2)).await;

            let idle = instrument.clone();
            cx.background_executor()
                .spawn(async move { idle.observe_idle_over(std::time::Duration::from_secs(2)) })
                .await;

            let report = instrument.to_json(Runner::LinuxCgroup, "probe", 0);
            println!("{report}");
            cx.update(|cx| cx.quit()).ok();
        })
        .detach();
    });
}
