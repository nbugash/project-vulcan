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

pub struct GpuiCaptureAdapter {
    preview_binary: PathBuf,
    settle: std::time::Duration,
}

impl Default for GpuiCaptureAdapter {
    fn default() -> Self {
        Self {
            preview_binary: PathBuf::from("target/debug/shell-preview"),
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
