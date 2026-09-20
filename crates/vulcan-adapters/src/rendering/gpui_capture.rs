//! Capturing the shell.
//!
//! GPUI presents a Vulkan swapchain. On X11 that handoff needs DRI3, which a
//! machine without a GPU does not provide, so capture under a virtual X
//! framebuffer returns an empty image with no error to explain it. Wayland
//! presents through shared memory instead, so this captures from a headless
//! wlroots session through `wlr-screencopy`, which needs no GPU at all.

use std::path::PathBuf;

use vulcan_app::ports::render_capture::{RenderCapturePort, RenderError};
use vulcan_domain::rendering::{Image, Viewport};

use super::reference_store::read_png;

/// The floor a real render clears without trying. The shell's own reference
/// holds hundreds; an empty desktop holds one. Set low on purpose: this
/// distinguishes drawing from not drawing, and is not a fidelity judgement.
const MINIMUM_COLOURS: usize = 16;

pub struct GpuiCaptureAdapter {
    preview_binary: PathBuf,
    settle: std::time::Duration,
}

impl Default for GpuiCaptureAdapter {
    fn default() -> Self {
        Self {
            preview_binary: preview_binary(),
            // The compositor needs a presented frame before there is anything
            // to copy; capturing sooner yields the pre-first-frame surface.
            settle: std::time::Duration::from_secs(9),
        }
    }
}

impl GpuiCaptureAdapter {
    /// Whether this process is inside the pinned comparison environment.
    ///
    /// Exposed because a caller must be able to refuse *before* reaching any
    /// code that could return a pass or a fail. Outside the pinned environment
    /// there is no verdict to give, only a refusal.
    pub fn in_pinned_environment() -> Result<(), RenderError> {
        if std::env::var("VULCAN_PINNED_ENV").as_deref() != Ok("1") {
            return Err(RenderError::NotPinnedEnvironment);
        }
        Ok(())
    }
}

impl RenderCapturePort for GpuiCaptureAdapter {
    fn capture(&self, viewport: Viewport) -> Result<Image, RenderError> {
        Self::in_pinned_environment()?;
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            return Err(RenderError::Failed(
                "no Wayland session; capture needs wl_shm, not an X framebuffer".into(),
            ));
        }
        typefaces_present()?;

        let output = std::env::temp_dir().join("vulcan-capture.png");
        let _ = std::fs::remove_file(&output);

        let mut shell = std::process::Command::new(&self.preview_binary)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| RenderError::Failed(format!("shell-preview: {error}")))?;
        std::thread::sleep(self.settle);

        let grim = std::process::Command::new("grim")
            .arg(&output)
            .output()
            .map_err(|error| RenderError::Failed(format!("grim: {error}")));
        let _ = shell.kill();
        let grim = grim?;

        if !grim.status.success() {
            return Err(RenderError::Failed(
                String::from_utf8_lossy(&grim.stderr).trim().to_string(),
            ));
        }

        let image = read_png(&output).map_err(|error| RenderError::Failed(format!("{error:?}")))?;
        if image.viewport != viewport {
            return Err(RenderError::Failed(format!(
                "captured {}x{}, expected {}x{}",
                image.viewport.width, image.viewport.height, viewport.width, viewport.height
            )));
        }
        // A capture with almost no colour in it is the compositor's empty
        // desktop, not the product. The pinned image once pointed
        // VK_ICD_FILENAMES at a filename its own distribution does not use, so
        // the Vulkan loader enumerated no driver, the shell drew nothing, and
        // `capture-reference` wrote the blank result out as the signed-off
        // reference and reported PASS. A gate must not be able to pass by
        // producing nothing.
        let colours = image.distinct_colours(MINIMUM_COLOURS);
        if colours < MINIMUM_COLOURS {
            return Err(RenderError::Failed(format!(
                "the capture holds {colours} distinct colours, fewer than the {MINIMUM_COLOURS} \
                 any rendered interface has: the shell did not draw"
            )));
        }
        Ok(image)
    }
}

/// The prototype's typefaces must be present, or the capture records a
/// substitute font and every later comparison inherits it (FR-020).
fn typefaces_present() -> Result<(), RenderError> {
    for required in [
        "assets/fonts/inter-latin-400.ttf",
        "assets/fonts/jetbrains-mono-400.ttf",
        "assets/fonts/Phosphor.ttf",
    ] {
        if !std::path::Path::new(required).exists() {
            return Err(RenderError::TypefaceMissing(required.to_string()));
        }
    }
    Ok(())
}

/// Where the shell binary is, which is not always `target/`.
///
/// The pinned environment builds into a target directory of its own, because
/// the repository is bind-mounted and binaries built against the host's C
/// library will not run against the container's.
fn preview_binary() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    PathBuf::from(target).join("debug").join("shell-preview")
}
