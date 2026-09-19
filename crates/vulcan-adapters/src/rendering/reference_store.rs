//! Reading and writing fidelity references.
//!
//! A reference records what it was captured against: the prototype's digest, the
//! environment's digest and the typefaces present. A comparison failure can then
//! name its cause rather than only its size.

use std::path::{Path, PathBuf};

use vulcan_domain::rendering::{Image, Viewport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceMetadata {
    pub viewport: Viewport,
    pub typefaces: Vec<String>,
    pub prototype_digest: String,
    pub environment_digest: String,
    pub captured_at: String,
}

#[derive(Debug)]
pub enum ReferenceError {
    Io(String),
    Decode(String),
    Missing(String),
}

pub struct ReferenceStore {
    directory: PathBuf,
}

impl ReferenceStore {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self { directory: directory.into() }
    }

    pub fn image_path(&self, name: &str) -> PathBuf {
        self.directory.join(format!("{name}.png"))
    }

    pub fn metadata_path(&self, name: &str) -> PathBuf {
        self.directory.join(format!("{name}.json"))
    }

    pub fn save(&self, name: &str, image: &Image, metadata: &ReferenceMetadata) -> Result<(), ReferenceError> {
        std::fs::create_dir_all(&self.directory).map_err(|e| ReferenceError::Io(e.to_string()))?;
        write_png(&self.image_path(name), image)?;
        std::fs::write(self.metadata_path(name), render_metadata(metadata))
            .map_err(|e| ReferenceError::Io(e.to_string()))
    }

    pub fn load(&self, name: &str) -> Result<(Image, ReferenceMetadata), ReferenceError> {
        let image = read_png(&self.image_path(name))?;
        let text = std::fs::read_to_string(self.metadata_path(name))
            .map_err(|_| ReferenceError::Missing(self.metadata_path(name).display().to_string()))?;
        Ok((image, parse_metadata(&text, image_viewport(&text))))
    }
}

pub fn read_png(path: &Path) -> Result<Image, ReferenceError> {
    let file = std::fs::File::open(path)
        .map_err(|_| ReferenceError::Missing(path.display().to_string()))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().map_err(|e| ReferenceError::Decode(e.to_string()))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| ReferenceError::Decode("image too large to decode".into()))?;
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer).map_err(|e| ReferenceError::Decode(e.to_string()))?;
    buffer.truncate(info.buffer_size());

    // Normalise to RGBA8 so comparison never depends on the source encoding.
    let pixels = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::Rgb => buffer
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        other => return Err(ReferenceError::Decode(format!("unsupported colour type {other:?}"))),
    };

    Ok(Image {
        viewport: Viewport { width: info.width, height: info.height },
        pixels,
    })
}

pub fn write_png(path: &Path, image: &Image) -> Result<(), ReferenceError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ReferenceError::Io(e.to_string()))?;
    }
    let file = std::fs::File::create(path).map_err(|e| ReferenceError::Io(e.to_string()))?;
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(file),
        image.viewport.width,
        image.viewport.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .map_err(|e| ReferenceError::Io(e.to_string()))?
        .write_image_data(&image.pixels)
        .map_err(|e| ReferenceError::Io(e.to_string()))
}

fn render_metadata(metadata: &ReferenceMetadata) -> String {
    format!(
        "{{\n  \"viewport\": {{ \"width\": {}, \"height\": {} }},\n  \"typefaces\": [{}],\n  \"prototype_digest\": \"{}\",\n  \"environment_digest\": \"{}\",\n  \"captured_at\": \"{}\"\n}}",
        metadata.viewport.width,
        metadata.viewport.height,
        metadata.typefaces.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", "),
        metadata.prototype_digest,
        metadata.environment_digest,
        metadata.captured_at
    )
}

fn image_viewport(text: &str) -> Viewport {
    Viewport {
        width: field(text, "width").and_then(|v| v.parse().ok()).unwrap_or(0),
        height: field(text, "height").and_then(|v| v.parse().ok()).unwrap_or(0),
    }
}

fn parse_metadata(text: &str, viewport: Viewport) -> ReferenceMetadata {
    ReferenceMetadata {
        viewport,
        typefaces: quoted_list(text, "typefaces"),
        prototype_digest: quoted(text, "prototype_digest").unwrap_or_default(),
        environment_digest: quoted(text, "environment_digest").unwrap_or_default(),
        captured_at: quoted(text, "captured_at").unwrap_or_default(),
    }
}

fn field(text: &str, key: &str) -> Option<String> {
    let rest = text.split(&format!("\"{key}\":")).nth(1)?.trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit())?;
    Some(rest[..end].to_string())
}

fn quoted(text: &str, key: &str) -> Option<String> {
    let rest = text.split(&format!("\"{key}\":")).nth(1)?;
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')?;
    Some(rest[start..start + end].to_string())
}

fn quoted_list(text: &str, key: &str) -> Vec<String> {
    text.split(&format!("\"{key}\":"))
        .nth(1)
        .and_then(|rest| rest.split(']').next())
        .map(|list| {
            list.split('"')
                .filter(|part| !part.trim().is_empty() && !part.contains('[') && !part.contains(','))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
